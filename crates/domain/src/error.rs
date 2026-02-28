use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum DomainError {
    InvalidSymbolCode(String),
    InvalidPrice(f64),
    InvalidPercentage(f64),
    InvalidChatId(i64),
    InvalidTimestamp(i64),
    InvalidUserId(u64),
    InvalidVolume(u64),
    InvalidTimeWindowSecs(u64),
    RuleMismatch,
    InvalidNotificationState,
    EmptyMessageTemplate,
}

impl Display for DomainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainError::InvalidSymbolCode(code) => write!(f, "invalid VN stock symbol code: {code}"),
            DomainError::InvalidPrice(price) => write!(f, "invalid price: {price}"),
            DomainError::InvalidPercentage(percent) => write!(f, "invalid percentage: {percent}"),
            DomainError::InvalidChatId(chat_id) => write!(f, "invalid telegram chat id: {chat_id}"),
            DomainError::InvalidTimestamp(ts) => write!(f, "invalid timestamp (ms): {ts}"),
            DomainError::InvalidUserId(user_id) => write!(f, "invalid user id: {user_id}"),
            DomainError::InvalidVolume(volume) => write!(f, "invalid volume: {volume}"),
            DomainError::InvalidTimeWindowSecs(secs) => write!(f, "invalid time window secs: {secs}"),
            DomainError::RuleMismatch => write!(f, "alert rule type mismatch"),
            DomainError::InvalidNotificationState => write!(f, "invalid notification state transition"),
            DomainError::EmptyMessageTemplate => write!(f, "message template cannot be empty"),
        }
    }
}

impl std::error::Error for DomainError {}
