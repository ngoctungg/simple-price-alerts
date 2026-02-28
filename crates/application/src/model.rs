#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub code: String,
    pub display_name: String,
    pub exchange: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchedSymbol {
    pub user_id: String,
    pub symbol: Symbol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertDirection {
    PriceAbove,
    PriceBelow,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRule {
    pub id: String,
    pub user_id: String,
    pub symbol_code: String,
    pub direction: AlertDirection,
    pub threshold_price: f64,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PriceTick {
    pub symbol_code: String,
    pub price: f64,
    pub timestamp_ms: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TriggeredAlert {
    pub rule_id: String,
    pub user_id: String,
    pub symbol_code: String,
    pub current_price: f64,
    pub threshold_price: f64,
    pub direction: AlertDirection,
    pub triggered_at_ms: i64,
}
