use crate::{ChatId, DomainError, Percentage, Price, SymbolCode, Timestamp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StockSymbol {
    code: SymbolCode,
    listed: bool,
}

impl StockSymbol {
    pub fn new(code: SymbolCode) -> Self {
        Self { code, listed: true }
    }

    pub fn code(&self) -> &SymbolCode {
        &self.code
    }

    pub fn delist(&mut self) {
        self.listed = false;
    }

    pub fn relist(&mut self) {
        self.listed = true;
    }

    pub fn is_listed(&self) -> bool {
        self.listed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchItem {
    user_id: u64,
    symbol: SymbolCode,
    active: bool,
}

impl WatchItem {
    pub fn new(user_id: u64, symbol: SymbolCode) -> Result<Self, DomainError> {
        if user_id == 0 {
            return Err(DomainError::InvalidUserId(user_id));
        }

        Ok(Self {
            user_id,
            symbol,
            active: true,
        })
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn user_id(&self) -> u64 {
        self.user_id
    }

    pub fn symbol(&self) -> &SymbolCode {
        &self.symbol
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AlertThreshold {
    Percent(Percentage),
    Absolute(Price),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AlertRule {
    threshold: AlertThreshold,
    time_window_secs: u64,
}

impl AlertRule {
    pub fn new(threshold: AlertThreshold, time_window_secs: u64) -> Result<Self, DomainError> {
        if time_window_secs == 0 {
            return Err(DomainError::InvalidTimeWindowSecs(time_window_secs));
        }

        Ok(Self {
            threshold,
            time_window_secs,
        })
    }

    pub fn threshold(&self) -> AlertThreshold {
        self.threshold
    }

    pub fn time_window_secs(&self) -> u64 {
        self.time_window_secs
    }

    pub fn is_triggered(&self, previous: PriceTick, current: PriceTick) -> Result<bool, DomainError> {
        if previous.symbol != current.symbol {
            return Err(DomainError::RuleMismatch);
        }

        let elapsed_secs = current
            .timestamp
            .elapsed_secs_since(previous.timestamp)
            .ok_or(DomainError::RuleMismatch)?;

        if elapsed_secs > self.time_window_secs {
            return Ok(false);
        }

        let moved = match self.threshold {
            AlertThreshold::Percent(min_change) => {
                current.price.percent_change_from(previous.price)? >= min_change.value()
            }
            AlertThreshold::Absolute(min_change) => {
                current.price.absolute_difference(previous.price) >= min_change.value()
            }
        };

        Ok(moved)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PriceTick {
    symbol: SymbolCode,
    price: Price,
    volume: u64,
    timestamp: Timestamp,
}

impl PriceTick {
    pub fn new(
        symbol: SymbolCode,
        price: Price,
        volume: u64,
        timestamp: Timestamp,
    ) -> Result<Self, DomainError> {
        if volume == 0 {
            return Err(DomainError::InvalidVolume(volume));
        }

        Ok(Self {
            symbol,
            price,
            volume,
            timestamp,
        })
    }

    pub fn symbol(&self) -> &SymbolCode {
        &self.symbol
    }

    pub fn price(&self) -> Price {
        self.price
    }

    pub fn volume(&self) -> u64 {
        self.volume
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NotificationStatus {
    Pending,
    Sent,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    chat_id: ChatId,
    message_template: String,
    status: NotificationStatus,
}

impl Notification {
    pub fn new(chat_id: ChatId, message_template: impl Into<String>) -> Result<Self, DomainError> {
        let message_template = message_template.into().trim().to_string();
        if message_template.is_empty() {
            return Err(DomainError::EmptyMessageTemplate);
        }

        Ok(Self {
            chat_id,
            message_template,
            status: NotificationStatus::Pending,
        })
    }

    pub fn mark_sent(&mut self) -> Result<(), DomainError> {
        match self.status {
            NotificationStatus::Pending | NotificationStatus::Failed => {
                self.status = NotificationStatus::Sent;
                Ok(())
            }
            NotificationStatus::Sent => Err(DomainError::InvalidNotificationState),
        }
    }

    pub fn mark_failed(&mut self) -> Result<(), DomainError> {
        match self.status {
            NotificationStatus::Pending => {
                self.status = NotificationStatus::Failed;
                Ok(())
            }
            NotificationStatus::Sent | NotificationStatus::Failed => {
                Err(DomainError::InvalidNotificationState)
            }
        }
    }

    pub fn render_message(&self, symbol: &SymbolCode, price: Price) -> String {
        self.message_template
            .replace("{symbol}", symbol.as_str())
            .replace("{price}", &format!("{:.2}", price.value()))
    }

    pub fn chat_id(&self) -> ChatId {
        self.chat_id
    }

    pub fn status(&self) -> NotificationStatus {
        self.status
    }

    pub fn message_template(&self) -> &str {
        &self.message_template
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn symbol() -> SymbolCode {
        SymbolCode::new("VCB").unwrap()
    }

    #[test]
    fn symbol_code_is_uppercased_and_validated() {
        let code = SymbolCode::new("fpt").unwrap();
        assert_eq!(code.as_str(), "FPT");
        assert!(SymbolCode::new("$bad").is_err());
    }

    #[test]
    fn alert_rule_percent_threshold_triggers_within_window() {
        let rule = AlertRule::new(
            AlertThreshold::Percent(Percentage::new(3.0).unwrap()),
            60,
        )
        .unwrap();

        let old_tick = PriceTick::new(
            symbol(),
            Price::new(100.0).unwrap(),
            10,
            Timestamp::new(1_000).unwrap(),
        )
        .unwrap();

        let new_tick = PriceTick::new(
            symbol(),
            Price::new(103.5).unwrap(),
            12,
            Timestamp::new(30_000).unwrap(),
        )
        .unwrap();

        assert!(rule.is_triggered(old_tick, new_tick).unwrap());
    }

    #[test]
    fn notification_state_flow_is_enforced() {
        let chat_id = ChatId::new(123).unwrap();
        let mut notification = Notification::new(chat_id, "{symbol} crossed {price}").unwrap();
        notification.mark_failed().unwrap();
        notification.mark_sent().unwrap();
        assert_eq!(notification.status(), NotificationStatus::Sent);
        assert!(notification.mark_failed().is_err());
    }
}
