use application::{ApplicationError, NotifierPort, PriceIngestPort};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::PriceSnapshot;
use reqwest::Client;
use serde::Deserialize;

#[derive(Clone)]
pub struct HttpPriceIngest {
    client: Client,
    base_url: String,
}

impl HttpPriceIngest {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::builder().no_proxy().build().expect("client"),
            base_url: base_url.into(),
        }
    }
}

#[derive(Deserialize)]
struct PriceApiResponse {
    product_id: String,
    observed_price: f64,
    observed_at: DateTime<Utc>,
}

#[async_trait]
impl PriceIngestPort for HttpPriceIngest {
    async fn fetch_latest(&self, product_id: &str) -> Result<PriceSnapshot, ApplicationError> {
        let response = self
            .client
            .get(format!("{}/prices/{}", self.base_url, product_id))
            .send()
            .await
            .map_err(|e| ApplicationError::PriceFetch(e.to_string()))?
            .error_for_status()
            .map_err(|e| ApplicationError::PriceFetch(e.to_string()))?;

        let payload: PriceApiResponse = response
            .json()
            .await
            .map_err(|e| ApplicationError::PriceFetch(e.to_string()))?;

        PriceSnapshot::new(payload.product_id, payload.observed_price, payload.observed_at)
            .map_err(|e| ApplicationError::PriceFetch(e.to_string()))
    }
}

#[derive(Clone)]
pub struct TelegramNotifier {
    client: Client,
    api_base: String,
    bot_token: String,
}

impl TelegramNotifier {
    pub fn new(api_base: impl Into<String>, bot_token: impl Into<String>) -> Self {
        Self {
            client: Client::builder().no_proxy().build().expect("client"),
            api_base: api_base.into(),
            bot_token: bot_token.into(),
        }
    }
}

#[async_trait]
impl NotifierPort for TelegramNotifier {
    async fn send(&self, chat_id: &str, message: &str) -> Result<(), ApplicationError> {
        self.client
            .post(format!("{}/bot{}/sendMessage", self.api_base, self.bot_token))
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "text": message,
            }))
            .send()
            .await
            .map_err(|e| ApplicationError::Notification(e.to_string()))?
            .error_for_status()
            .map_err(|e| ApplicationError::Notification(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn contract_price_ingest_maps_http_payload() {
        let server = MockServer::start().await;
        let now = Utc::now();
        Mock::given(method("GET"))
            .and(path("/prices/sku-1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "product_id": "sku-1",
                "observed_price": 42.5,
                "observed_at": now,
            })))
            .mount(&server)
            .await;

        let adapter = HttpPriceIngest::new(server.uri());
        let snapshot = adapter.fetch_latest("sku-1").await.unwrap();

        assert_eq!(snapshot.product_id, "sku-1");
        assert_eq!(snapshot.observed_price, 42.5);
    }

    #[tokio::test]
    async fn contract_telegram_notifier_sends_expected_payload() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/bottest-token/sendMessage"))
            .and(body_json(serde_json::json!({
                "chat_id": "chat-1",
                "text": "hello"
            })))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let notifier = TelegramNotifier::new(server.uri(), "test-token");
        notifier.send("chat-1", "hello").await.unwrap();
    }
}
