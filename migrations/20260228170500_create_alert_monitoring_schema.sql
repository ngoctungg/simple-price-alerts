-- Add monitoring schema for price alerts.

CREATE TABLE IF NOT EXISTS symbols (
    id BIGSERIAL PRIMARY KEY,
    symbol TEXT NOT NULL UNIQUE,
    base_asset TEXT,
    quote_asset TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS watched_symbols (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    symbol TEXT NOT NULL REFERENCES symbols(symbol) ON UPDATE CASCADE ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, symbol)
);

CREATE TABLE IF NOT EXISTS alert_rules (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    symbol TEXT NOT NULL REFERENCES symbols(symbol) ON UPDATE CASCADE ON DELETE RESTRICT,
    rule_type TEXT NOT NULL,
    threshold_value NUMERIC(24, 10) NOT NULL,
    comparison_operator TEXT NOT NULL,
    rule_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_triggered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Partitioned table for high-volume tick data.
CREATE TABLE IF NOT EXISTS price_ticks (
    id BIGSERIAL,
    symbol TEXT NOT NULL REFERENCES symbols(symbol) ON UPDATE CASCADE ON DELETE RESTRICT,
    price NUMERIC(24, 10) NOT NULL,
    ts TIMESTAMPTZ NOT NULL,
    source TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, ts)
) PARTITION BY RANGE (ts);

-- Default partition fallback.
CREATE TABLE IF NOT EXISTS price_ticks_default PARTITION OF price_ticks DEFAULT;

-- Example monthly partition (current month, helpful as template for rotation jobs).
CREATE TABLE IF NOT EXISTS price_ticks_2026_02 PARTITION OF price_ticks
FOR VALUES FROM ('2026-02-01 00:00:00+00') TO ('2026-03-01 00:00:00+00');

CREATE TABLE IF NOT EXISTS notifications (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    alert_rule_id BIGINT REFERENCES alert_rules(id) ON UPDATE CASCADE ON DELETE SET NULL,
    symbol TEXT NOT NULL REFERENCES symbols(symbol) ON UPDATE CASCADE ON DELETE RESTRICT,
    channel TEXT NOT NULL,
    payload JSONB,
    send_status TEXT NOT NULL DEFAULT 'pending', -- pending, sent, failed
    sent_at TIMESTAMPTZ,
    failure_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS outbox_events (
    id BIGSERIAL PRIMARY KEY,
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    published BOOLEAN NOT NULL DEFAULT FALSE,
    published_at TIMESTAMPTZ,
    retry_count INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes requested for monitor query optimization.
CREATE INDEX IF NOT EXISTS idx_price_ticks_symbol_ts_desc
    ON price_ticks (symbol, ts DESC);

CREATE INDEX IF NOT EXISTS idx_watched_symbols_user_symbol
    ON watched_symbols (user_id, symbol);

CREATE INDEX IF NOT EXISTS idx_alert_rules_active_symbol
    ON alert_rules (rule_active, symbol);

-- Helpful supporting indexes.
CREATE INDEX IF NOT EXISTS idx_notifications_status_created_at
    ON notifications (send_status, created_at);

CREATE INDEX IF NOT EXISTS idx_outbox_events_unpublished_created_at
    ON outbox_events (published, created_at);
