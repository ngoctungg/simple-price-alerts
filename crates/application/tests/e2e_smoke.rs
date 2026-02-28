use application::{AlertRepositoryPort, ApplicationError, NotifierPort, PriceAlertService, PriceIngestPort};
use async_trait::async_trait;
use chrono::Utc;
use domain::{PriceAlert, PriceSnapshot};
use std::sync::{Arc, Mutex};

struct InMemoryIngest;

#[async_trait]
impl PriceIngestPort for InMemoryIngest {
    async fn fetch_latest(&self, product_id: &str) -> Result<PriceSnapshot, ApplicationError> {
        PriceSnapshot::new(product_id, 49.0, Utc::now())
            .map_err(|e| ApplicationError::PriceFetch(e.to_string()))
    }
}

struct InMemoryRepo;

#[async_trait]
impl AlertRepositoryPort for InMemoryRepo {
    async fn find_for_product(&self, product_id: &str) -> Result<Vec<PriceAlert>, ApplicationError> {
        Ok(vec![PriceAlert::new(product_id, 50.0, "chat-smoke")
            .map_err(|e| ApplicationError::Repository(e.to_string()))?])
    }
}

#[derive(Default, Clone)]
struct SpyNotifier(Arc<Mutex<Vec<String>>>);

#[async_trait]
impl NotifierPort for SpyNotifier {
    async fn send(&self, _chat_id: &str, message: &str) -> Result<(), ApplicationError> {
        self.0.lock().unwrap().push(message.to_string());
        Ok(())
    }
}

#[tokio::test]
async fn smoke_pipeline_ingest_to_notify() {
    let notifier = SpyNotifier::default();
    let app = PriceAlertService::new(InMemoryIngest, InMemoryRepo, notifier.clone());

    let sent = app.ingest_and_notify("sku-smoke").await.unwrap();

    assert_eq!(sent, 1);
    assert_eq!(notifier.0.lock().unwrap().len(), 1);
}
