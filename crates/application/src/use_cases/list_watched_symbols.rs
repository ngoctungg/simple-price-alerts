use crate::model::WatchedSymbol;
use crate::ports::WatchlistPort;

#[derive(Debug, Clone)]
pub struct ListWatchedSymbolsInput {
    pub user_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListWatchedSymbolsOutput {
    pub watched_symbols: Vec<WatchedSymbol>,
}

#[derive(Debug)]
pub enum ListWatchedSymbolsError<E> {
    EmptyUserId,
    ListFailed(E),
}

pub fn list_watched_symbols<P: WatchlistPort>(
    watchlist_port: &P,
    input: ListWatchedSymbolsInput,
) -> Result<ListWatchedSymbolsOutput, ListWatchedSymbolsError<P::Error>> {
    let user_id = input.user_id.trim();
    if user_id.is_empty() {
        return Err(ListWatchedSymbolsError::EmptyUserId);
    }

    let watched_symbols = watchlist_port
        .list_watched_symbols(user_id)
        .map_err(ListWatchedSymbolsError::ListFailed)?;

    Ok(ListWatchedSymbolsOutput { watched_symbols })
}
