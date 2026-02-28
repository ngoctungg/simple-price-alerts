use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use serde::Serialize;
use tokio::time::sleep;

use crate::ports::{InfrastructureError, TelegramPort};

#[derive(Clone)]
pub struct TelegramBotAdapter {
    client: Client,
    token: String,
    max_retries: usize,
    base_delay: Duration,
}

impl TelegramBotAdapter {
    pub fn new(token: impl Into<String>, max_retries: usize, base_delay: Duration) -> Self {
        Self {
            client: Client::new(),
            token: token.into(),
            max_retries,
            base_delay,
        }
    }

    fn api_url(&self) -> String {
        format!("https://api.telegram.org/bot{}/sendMessage", self.token)
    }
}

#[derive(Serialize)]
struct SendMessage<'a> {
    chat_id: i64,
    text: &'a str,
}

#[async_trait]
impl TelegramPort for TelegramBotAdapter {
    async fn send_message(&self, chat_id: i64, text: &str) -> Result<(), InfrastructureError> {
        let mut attempt = 0;

        loop {
            let response = self
                .client
                .post(self.api_url())
                .json(&SendMessage { chat_id, text })
                .send()
                .await;

            match response {
                Ok(resp) if resp.status().is_success() => return Ok(()),
                Ok(resp) if attempt >= self.max_retries => {
                    return Err(InfrastructureError::Downstream(format!(
                        "telegram returned status {} after {} retries",
                        resp.status(),
                        attempt
                    )));
                }
                Err(err) if attempt >= self.max_retries => {
                    return Err(InfrastructureError::Http(format!(
                        "telegram request failed after {} retries: {}",
                        attempt, err
                    )));
                }
                _ => {
                    let backoff = self.base_delay.mul_f64(2_f64.powi(attempt as i32));
                    sleep(backoff).await;
                    attempt += 1;
                }
            }
        }
    }
}
