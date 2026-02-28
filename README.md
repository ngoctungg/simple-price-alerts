# simple-price-alerts

Service mẫu bằng Rust cho pipeline ingest → evaluate → notify.

## Tính năng observability

- `tracing` + `tracing-subscriber` (JSON logs).
- Metrics Prometheus tại `GET /metrics`:
  - `ingest_latency_seconds`
  - `evaluate_latency_seconds`
  - `notify_latency_seconds`
  - `alerts_triggered_total`
  - `alerts_triggered_per_minute`
  - `provider_errors_total`
  - `telegram_errors_total`
  - `queue_lag_seconds`
  - Các metrics cho retry/circuit-breaker external API.

## Resilience

- Retry có exponential-ish backoff theo attempt + jitter ngẫu nhiên.
- Circuit breaker cho provider và Telegram API.

## Chạy local

```bash
cargo run
```

Server listen tại `0.0.0.0:3000`.
