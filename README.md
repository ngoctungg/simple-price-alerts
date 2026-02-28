# simple-price-alerts

Workspace mô phỏng pipeline cảnh báo giá theo vòng lặp **Red-Green-Refactor** với nhiều tầng test:

- `domain`: unit test cho invariants.
- `application`: test use case với fake ports + smoke e2e ingest -> notify.
- `adapters`: contract test cho HTTP adapters (API giá + Telegram).
- `repositories`: integration test với Postgres testcontainer.

## Chạy test theo tầng

```bash
cargo test -p domain
cargo test -p application
cargo test -p adapters
cargo test -p repositories integration_find_alerts_in_postgres_container
```

## CI

CI được tách thành 2 jobs:

1. `unit-and-contract`: chạy unit/application/contract tests trước.
2. `integration`: chỉ chạy sau khi job đầu thành công.
