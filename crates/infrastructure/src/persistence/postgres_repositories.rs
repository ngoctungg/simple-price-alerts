use async_trait::async_trait;
use sqlx::PgPool;

use crate::persistence::mapper::{
    to_alert_domain, to_alert_row, to_notification_row, PriceAlertRow,
};
use crate::ports::{
    AlertNotification, InfrastructureError, NotificationRepository, PriceAlert,
    PriceAlertRepository,
};

pub struct PostgresPriceAlertRepository {
    pool: PgPool,
}

impl PostgresPriceAlertRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PriceAlertRepository for PostgresPriceAlertRepository {
    async fn save_alert(&self, alert: &PriceAlert) -> Result<(), InfrastructureError> {
        let row = to_alert_row(alert);
        sqlx::query(
            r#"
            insert into price_alerts (id, user_id, symbol, target_price, direction, created_at)
            values ($1, $2, $3, $4, $5, $6)
            on conflict (id) do update
              set target_price = excluded.target_price,
                  direction = excluded.direction
            "#,
        )
        .bind(row.id)
        .bind(row.user_id)
        .bind(row.symbol)
        .bind(row.target_price)
        .bind(row.direction)
        .bind(row.created_at)
        .execute(&self.pool)
        .await
        .map_err(|err| InfrastructureError::Database(err.to_string()))?;

        Ok(())
    }

    async fn find_by_symbol(&self, symbol: &str) -> Result<Vec<PriceAlert>, InfrastructureError> {
        let rows = sqlx::query_as::<_, PriceAlertRow>(
            r#"
            select id, user_id, symbol, target_price, direction, created_at
            from price_alerts
            where symbol = $1
            "#,
        )
        .bind(symbol)
        .fetch_all(&self.pool)
        .await
        .map_err(|err| InfrastructureError::Database(err.to_string()))?;

        Ok(rows.into_iter().map(to_alert_domain).collect())
    }
}

pub struct PostgresNotificationRepository {
    pool: PgPool,
}

impl PostgresNotificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NotificationRepository for PostgresNotificationRepository {
    async fn mark_delivered(
        &self,
        notification: &AlertNotification,
    ) -> Result<(), InfrastructureError> {
        let row = to_notification_row(notification);
        sqlx::query(
            r#"
            update alert_notifications
            set delivered_at = coalesce($2, now())
            where id = $1 and alert_id = $3
            "#,
        )
        .bind(row.id)
        .bind(row.delivered_at)
        .bind(row.alert_id)
        .execute(&self.pool)
        .await
        .map_err(|err| InfrastructureError::Database(err.to_string()))?;

        Ok(())
    }
}
