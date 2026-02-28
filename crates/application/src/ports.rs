use crate::model::{AlertRule, PriceTick, Symbol, TriggeredAlert, WatchedSymbol};

pub trait SymbolSearchPort {
    type Error;

    fn search_symbols(&self, query: &str, limit: usize) -> Result<Vec<Symbol>, Self::Error>;

    fn find_symbol_by_code(&self, symbol_code: &str) -> Result<Option<Symbol>, Self::Error>;
}

pub trait WatchlistPort {
    type Error;

    fn upsert_watched_symbol(&self, watched: WatchedSymbol) -> Result<WatchedSymbol, Self::Error>;

    fn list_watched_symbols(&self, user_id: &str) -> Result<Vec<WatchedSymbol>, Self::Error>;
}

pub trait AlertRulePort {
    type Error;

    fn upsert_alert_rule(&self, rule: AlertRule) -> Result<AlertRule, Self::Error>;

    fn list_enabled_rules_by_symbol(
        &self,
        symbol_code: &str,
    ) -> Result<Vec<AlertRule>, Self::Error>;
}

pub trait TriggeredAlertPort {
    type Error;

    fn save_triggered_alert(&self, alert: TriggeredAlert) -> Result<TriggeredAlert, Self::Error>;
}

pub trait TelegramPort {
    type Error;

    fn send_message(&self, user_id: &str, message: &str) -> Result<(), Self::Error>;
}

pub trait PriceTickSinkPort {
    type Error;

    fn append_tick(&self, tick: PriceTick) -> Result<(), Self::Error>;
}
