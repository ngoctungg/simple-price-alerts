use crate::model::Symbol;
use crate::ports::SymbolSearchPort;

#[derive(Debug, Clone)]
pub struct SearchSymbolInput {
    pub query: String,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchSymbolOutput {
    pub symbols: Vec<Symbol>,
}

#[derive(Debug)]
pub enum SearchSymbolError<E> {
    EmptyQuery,
    InvalidLimit,
    SearchFailed(E),
}

pub fn search_symbol<P: SymbolSearchPort>(
    search_port: &P,
    input: SearchSymbolInput,
) -> Result<SearchSymbolOutput, SearchSymbolError<P::Error>> {
    let query = input.query.trim();
    if query.is_empty() {
        return Err(SearchSymbolError::EmptyQuery);
    }
    if input.limit == 0 {
        return Err(SearchSymbolError::InvalidLimit);
    }

    let symbols = search_port
        .search_symbols(query, input.limit)
        .map_err(SearchSymbolError::SearchFailed)?;

    Ok(SearchSymbolOutput { symbols })
}
