use crate::model::WatchedSymbol;
use crate::ports::{SymbolSearchPort, WatchlistPort};

#[derive(Debug, Clone)]
pub struct SelectWatchSymbolInput {
    pub user_id: String,
    pub symbol_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectWatchSymbolOutput {
    pub watched_symbol: WatchedSymbol,
}

#[derive(Debug)]
pub enum SelectWatchSymbolError<SearchError, WatchError> {
    EmptyUserId,
    EmptySymbolCode,
    SymbolLookupFailed(SearchError),
    SymbolNotFound,
    WatchlistWriteFailed(WatchError),
}

pub fn select_watch_symbol<S, W>(
    symbol_port: &S,
    watchlist_port: &W,
    input: SelectWatchSymbolInput,
) -> Result<SelectWatchSymbolOutput, SelectWatchSymbolError<S::Error, W::Error>>
where
    S: SymbolSearchPort,
    W: WatchlistPort,
{
    let user_id = input.user_id.trim();
    if user_id.is_empty() {
        return Err(SelectWatchSymbolError::EmptyUserId);
    }

    let symbol_code = input.symbol_code.trim();
    if symbol_code.is_empty() {
        return Err(SelectWatchSymbolError::EmptySymbolCode);
    }

    let symbol = symbol_port
        .find_symbol_by_code(symbol_code)
        .map_err(SelectWatchSymbolError::SymbolLookupFailed)?
        .ok_or(SelectWatchSymbolError::SymbolNotFound)?;

    let watched_symbol = watchlist_port
        .upsert_watched_symbol(WatchedSymbol {
            user_id: user_id.to_string(),
            symbol,
        })
        .map_err(SelectWatchSymbolError::WatchlistWriteFailed)?;

    Ok(SelectWatchSymbolOutput { watched_symbol })
}
