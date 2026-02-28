use crate::model::{AlertDirection, TriggeredAlert};
use crate::ports::TelegramPort;

#[derive(Debug, Clone)]
pub struct DispatchTelegramAlertInput {
    pub alert: TriggeredAlert,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DispatchTelegramAlertOutput {
    pub user_id: String,
    pub message: String,
}

#[derive(Debug)]
pub enum DispatchTelegramAlertError<E> {
    EmptyUserId,
    EmptySymbolCode,
    SendFailed(E),
}

pub fn dispatch_telegram_alert<P: TelegramPort>(
    telegram_port: &P,
    input: DispatchTelegramAlertInput,
) -> Result<DispatchTelegramAlertOutput, DispatchTelegramAlertError<P::Error>> {
    if input.alert.user_id.trim().is_empty() {
        return Err(DispatchTelegramAlertError::EmptyUserId);
    }
    if input.alert.symbol_code.trim().is_empty() {
        return Err(DispatchTelegramAlertError::EmptySymbolCode);
    }

    let comparator = match input.alert.direction {
        AlertDirection::PriceAbove => "≥",
        AlertDirection::PriceBelow => "≤",
    };

    let message = format!(
        "🔔 {}: {} {} {:.4} (current: {:.4})",
        input.alert.symbol_code,
        "price",
        comparator,
        input.alert.threshold_price,
        input.alert.current_price
    );

    telegram_port
        .send_message(input.alert.user_id.trim(), &message)
        .map_err(DispatchTelegramAlertError::SendFailed)?;

    Ok(DispatchTelegramAlertOutput {
        user_id: input.alert.user_id,
        message,
    })
}
