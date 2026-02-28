use crate::error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SymbolCode(String);

impl SymbolCode {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let code = value.into().trim().to_uppercase();
        let valid_len = (3..=5).contains(&code.len());
        let valid_charset = code.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit());

        if valid_len && valid_charset {
            Ok(Self(code))
        } else {
            Err(DomainError::InvalidSymbolCode(code))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Price(f64);

impl Price {
    pub fn new(value: f64) -> Result<Self, DomainError> {
        if value.is_finite() && value > 0.0 {
            Ok(Self(value))
        } else {
            Err(DomainError::InvalidPrice(value))
        }
    }

    pub fn value(self) -> f64 {
        self.0
    }

    pub fn absolute_difference(self, other: Price) -> f64 {
        (self.0 - other.0).abs()
    }

    pub fn percent_change_from(self, previous: Price) -> Result<f64, DomainError> {
        if previous.0 <= 0.0 {
            return Err(DomainError::InvalidPrice(previous.0));
        }
        Ok(((self.0 - previous.0) / previous.0).abs() * 100.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Percentage(f64);

impl Percentage {
    pub fn new(value: f64) -> Result<Self, DomainError> {
        if value.is_finite() && value > 0.0 && value <= 100.0 {
            Ok(Self(value))
        } else {
            Err(DomainError::InvalidPercentage(value))
        }
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ChatId(i64);

impl ChatId {
    pub fn new(value: i64) -> Result<Self, DomainError> {
        if value != 0 {
            Ok(Self(value))
        } else {
            Err(DomainError::InvalidChatId(value))
        }
    }

    pub fn value(self) -> i64 {
        self.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Timestamp(i64);

impl Timestamp {
    pub fn new(epoch_millis: i64) -> Result<Self, DomainError> {
        if epoch_millis > 0 {
            Ok(Self(epoch_millis))
        } else {
            Err(DomainError::InvalidTimestamp(epoch_millis))
        }
    }

    pub fn epoch_millis(self) -> i64 {
        self.0
    }

    pub fn elapsed_secs_since(self, older: Timestamp) -> Option<u64> {
        let diff = self.0.checked_sub(older.0)?;
        (diff >= 0).then_some((diff as u64) / 1_000)
    }
}
