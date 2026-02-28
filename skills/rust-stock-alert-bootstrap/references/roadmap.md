# Realtime Stock Alert Roadmap (DDD + Clean Architecture)

## Phase 1: Foundations

- Initialize Cargo workspace.
- Configure lint/format (`clippy`, `rustfmt`).
- Add shared config loading and structured logging (`tracing`).

## Phase 2: Domain

- Model aggregates: `StockSymbol`, `WatchItem`, `AlertRule`, `PriceTick`, `Notification`.
- Model value objects: `SymbolCode`, `Price`, `Percentage`, `ChatId`, `Timestamp`.
- Add domain events: `PriceTickReceived`, `AlertTriggered`, `NotificationDispatched`.

## Phase 3: Application

- Define ports: repositories, market data provider, notifier, event bus.
- Implement use-cases:
  - `search_symbol`
  - `add_watch_item`
  - `update_alert_rule`
  - `process_price_tick`
  - `dispatch_notification`

## Phase 4: Infrastructure

- Create PostgreSQL schema and migrations.
- Implement SQLx repositories.
- Implement market API adapter (polling/WebSocket depending on provider).
- Implement Telegram Bot API adapter with retry and idempotency safeguards.

## Phase 5: Apps

- `apps/api`: REST endpoints for symbol search/watchlist/rules.
- `apps/worker`: realtime ingestion and alert engine.

## Phase 6: Reliability

- Add outbox or dedup key for notification idempotency.
- Add metrics: tick throughput, trigger rate, notification success/fail.
- Add health/readiness endpoints.

## Phase 7: Delivery

- CI pipeline: format, lint, unit tests, integration tests, build.
- Containerization and env-based configuration.
