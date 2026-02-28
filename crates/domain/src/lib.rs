use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceSnapshot {
    pub product_id: String,
    pub observed_price: f64,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceAlert {
    pub product_id: String,
    pub threshold_price: f64,
    pub chat_id: String,
    pub message: String,
}

#[derive(Debug, Error, PartialEq)]
pub enum DomainError {
    #[error("product_id must not be blank")]
    EmptyProductId,
    #[error("chat_id must not be blank")]
    EmptyChatId,
    #[error("threshold_price must be > 0")]
    InvalidThreshold,
    #[error("observed_price must be >= 0")]
    InvalidObservedPrice,
}

impl PriceAlert {
    pub fn new(
        product_id: impl Into<String>,
        threshold_price: f64,
        chat_id: impl Into<String>,
    ) -> Result<Self, DomainError> {
        let product_id = product_id.into();
        let chat_id = chat_id.into();
        if product_id.trim().is_empty() {
            return Err(DomainError::EmptyProductId);
        }
        if chat_id.trim().is_empty() {
            return Err(DomainError::EmptyChatId);
        }
        if threshold_price <= 0.0 {
            return Err(DomainError::InvalidThreshold);
        }

        Ok(Self {
            message: format!("Price dropped below {threshold_price:.2} for product {product_id}"),
            product_id,
            threshold_price,
            chat_id,
        })
    }

    pub fn should_notify(&self, snapshot: &PriceSnapshot) -> bool {
        self.product_id == snapshot.product_id && snapshot.observed_price <= self.threshold_price
    }
}

impl PriceSnapshot {
    pub fn new(
        product_id: impl Into<String>,
        observed_price: f64,
        observed_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        let product_id = product_id.into();
        if product_id.trim().is_empty() {
            return Err(DomainError::EmptyProductId);
        }
        if observed_price < 0.0 {
            return Err(DomainError::InvalidObservedPrice);
        }

        Ok(Self {
            product_id,
            observed_price,
            observed_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_alert_rejects_empty_product_id() {
        let err = PriceAlert::new("", 10.0, "chat-1").unwrap_err();
        assert_eq!(err, DomainError::EmptyProductId);
    }

    #[test]
    fn price_alert_rejects_non_positive_threshold() {
        let err = PriceAlert::new("sku-1", 0.0, "chat-1").unwrap_err();
        assert_eq!(err, DomainError::InvalidThreshold);
    }

    #[test]
    fn price_snapshot_rejects_negative_price() {
        let err = PriceSnapshot::new("sku-1", -1.0, Utc::now()).unwrap_err();
        assert_eq!(err, DomainError::InvalidObservedPrice);
    }

    #[test]
    fn should_notify_when_price_drops_below_threshold() {
        let alert = PriceAlert::new("sku-1", 99.0, "chat-1").unwrap();
        let snapshot = PriceSnapshot::new("sku-1", 89.0, Utc::now()).unwrap();

        assert!(alert.should_notify(&snapshot));
    }

    #[test]
    fn should_not_notify_for_different_product() {
        let alert = PriceAlert::new("sku-1", 99.0, "chat-1").unwrap();
        let snapshot = PriceSnapshot::new("sku-2", 89.0, Utc::now()).unwrap();

        assert!(!alert.should_notify(&snapshot));
    }
}
