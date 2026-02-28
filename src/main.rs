use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant, SystemTime},
};

use anyhow::Result;
use axum::{extract::State, response::IntoResponse, routing::get, Router};
use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use rand::Rng;
use reqwest::Client;
use serde::Serialize;
use tokio::{
    sync::{mpsc, Mutex},
    time::sleep,
};
use tracing::{error, info, instrument, warn};

#[derive(Debug, Clone)]
struct PriceTick {
    symbol: String,
    price: f64,
    queued_at: Instant,
}

#[derive(Debug, Clone)]
struct Alert {
    symbol: String,
    price: f64,
    threshold: f64,
}

#[derive(Clone)]
struct AppState {
    metrics: Metrics,
    external: ExternalClients,
}

#[derive(Clone)]
struct Metrics {
    prometheus: PrometheusHandle,
    alerts_last_minute: Arc<AtomicU64>,
}

#[derive(Clone)]
struct ExternalClients {
    provider: ResilientApi,
    telegram: ResilientApi,
}

#[derive(Clone)]
struct ResilientApi {
    name: &'static str,
    client: Client,
    endpoint: String,
    retry: RetryPolicy,
    breaker: CircuitBreaker,
}

#[derive(Clone, Copy)]
struct RetryPolicy {
    max_retries: usize,
    base_delay: Duration,
    max_jitter: Duration,
}

#[derive(Clone)]
struct CircuitBreaker {
    inner: Arc<Mutex<CircuitState>>,
    failure_threshold: u64,
    open_window: Duration,
    half_open_max_calls: u64,
}

#[derive(Debug)]
struct CircuitState {
    failures: u64,
    open_until: Option<Instant>,
    half_open_calls: u64,
    state: BreakerState,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum BreakerState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let metrics = init_metrics()?;
    spawn_alerts_per_minute_reset(metrics.alerts_last_minute.clone());

    let state = AppState {
        metrics: metrics.clone(),
        external: ExternalClients {
            provider: ResilientApi::new(
                "provider",
                "https://example.com/provider".to_owned(),
                RetryPolicy {
                    max_retries: 3,
                    base_delay: Duration::from_millis(120),
                    max_jitter: Duration::from_millis(200),
                },
                CircuitBreaker::new(5, Duration::from_secs(30), 2),
            ),
            telegram: ResilientApi::new(
                "telegram",
                "https://api.telegram.org/bot<TOKEN>/sendMessage".to_owned(),
                RetryPolicy {
                    max_retries: 4,
                    base_delay: Duration::from_millis(150),
                    max_jitter: Duration::from_millis(300),
                },
                CircuitBreaker::new(5, Duration::from_secs(45), 2),
            ),
        },
    };

    let (ingest_tx, ingest_rx) = mpsc::channel::<PriceTick>(1024);
    let (notify_tx, notify_rx) = mpsc::channel::<Alert>(1024);

    tokio::spawn(run_ingest_loop(ingest_tx));
    tokio::spawn(run_evaluate_loop(state.clone(), ingest_rx, notify_tx));
    tokio::spawn(run_notify_loop(state.clone(), notify_rx));

    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/healthz", get(health_handler))
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:3000".parse()?;
    info!(%addr, "service started");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,simple_price_alerts=debug".into()),
        )
        .json()
        .init();
}

fn init_metrics() -> Result<Metrics> {
    let recorder = PrometheusBuilder::new().install_recorder()?;
    Ok(Metrics {
        prometheus: recorder,
        alerts_last_minute: Arc::new(AtomicU64::new(0)),
    })
}

fn spawn_alerts_per_minute_reset(alerts_last_minute: Arc<AtomicU64>) {
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(60)).await;
            alerts_last_minute.store(0, Ordering::Relaxed);
        }
    });
}

#[instrument(skip(tx), name = "ingest_loop")]
async fn run_ingest_loop(tx: mpsc::Sender<PriceTick>) {
    let mut ticker = tokio::time::interval(Duration::from_secs(1));
    loop {
        ticker.tick().await;
        let started = Instant::now();
        let tick = PriceTick {
            symbol: "BTCUSDT".to_owned(),
            price: rand::thread_rng().gen_range(55_000.0..65_000.0),
            queued_at: Instant::now(),
        };

        if let Err(err) = tx.send(tick).await {
            error!(error = %err, "failed to enqueue price tick");
            counter!("ingest_errors_total").increment(1);
            break;
        }

        histogram!("ingest_latency_seconds").record(started.elapsed().as_secs_f64());
    }
}

