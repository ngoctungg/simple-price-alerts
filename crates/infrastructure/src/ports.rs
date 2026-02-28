use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PriceAlert {
    pub id: Uuid,
    pub user_id: i64,
    pub symbol: String,
    pub target_price: f64,
    pub direction: AlertDirection,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum AlertDirection {
    Above,
    Below,
}

#[derive(Debug, Clone)]
pub struct AlertNotification {
    pub id: Uuid,
    pub alert_id: Uuid,
    pub delivered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockQuote {
    pub symbol: String,
    pub price: f64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum InfrastructureError {
    #[error("database error: {0}")]
    Database(String),
    #[error("http error: {0}")]
    Http(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("downstream error: {0}")]
    Downstream(String),
}

use serde::{Deserialize, Serialize};

#[async_trait]
pub trait PriceAlertRepository: Send + Sync {
    async fn save_alert(&self, alert: &PriceAlert) -> Result<(), InfrastructureError>;
    async fn find_by_symbol(&self, symbol: &str) -> Result<Vec<PriceAlert>, InfrastructureError>;
}

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn mark_delivered(
        &self,
        notification: &AlertNotification,
    ) -> Result<(), InfrastructureError>;
}

#[async_trait]
pub trait StockLookupPort: Send + Sync {
    async fn search_symbols(&self, query: &str) -> Result<Vec<String>, InfrastructureError>;
    async fn get_quote(&self, symbol: &str) -> Result<StockQuote, InfrastructureError>;
}

#[async_trait]
pub trait TelegramPort: Send + Sync {
    async fn send_message(&self, chat_id: i64, text: &str) -> Result<(), InfrastructureError>;
}
