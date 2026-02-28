use crate::model::{AlertDirection, PriceTick, TriggeredAlert};
use crate::ports::{AlertRulePort, PriceTickSinkPort, TriggeredAlertPort};

#[derive(Debug, Clone)]
pub struct ProcessPriceTickInput {
    pub tick: PriceTick,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessPriceTickOutput {
    pub triggered_alerts: Vec<TriggeredAlert>,
}

#[derive(Debug)]
pub enum ProcessPriceTickError<TickError, RuleError, TriggerError> {
    EmptySymbolCode,
    NonPositivePrice,
    TickPersistFailed(TickError),
    RuleLookupFailed(RuleError),
    TriggerPersistFailed(TriggerError),
}

pub fn process_price_tick<T, R, G>(
    tick_sink: &T,
    alert_rule_port: &R,
    triggered_alert_port: &G,
    input: ProcessPriceTickInput,
) -> Result<ProcessPriceTickOutput, ProcessPriceTickError<T::Error, R::Error, G::Error>>
where
    T: PriceTickSinkPort,
    R: AlertRulePort,
    G: TriggeredAlertPort,
{
    if input.tick.symbol_code.trim().is_empty() {
        return Err(ProcessPriceTickError::EmptySymbolCode);
    }
    if input.tick.price <= 0.0 {
        return Err(ProcessPriceTickError::NonPositivePrice);
    }

    tick_sink
        .append_tick(input.tick.clone())
        .map_err(ProcessPriceTickError::TickPersistFailed)?;

    let rules = alert_rule_port
        .list_enabled_rules_by_symbol(input.tick.symbol_code.trim())
        .map_err(ProcessPriceTickError::RuleLookupFailed)?;

    let mut triggered_alerts = Vec::new();

    for rule in rules {
        let is_triggered = match rule.direction {
            AlertDirection::PriceAbove => input.tick.price >= rule.threshold_price,
            AlertDirection::PriceBelow => input.tick.price <= rule.threshold_price,
        };

        if is_triggered {
            let saved = triggered_alert_port
                .save_triggered_alert(TriggeredAlert {
                    rule_id: rule.id,
                    user_id: rule.user_id,
                    symbol_code: input.tick.symbol_code.clone(),
                    current_price: input.tick.price,
                    threshold_price: rule.threshold_price,
                    direction: rule.direction,
                    triggered_at_ms: input.tick.timestamp_ms,
                })
                .map_err(ProcessPriceTickError::TriggerPersistFailed)?;
            triggered_alerts.push(saved);
        }
    }

    Ok(ProcessPriceTickOutput { triggered_alerts })
}
