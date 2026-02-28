# TDD Checklist

## Domain tests

- Value object validation and edge cases.
- Alert threshold math for increase/decrease conditions.
- Event emission on crossing threshold.

## Application tests

- Use-case success paths with fakes.
- Failure paths from repository/notifier/provider.
- Idempotent behavior for duplicate ticks.

## Infrastructure tests

- SQLx repositories against PostgreSQL test DB.
- Telegram adapter request payload formatting.
- Market API adapter deserialization and retry behavior.

## API tests

- Input validation errors.
- Status code and response contract.
- Pagination/filter behavior for watchlist.

## Worker tests

- Tick ingestion loop backpressure handling.
- End-to-end flow: tick -> trigger -> notification.
