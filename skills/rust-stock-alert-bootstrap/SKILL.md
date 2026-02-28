---
name: rust-stock-alert-bootstrap
description: Bootstrap and evolve a Rust backend for realtime Vietnamese stock price alerts with Clean Architecture, DDD, TDD, PostgreSQL persistence, and Telegram notifications. Use when creating modules, domain/use-case boundaries, ports/adapters, migrations, and test strategy for this project.
---

# Rust Stock Alert Bootstrap

Use this skill to implement and maintain the `simple-price-alerts` backend with strict decoupling.

## Follow architecture boundaries

- Keep dependencies one-way: `domain <- application <- infrastructure <- apps`.
- Keep `domain` free from framework/IO/database concerns.
- Define use-cases in `application` and depend on traits (ports).
- Implement all external integration in `infrastructure` adapters.

## Create workspace skeleton

Create this workspace structure when missing:

- `apps/api` for HTTP APIs.
- `apps/worker` for realtime ingestion and alert processing.
- `crates/domain` for entities, value objects, domain services, domain events.
- `crates/application` for use-cases and ports.
- `crates/infrastructure` for Postgres repositories, HTTP market adapter, Telegram adapter.
- `crates/contracts` for DTOs and boundary contracts.
- `crates/shared` for config, tracing, common errors/utilities.

## Implement feature flow

Implement features in this sequence:

1. Search stock symbol from market API.
2. Select symbols to watch.
3. Persist watched symbols and alert rules in PostgreSQL.
4. Ingest realtime price ticks.
5. Evaluate price movement thresholds.
6. Send Telegram notifications when threshold is reached.

## Apply TDD per layer

- Write domain tests first for invariants and threshold calculations.
- Add application tests with fake ports for use-case orchestration.
- Add infrastructure integration tests with PostgreSQL for repository contracts.
- Add API tests for handlers and validation boundaries.

## Use bundled reference

Read `references/roadmap.md` for an actionable implementation plan and `references/testing-checklist.md` for TDD gates.