#[instrument(skip(state, rx, notify_tx), name = "evaluate_loop")]
async fn run_evaluate_loop(
    state: AppState,
    mut rx: mpsc::Receiver<PriceTick>,
    notify_tx: mpsc::Sender<Alert>,
) {
    while let Some(tick) = rx.recv().await {
        let started = Instant::now();
        let queue_lag = tick.queued_at.elapsed().as_secs_f64();
        histogram!("queue_lag_seconds").record(queue_lag);

        if let Err(err) = state.external.provider.call("fetch-price-context").await {
            warn!(error = %err, "provider request failed during evaluate");
            counter!("provider_errors_total").increment(1);
        }

        let threshold = 60_500.0;
        if tick.price > threshold {
            let alert = Alert {
                symbol: tick.symbol,
                price: tick.price,
                threshold,
            };
            if notify_tx.send(alert).await.is_ok() {
                counter!("alerts_triggered_total").increment(1);
                let current = state
                    .metrics
                    .alerts_last_minute
                    .fetch_add(1, Ordering::Relaxed)
                    + 1;
                gauge!("alerts_triggered_per_minute").set(current as f64);
            }
        }

        histogram!("evaluate_latency_seconds").record(started.elapsed().as_secs_f64());
    }
}

#[instrument(skip(state, rx), name = "notify_loop")]
async fn run_notify_loop(state: AppState, mut rx: mpsc::Receiver<Alert>) {
    while let Some(alert) = rx.recv().await {
        let started = Instant::now();
        let message = format!(
            "{} crossed threshold {} with current {}",
            alert.symbol, alert.threshold, alert.price
        );

        if let Err(err) = state.external.telegram.call(&message).await {
            error!(error = %err, "telegram notification failed");
            counter!("telegram_errors_total").increment(1);
            counter!("notify_errors_total").increment(1);
        }

        histogram!("notify_latency_seconds").record(started.elapsed().as_secs_f64());
    }
}

async fn health_handler() -> impl IntoResponse {
    axum::Json(Health { status: "ok" })
}

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    state.metrics.prometheus.render()
}

impl ResilientApi {
    fn new(
        name: &'static str,
        endpoint: String,
        retry: RetryPolicy,
        breaker: CircuitBreaker,
    ) -> Self {
        Self {
            name,
            client: Client::new(),
            endpoint,
            retry,
            breaker,
        }
    }

    #[instrument(skip(self), fields(api = self.name))]
    async fn call(&self, payload: &str) -> Result<()> {
        self.breaker.allow_call().await?;

        let mut last_err: Option<anyhow::Error> = None;

        for attempt in 0..=self.retry.max_retries {
            let request_start = Instant::now();
            match self.try_call(payload).await {
                Ok(_) => {
                    self.breaker.record_success().await;
                    histogram!("external_api_latency_seconds", "api" => self.name)
                        .record(request_start.elapsed().as_secs_f64());
                    counter!("external_api_requests_total", "api" => self.name, "status" => "ok")
                        .increment(1);
                    return Ok(());
                }
                Err(err) => {
                    self.breaker.record_failure().await;
                    counter!("external_api_requests_total", "api" => self.name, "status" => "error").increment(1);
                    last_err = Some(err);
                    if attempt < self.retry.max_retries {
                        let jitter = rand::thread_rng()
                            .gen_range(0..=self.retry.max_jitter.as_millis() as u64);
                        let backoff = self.retry.base_delay * (attempt as u32 + 1);
                        sleep(backoff + Duration::from_millis(jitter)).await;
                        continue;
                    }
                }
            }
            break;
        }

        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("api call failed")))
    }

    async fn try_call(&self, payload: &str) -> Result<()> {
        let response = self
            .client
            .post(&self.endpoint)
            .header("x-request-ts", format!("{:?}", SystemTime::now()))
            .body(payload.to_owned())
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "request failed with status {}",
                response.status()
            ))
        }
    }
}

