use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: String,
    pub exchange: String,
    pub base_asset: String,
    pub quote_asset: String,
}

#[derive(Debug, Clone)]
pub struct Watch {
    pub id: String,
    pub user_id: String,
    pub symbol_id: String,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub enum AlertCondition {
    PriceAbove,
    PriceBelow,
    PercentChangeUp,
    PercentChangeDown,
}

#[derive(Debug, Clone)]
pub struct AlertRule {
    pub id: String,
    pub watch_id: String,
    pub condition: AlertCondition,
    pub threshold: f64,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct PriceTick {
    pub symbol_id: String,
    pub price: f64,
    pub volume_24h: Option<f64>,
    pub exchange_ts_ms: u64,
}

#[derive(Debug, Clone)]
pub enum NotificationChannel {
    Telegram,
}

#[derive(Debug, Clone)]
pub enum NotificationStatus {
    Pending,
    Sent,
    Failed,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: String,
    pub user_id: String,
    pub channel: NotificationChannel,
    pub message: String,
    pub status: NotificationStatus,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub topic: String,
    pub key: String,
    pub payload: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum SymbolRepositoryError {
    #[error("symbol not found: {symbol_id}")]
    NotFound { symbol_id: String },
    #[error("symbol with id {symbol_id} already exists")]
    Conflict { symbol_id: String },
    #[error("symbol storage unavailable")]
    Unavailable,
}

#[derive(Debug, thiserror::Error)]
pub enum WatchRepositoryError {
    #[error("watch not found: {watch_id}")]
    NotFound { watch_id: String },
    #[error("watch with id {watch_id} already exists")]
    Conflict { watch_id: String },
    #[error("watch storage unavailable")]
    Unavailable,
}

#[derive(Debug, thiserror::Error)]
pub enum AlertRuleRepositoryError {
    #[error("alert rule not found: {rule_id}")]
    NotFound { rule_id: String },
    #[error("alert rule with id {rule_id} already exists")]
    Conflict { rule_id: String },
    #[error("alert rule storage unavailable")]
    Unavailable,
}

#[derive(Debug, thiserror::Error)]
pub enum PriceTickRepositoryError {
    #[error("price tick not found for symbol: {symbol_id}")]
    NotFound { symbol_id: String },
    #[error("price tick storage unavailable")]
    Unavailable,
}

#[derive(Debug, thiserror::Error)]
pub enum NotificationRepositoryError {
    #[error("notification not found: {notification_id}")]
    NotFound { notification_id: String },
    #[error("notification with id {notification_id} already exists")]
    Conflict { notification_id: String },
    #[error("notification storage unavailable")]
    Unavailable,
}

#[derive(Debug, thiserror::Error)]
pub enum MarketDataProviderError {
    #[error("symbol unsupported by provider: {symbol_id}")]
    UnsupportedSymbol { symbol_id: String },
    #[error("market data provider unavailable")]
    Unavailable,
    #[error("market data request timed out")]
    Timeout,
}

#[derive(Debug, thiserror::Error)]
pub enum TelegramNotifierError {
    #[error("invalid telegram chat id")]
    InvalidChatId,
    #[error("telegram bot unauthorized")]
    Unauthorized,
    #[error("telegram notifier unavailable")]
    Unavailable,
}

#[derive(Debug, thiserror::Error)]
pub enum EventPublisherError {
    #[error("event topic not found: {topic}")]
    TopicNotFound { topic: String },
    #[error("event publisher unavailable")]
    Unavailable,
}

#[async_trait]
pub trait SymbolRepository: Send + Sync {
    async fn get_by_id(&self, symbol_id: &str) -> Result<Symbol, SymbolRepositoryError>;
    async fn find_by_exchange_symbol(
        &self,
        exchange: &str,
        base_asset: &str,
        quote_asset: &str,
    ) -> Result<Option<Symbol>, SymbolRepositoryError>;
    async fn list_all(&self) -> Result<Vec<Symbol>, SymbolRepositoryError>;
    async fn save(&self, symbol: &Symbol) -> Result<(), SymbolRepositoryError>;
}

#[async_trait]
pub trait WatchRepository: Send + Sync {
    async fn get_by_id(&self, watch_id: &str) -> Result<Watch, WatchRepositoryError>;
    async fn list_by_user(&self, user_id: &str) -> Result<Vec<Watch>, WatchRepositoryError>;
    async fn list_active_by_symbol(
        &self,
        symbol_id: &str,
    ) -> Result<Vec<Watch>, WatchRepositoryError>;
    async fn save(&self, watch: &Watch) -> Result<(), WatchRepositoryError>;
    async fn delete(&self, watch_id: &str) -> Result<(), WatchRepositoryError>;
}

#[async_trait]
pub trait AlertRuleRepository: Send + Sync {
    async fn get_by_id(&self, rule_id: &str) -> Result<AlertRule, AlertRuleRepositoryError>;
    async fn list_enabled_by_watch(
        &self,
        watch_id: &str,
    ) -> Result<Vec<AlertRule>, AlertRuleRepositoryError>;
    async fn list_enabled_by_symbol(
        &self,
        symbol_id: &str,
    ) -> Result<Vec<AlertRule>, AlertRuleRepositoryError>;
    async fn save(&self, rule: &AlertRule) -> Result<(), AlertRuleRepositoryError>;
    async fn delete(&self, rule_id: &str) -> Result<(), AlertRuleRepositoryError>;
}

#[async_trait]
pub trait PriceTickRepository: Send + Sync {
    async fn get_latest_by_symbol(
        &self,
        symbol_id: &str,
    ) -> Result<Option<PriceTick>, PriceTickRepositoryError>;
    async fn save(&self, tick: &PriceTick) -> Result<(), PriceTickRepositoryError>;
}

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn get_by_id(
        &self,
        notification_id: &str,
    ) -> Result<Notification, NotificationRepositoryError>;
    async fn list_pending(
        &self,
        limit: usize,
    ) -> Result<Vec<Notification>, NotificationRepositoryError>;
    async fn save(&self, notification: &Notification) -> Result<(), NotificationRepositoryError>;
    async fn mark_sent(&self, notification_id: &str) -> Result<(), NotificationRepositoryError>;
    async fn mark_failed(
        &self,
        notification_id: &str,
        reason: &str,
    ) -> Result<(), NotificationRepositoryError>;
}

#[async_trait]
pub trait MarketDataProvider: Send + Sync {
    async fn get_latest_price(&self, symbol_id: &str) -> Result<PriceTick, MarketDataProviderError>;
}

#[async_trait]
pub trait TelegramNotifier: Send + Sync {
    async fn send_message(
        &self,
        chat_id: &str,
        message: &str,
    ) -> Result<(), TelegramNotifierError>;
}

#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: &Event) -> Result<(), EventPublisherError>;
}
