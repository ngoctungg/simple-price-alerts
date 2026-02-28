use application::{AlertRepositoryPort, ApplicationError};
use async_trait::async_trait;
use domain::PriceAlert;
use sqlx::PgPool;

pub struct PostgresAlertRepository {
    pool: PgPool,
}

impl PostgresAlertRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AlertRepositoryPort for PostgresAlertRepository {
    async fn find_for_product(&self, product_id: &str) -> Result<Vec<PriceAlert>, ApplicationError> {
        let records = sqlx::query_as::<_, (String, f64, String)>(
            "SELECT product_id, threshold_price, chat_id FROM alerts WHERE product_id = $1",
        )
        .bind(product_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        records
            .into_iter()
            .map(|(product_id, threshold_price, chat_id)| {
                PriceAlert::new(product_id, threshold_price, chat_id)
                    .map_err(|e| ApplicationError::Repository(e.to_string()))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use testcontainers_modules::{postgres::Postgres, testcontainers::runners::AsyncRunner};

    #[tokio::test]
    async fn integration_find_alerts_in_postgres_container() {
        let container = match Postgres::default().start().await {
            Ok(container) => container,
            Err(err) => {
                eprintln!("skipping test, docker not available: {err}");
                return;
            }
        };
        let port = container.get_host_port_ipv4(5432).await.unwrap();
        let connection_string = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");

        let pool = PgPool::connect(&connection_string).await.unwrap();
        sqlx::query(
            "CREATE TABLE alerts (product_id TEXT NOT NULL, threshold_price DOUBLE PRECISION NOT NULL, chat_id TEXT NOT NULL)",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO alerts (product_id, threshold_price, chat_id) VALUES ($1, $2, $3), ($4, $5, $6)",
        )
        .bind("sku-1")
        .bind(100.0_f64)
        .bind("chat-1")
        .bind("sku-2")
        .bind(80.0_f64)
        .bind("chat-2")
        .execute(&pool)
        .await
        .unwrap();

        let repo = PostgresAlertRepository::new(pool);
        let alerts = repo.find_for_product("sku-1").await.unwrap();

        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].chat_id, "chat-1");
    }
}