impl CircuitBreaker {
    fn new(failure_threshold: u64, open_window: Duration, half_open_max_calls: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(CircuitState {
                failures: 0,
                open_until: None,
                half_open_calls: 0,
                state: BreakerState::Closed,
            })),
            failure_threshold,
            open_window,
            half_open_max_calls,
        }
    }

    async fn allow_call(&self) -> Result<()> {
        let mut guard = self.inner.lock().await;
        match guard.state {
            BreakerState::Closed => Ok(()),
            BreakerState::Open => {
                if let Some(open_until) = guard.open_until {
                    if Instant::now() >= open_until {
                        guard.state = BreakerState::HalfOpen;
                        guard.half_open_calls = 1;
                        counter!("circuit_breaker_state_transitions_total", "state" => "half_open")
                            .increment(1);
                        Ok(())
                    } else {
                        counter!("circuit_breaker_rejections_total").increment(1);
                        Err(anyhow::anyhow!("circuit breaker is open"))
                    }
                } else {
                    Err(anyhow::anyhow!("circuit breaker has invalid open state"))
                }
            }
            BreakerState::HalfOpen => {
                if guard.half_open_calls < self.half_open_max_calls {
                    guard.half_open_calls += 1;
                    Ok(())
                } else {
                    counter!("circuit_breaker_rejections_total").increment(1);
                    Err(anyhow::anyhow!("circuit breaker half-open limit reached"))
                }
            }
        }
    }

    async fn record_success(&self) {
        let mut guard = self.inner.lock().await;
        if guard.state != BreakerState::Closed {
            counter!("circuit_breaker_state_transitions_total", "state" => "closed").increment(1);
        }
        guard.failures = 0;
        guard.open_until = None;
        guard.half_open_calls = 0;
        guard.state = BreakerState::Closed;
    }

    async fn record_failure(&self) {
        let mut guard = self.inner.lock().await;
        guard.failures += 1;

        match guard.state {
            BreakerState::HalfOpen => {
                guard.state = BreakerState::Open;
                guard.open_until = Some(Instant::now() + self.open_window);
                counter!("circuit_breaker_state_transitions_total", "state" => "open").increment(1);
            }
            BreakerState::Closed if guard.failures >= self.failure_threshold => {
                guard.state = BreakerState::Open;
                guard.open_until = Some(Instant::now() + self.open_window);
                counter!("circuit_breaker_state_transitions_total", "state" => "open").increment(1);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[tokio::test]
    async fn circuit_breaker_opens_after_threshold() {
        let cb = CircuitBreaker::new(2, Duration::from_secs(10), 1);

        cb.record_failure().await;
        assert!(cb.allow_call().await.is_ok());

        cb.record_failure().await;
        assert!(cb.allow_call().await.is_err());
    }

    #[tokio::test]
    async fn circuit_breaker_closes_after_success_in_half_open() {
        let cb = CircuitBreaker::new(1, Duration::from_millis(1), 1);

        cb.record_failure().await;
        sleep(Duration::from_millis(5)).await;
        assert!(cb.allow_call().await.is_ok());

        cb.record_success().await;
        assert!(cb.allow_call().await.is_ok());
    }

    #[test]
    fn retry_policy_is_configurable() {
        let retry = RetryPolicy {
            max_retries: 4,
            base_delay: Duration::from_millis(100),
            max_jitter: Duration::from_millis(250),
        };

        assert_eq!(retry.max_retries, 4);
        assert_eq!(retry.base_delay, Duration::from_millis(100));
        assert_eq!(retry.max_jitter, Duration::from_millis(250));
    }

    #[test]
    fn queue_lag_uses_tick_timestamp() {
        let tick = PriceTick {
            symbol: "ETHUSDT".into(),
            price: 3000.0,
            queued_at: Instant::now() - Duration::from_secs(2),
        };

        assert!(tick.queued_at.elapsed() >= Duration::from_secs(2));
    }

    #[test]
    fn alerts_per_minute_can_be_reset() {
        let value = Arc::new(AtomicU64::new(10));
        value.store(0, Ordering::Relaxed);
        assert_eq!(value.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn rolling_buffer_example_for_future_queue_lag_tracking() {
        let mut queue_lags = VecDeque::new();
        queue_lags.push_back(0.4_f64);
        queue_lags.push_back(0.9_f64);

        assert_eq!(queue_lags.len(), 2);
    }
}
