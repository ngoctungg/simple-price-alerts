#[cfg(test)]
mod tests {
    use crate::model::{
        AlertDirection, AlertRule, PriceTick, Symbol, TriggeredAlert, WatchedSymbol,
    };
    use crate::ports::{
        AlertRulePort, PriceTickSinkPort, SymbolSearchPort, TelegramPort, TriggeredAlertPort,
        WatchlistPort,
    };
    use crate::use_cases::dispatch_telegram_alert::{
        dispatch_telegram_alert, DispatchTelegramAlertInput,
    };
    use crate::use_cases::list_watched_symbols::{list_watched_symbols, ListWatchedSymbolsInput};
    use crate::use_cases::process_price_tick::{process_price_tick, ProcessPriceTickInput};
    use crate::use_cases::search_symbol::{search_symbol, SearchSymbolInput};
    use crate::use_cases::select_watch_symbol::{select_watch_symbol, SelectWatchSymbolInput};
    use crate::use_cases::upsert_alert_rule::{upsert_alert_rule, UpsertAlertRuleInput};

    #[derive(Default)]
    struct FakePorts {
        symbols: Vec<Symbol>,
        watched: Vec<WatchedSymbol>,
        rules: Vec<AlertRule>,
        triggers: Vec<TriggeredAlert>,
        messages: Vec<(String, String)>,
        ticks: Vec<PriceTick>,
    }

    impl SymbolSearchPort for FakePorts {
        type Error = String;

        fn search_symbols(&self, query: &str, _limit: usize) -> Result<Vec<Symbol>, Self::Error> {
            Ok(self
                .symbols
                .iter()
                .filter(|s| s.code.contains(query))
                .cloned()
                .collect())
        }

        fn find_symbol_by_code(&self, symbol_code: &str) -> Result<Option<Symbol>, Self::Error> {
            Ok(self.symbols.iter().find(|s| s.code == symbol_code).cloned())
        }
    }

    impl WatchlistPort for std::cell::RefCell<FakePorts> {
        type Error = String;

        fn upsert_watched_symbol(
            &self,
            watched: WatchedSymbol,
        ) -> Result<WatchedSymbol, Self::Error> {
            self.borrow_mut().watched.push(watched.clone());
            Ok(watched)
        }

        fn list_watched_symbols(&self, user_id: &str) -> Result<Vec<WatchedSymbol>, Self::Error> {
            Ok(self
                .borrow()
                .watched
                .iter()
                .filter(|w| w.user_id == user_id)
                .cloned()
                .collect())
        }
    }

    impl AlertRulePort for std::cell::RefCell<FakePorts> {
        type Error = String;

        fn upsert_alert_rule(&self, rule: AlertRule) -> Result<AlertRule, Self::Error> {
            self.borrow_mut().rules.push(rule.clone());
            Ok(rule)
        }

        fn list_enabled_rules_by_symbol(
            &self,
            symbol_code: &str,
        ) -> Result<Vec<AlertRule>, Self::Error> {
            Ok(self
                .borrow()
                .rules
                .iter()
                .filter(|r| r.enabled && r.symbol_code == symbol_code)
                .cloned()
                .collect())
        }
    }

    impl TriggeredAlertPort for std::cell::RefCell<FakePorts> {
        type Error = String;

        fn save_triggered_alert(
            &self,
            alert: TriggeredAlert,
        ) -> Result<TriggeredAlert, Self::Error> {
            self.borrow_mut().triggers.push(alert.clone());
            Ok(alert)
        }
    }

    impl TelegramPort for std::cell::RefCell<FakePorts> {
        type Error = String;

        fn send_message(&self, user_id: &str, message: &str) -> Result<(), Self::Error> {
            self.borrow_mut()
                .messages
                .push((user_id.to_string(), message.to_string()));
            Ok(())
        }
    }

    impl PriceTickSinkPort for std::cell::RefCell<FakePorts> {
        type Error = String;

        fn append_tick(&self, tick: PriceTick) -> Result<(), Self::Error> {
            self.borrow_mut().ticks.push(tick);
            Ok(())
        }
    }

    #[test]
    fn use_cases_flow_over_ports() {
        let symbol = Symbol {
            code: "BTCUSDT".into(),
            display_name: "Bitcoin".into(),
            exchange: "BINANCE".into(),
        };

        let search_source = FakePorts {
            symbols: vec![symbol.clone()],
            ..Default::default()
        };

        let search = search_symbol(
            &search_source,
            SearchSymbolInput {
                query: "BTC".into(),
                limit: 10,
            },
        )
        .unwrap();
        assert_eq!(search.symbols.len(), 1);

        let state = std::cell::RefCell::new(FakePorts {
            symbols: vec![symbol.clone()],
            ..Default::default()
        });

        let selected = select_watch_symbol(
            &search_source,
            &state,
            SelectWatchSymbolInput {
                user_id: "u1".into(),
                symbol_code: "BTCUSDT".into(),
            },
        )
        .unwrap();
        assert_eq!(selected.watched_symbol.user_id, "u1");

        let listed = list_watched_symbols(
            &state,
            ListWatchedSymbolsInput {
                user_id: "u1".into(),
            },
        )
        .unwrap();
        assert_eq!(listed.watched_symbols.len(), 1);

        upsert_alert_rule(
            &state,
            UpsertAlertRuleInput {
                id: "r1".into(),
                user_id: "u1".into(),
                symbol_code: "BTCUSDT".into(),
                direction: AlertDirection::PriceAbove,
                threshold_price: 60000.0,
                enabled: true,
            },
        )
        .unwrap();

        let processed = process_price_tick(
            &state,
            &state,
            &state,
            ProcessPriceTickInput {
                tick: PriceTick {
                    symbol_code: "BTCUSDT".into(),
                    price: 65000.0,
                    timestamp_ms: 1,
                },
            },
        )
        .unwrap();
        assert_eq!(processed.triggered_alerts.len(), 1);

        let dispatched = dispatch_telegram_alert(
            &state,
            DispatchTelegramAlertInput {
                alert: processed.triggered_alerts[0].clone(),
            },
        )
        .unwrap();
        assert_eq!(dispatched.user_id, "u1");
    }
}
