use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use futures::Stream;
use reqwest::Client;
use serde::Deserialize;
use tokio::time::{interval, Interval};

use crate::ports::{InfrastructureError, StockLookupPort, StockQuote};

#[derive(Clone)]
pub struct VnStockApiClient {
    client: Client,
    base_url: String,
}

impl VnStockApiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
        }
    }

    pub fn stream_quotes(&self, symbol: impl Into<String>, poll_every: Duration) -> VnStockStream {
        VnStockStream {
            interval: interval(poll_every),
            symbol: symbol.into(),
            client: self.clone(),
            in_flight: None,
        }
    }
}

#[derive(Deserialize)]
struct SymbolResponse {
    symbols: Vec<String>,
}

#[derive(Deserialize)]
struct QuoteResponse {
    symbol: String,
    price: f64,
}

#[async_trait]
impl StockLookupPort for VnStockApiClient {
    async fn search_symbols(&self, query: &str) -> Result<Vec<String>, InfrastructureError> {
        let endpoint = format!("{}/symbols", self.base_url);
        self.client
            .get(endpoint)
            .query(&[("q", query)])
            .send()
            .await
            .map_err(|err| InfrastructureError::Http(err.to_string()))?
            .json::<SymbolResponse>()
            .await
            .map(|response| response.symbols)
            .map_err(|err| InfrastructureError::Serialization(err.to_string()))
    }

    async fn get_quote(&self, symbol: &str) -> Result<StockQuote, InfrastructureError> {
        let endpoint = format!("{}/quotes/{symbol}", self.base_url);
        let payload = self
            .client
            .get(endpoint)
            .send()
            .await
            .map_err(|err| InfrastructureError::Http(err.to_string()))?
            .json::<QuoteResponse>()
            .await
            .map_err(|err| InfrastructureError::Serialization(err.to_string()))?;

        Ok(StockQuote {
            symbol: payload.symbol,
            price: payload.price,
            updated_at: Utc::now(),
        })
    }
}

pub struct VnStockStream {
    interval: Interval,
    symbol: String,
    client: VnStockApiClient,
    in_flight: Option<
        Pin<Box<dyn futures::Future<Output = Result<StockQuote, InfrastructureError>> + Send>>,
    >,
}

impl Stream for VnStockStream {
    type Item = Result<StockQuote, InfrastructureError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(fut) = self.in_flight.as_mut() {
            if let Poll::Ready(result) = fut.as_mut().poll(cx) {
                self.in_flight = None;
                return Poll::Ready(Some(result));
            }
            return Poll::Pending;
        }

        if Pin::new(&mut self.interval).poll_tick(cx).is_ready() {
            let symbol = self.symbol.clone();
            let client = self.client.clone();
            self.in_flight = Some(Box::pin(async move {
                StockLookupPort::get_quote(&client, &symbol).await
            }));
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        Poll::Pending
    }
}
