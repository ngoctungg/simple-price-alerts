use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::ports::{AlertDirection, AlertNotification, PriceAlert};

#[derive(Debug, FromRow)]
pub struct PriceAlertRow {
    pub id: Uuid,
    pub user_id: i64,
    pub symbol: String,
    pub target_price: f64,
    pub direction: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct NotificationRow {
    pub id: Uuid,
    pub alert_id: Uuid,
    pub delivered_at: Option<DateTime<Utc>>,
}

pub fn to_alert_row(domain: &PriceAlert) -> PriceAlertRow {
    PriceAlertRow {
        id: domain.id,
        user_id: domain.user_id,
        symbol: domain.symbol.clone(),
        target_price: domain.target_price,
        direction: match domain.direction {
            AlertDirection::Above => "above",
            AlertDirection::Below => "below",
        }
        .to_owned(),
        created_at: domain.created_at,
    }
}

pub fn to_alert_domain(row: PriceAlertRow) -> PriceAlert {
    PriceAlert {
        id: row.id,
        user_id: row.user_id,
        symbol: row.symbol,
        target_price: row.target_price,
        direction: match row.direction.as_str() {
            "below" => AlertDirection::Below,
            _ => AlertDirection::Above,
        },
        created_at: row.created_at,
    }
}

pub fn to_notification_row(domain: &AlertNotification) -> NotificationRow {
    NotificationRow {
        id: domain.id,
        alert_id: domain.alert_id,
        delivered_at: domain.delivered_at,
    }
}
