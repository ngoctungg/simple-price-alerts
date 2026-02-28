use crate::model::{AlertDirection, AlertRule};
use crate::ports::AlertRulePort;

#[derive(Debug, Clone)]
pub struct UpsertAlertRuleInput {
    pub id: String,
    pub user_id: String,
    pub symbol_code: String,
    pub direction: AlertDirection,
    pub threshold_price: f64,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpsertAlertRuleOutput {
    pub rule: AlertRule,
}

#[derive(Debug)]
pub enum UpsertAlertRuleError<E> {
    EmptyRuleId,
    EmptyUserId,
    EmptySymbolCode,
    NonPositiveThreshold,
    WriteFailed(E),
}

pub fn upsert_alert_rule<P: AlertRulePort>(
    alert_rule_port: &P,
    input: UpsertAlertRuleInput,
) -> Result<UpsertAlertRuleOutput, UpsertAlertRuleError<P::Error>> {
    if input.id.trim().is_empty() {
        return Err(UpsertAlertRuleError::EmptyRuleId);
    }
    if input.user_id.trim().is_empty() {
        return Err(UpsertAlertRuleError::EmptyUserId);
    }
    if input.symbol_code.trim().is_empty() {
        return Err(UpsertAlertRuleError::EmptySymbolCode);
    }
    if input.threshold_price <= 0.0 {
        return Err(UpsertAlertRuleError::NonPositiveThreshold);
    }

    let rule = alert_rule_port
        .upsert_alert_rule(AlertRule {
            id: input.id.trim().to_string(),
            user_id: input.user_id.trim().to_string(),
            symbol_code: input.symbol_code.trim().to_string(),
            direction: input.direction,
            threshold_price: input.threshold_price,
            enabled: input.enabled,
        })
        .map_err(UpsertAlertRuleError::WriteFailed)?;

    Ok(UpsertAlertRuleOutput { rule })
}
