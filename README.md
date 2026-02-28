# simple-price-alerts

## Workspace structure

Project được tổ chức theo Cargo workspace, tách theo ngữ cảnh nghiệp vụ và vai trò triển khai:

- `apps/api`: REST/WebSocket entrypoint.
- `apps/worker`: consumer + scheduler + price monitor.
- `crates/domain`: Entities, Value Objects, Domain Services, Domain Events.
- `crates/application`: Use cases, ports/interfaces.
- `crates/infrastructure`: PostgreSQL, HTTP client, Telegram adapter, message broker.
- `crates/contracts`: DTO/API schema, error contract, serde models.
- `crates/shared`: time, config, tracing, common result/error.

## Dependency direction

Luồng phụ thuộc bắt buộc:

`domain <- application <- infrastructure/apps`

Ý nghĩa:

- `domain` là lõi nghiệp vụ, **không phụ thuộc framework** và không phụ thuộc các layer còn lại.
- `application` phụ thuộc `domain` để điều phối use case qua ports/interfaces.
- `infrastructure` phụ thuộc `application` (và có thể đọc `domain`/`contracts`/`shared`) để triển khai adapters kỹ thuật.
- `apps` (api/worker) là composition root, wiring các thành phần `application` + `infrastructure`.
Repository for a realtime Vietnamese stock price alert backend.

## Skills

This repository includes a project skill:

- `skills/rust-stock-alert-bootstrap/SKILL.md`

Check that required skills are present:

```bash
python3 scripts/ensure_skills.py
```
