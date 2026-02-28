use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::ports::InfrastructureError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEvent {
    pub id: Uuid,
    pub aggregate_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct OutboxRow {
    id: Uuid,
    aggregate_id: Uuid,
    event_type: String,
    payload: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl From<OutboxRow> for OutboxEvent {
    fn from(row: OutboxRow) -> Self {
        Self {
            id: row.id,
            aggregate_id: row.aggregate_id,
            event_type: row.event_type,
            payload: row.payload,
            created_at: row.created_at,
        }
    }
}

#[async_trait]
pub trait PublishTransport: Send + Sync {
    async fn publish(&self, event: &OutboxEvent) -> Result<(), InfrastructureError>;
}

pub struct OutboxPublisher<T> {
    pool: PgPool,
    transport: T,
}

impl<T> OutboxPublisher<T>
where
    T: PublishTransport,
{
    pub fn new(pool: PgPool, transport: T) -> Self {
        Self { pool, transport }
    }

    pub async fn enqueue(&self, event: &OutboxEvent) -> Result<(), InfrastructureError> {
        sqlx::query(
            r#"
            insert into outbox_events (id, aggregate_id, event_type, payload, created_at)
            values ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(event.id)
        .bind(event.aggregate_id)
        .bind(&event.event_type)
        .bind(&event.payload)
        .bind(event.created_at)
        .execute(&self.pool)
        .await
        .map_err(|err| InfrastructureError::Database(err.to_string()))?;

        Ok(())
    }

    pub async fn publish_pending(&self, batch_size: i64) -> Result<usize, InfrastructureError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| InfrastructureError::Database(err.to_string()))?;

        let rows = sqlx::query_as::<_, OutboxRow>(
            r#"
            select id, aggregate_id, event_type, payload, created_at
            from outbox_events
            where published_at is null
            order by created_at
            for update skip locked
            limit $1
            "#,
        )
        .bind(batch_size)
        .fetch_all(&mut *tx)
        .await
        .map_err(|err| InfrastructureError::Database(err.to_string()))?;

        let mut published = 0usize;
        for event in rows.into_iter().map(OutboxEvent::from) {
            self.transport.publish(&event).await?;
            sqlx::query("update outbox_events set published_at = now() where id = $1")
                .bind(event.id)
                .execute(&mut *tx)
                .await
                .map_err(|err| InfrastructureError::Database(err.to_string()))?;
            published += 1;
        }

        tx.commit()
            .await
            .map_err(|err| InfrastructureError::Database(err.to_string()))?;

        Ok(published)
    }
}
