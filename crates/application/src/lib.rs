use async_trait::async_trait;
use domain::{PriceAlert, PriceSnapshot};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("price fetch failed: {0}")]
    PriceFetch(String),
    #[error("repository failed: {0}")]
    Repository(String),
    #[error("notification failed: {0}")]
    Notification(String),
}

#[async_trait]
pub trait PriceIngestPort: Send + Sync {
    async fn fetch_latest(&self, product_id: &str) -> Result<PriceSnapshot, ApplicationError>;
}

#[async_trait]
pub trait AlertRepositoryPort: Send + Sync {
    async fn find_for_product(&self, product_id: &str) -> Result<Vec<PriceAlert>, ApplicationError>;
}

#[async_trait]
pub trait NotifierPort: Send + Sync {
    async fn send(&self, chat_id: &str, message: &str) -> Result<(), ApplicationError>;
}

pub struct PriceAlertService<I, R, N> {
    ingest: I,
    repository: R,
    notifier: N,
}

impl<I, R, N> PriceAlertService<I, R, N>
where
    I: PriceIngestPort,
    R: AlertRepositoryPort,
    N: NotifierPort,
{
    pub fn new(ingest: I, repository: R, notifier: N) -> Self {
        Self {
            ingest,
            repository,
            notifier,
        }
    }

    pub async fn ingest_and_notify(&self, product_id: &str) -> Result<usize, ApplicationError> {
        let snapshot = self.ingest.fetch_latest(product_id).await?;
        let alerts = self.repository.find_for_product(product_id).await?;

        let mut sent = 0;
        for alert in alerts {
            if alert.should_notify(&snapshot) {
                self.notifier.send(&alert.chat_id, &alert.message).await?;
                sent += 1;
            }
        }

        Ok(sent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::sync::{Arc, Mutex};

    struct FakeIngest {
        snapshot: PriceSnapshot,
    }

    #[async_trait]
    impl PriceIngestPort for FakeIngest {
        async fn fetch_latest(&self, _product_id: &str) -> Result<PriceSnapshot, ApplicationError> {
            Ok(self.snapshot.clone())
        }
    }

    struct FakeRepo {
        alerts: Vec<PriceAlert>,
    }

    #[async_trait]
    impl AlertRepositoryPort for FakeRepo {
        async fn find_for_product(&self, _product_id: &str) -> Result<Vec<PriceAlert>, ApplicationError> {
            Ok(self.alerts.clone())
        }
    }

    #[derive(Default, Clone)]
    struct FakeNotifier {
        sent_messages: Arc<Mutex<Vec<(String, String)>>>,
    }

    #[async_trait]
    impl NotifierPort for FakeNotifier {
        async fn send(&self, chat_id: &str, message: &str) -> Result<(), ApplicationError> {
            self.sent_messages
                .lock()
                .expect("poisoned")
                .push((chat_id.to_string(), message.to_string()));
            Ok(())
        }
    }

    #[tokio::test]
    async fn sends_notifications_for_matching_alerts() {
        let ingest = FakeIngest {
            snapshot: PriceSnapshot::new("sku-1", 80.0, Utc::now()).unwrap(),
        };
        let repo = FakeRepo {
            alerts: vec![
                PriceAlert::new("sku-1", 99.0, "chat-1").unwrap(),
                PriceAlert::new("sku-1", 70.0, "chat-2").unwrap(),
            ],
        };
        let notifier = FakeNotifier::default();

        let service = PriceAlertService::new(ingest, repo, notifier.clone());
        let sent = service.ingest_and_notify("sku-1").await.unwrap();

        assert_eq!(sent, 1);
        let messages = notifier.sent_messages.lock().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].0, "chat-1");
    }
}
