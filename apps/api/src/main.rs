use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

#[derive(Clone, Default)]
struct AppState {
    symbols: Vec<String>,
    watchlist: Arc<RwLock<Vec<WatchlistItem>>>,
    alerts: Arc<RwLock<HashMap<Uuid, Alert>>>,
}

#[derive(Debug, Serialize, Clone)]
struct WatchlistItem {
    symbol: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Clone)]
struct Alert {
    id: Uuid,
    symbol: String,
    target_price: f64,
    is_active: bool,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
struct SearchSymbolsQuery {
    #[validate(length(min = 1, max = 20))]
    q: String,
}

#[derive(Debug, Deserialize, Validate)]
struct CreateWatchlistDto {
    #[validate(length(min = 1, max = 10), regex(path = "*SYMBOL_REGEX"))]
    symbol: String,
    #[validate(range(min = 0.000001))]
    target_price: f64,
}

#[derive(Debug, Deserialize, Validate)]
struct UpdateAlertDto {
    #[validate(range(min = 0.000001))]
    target_price: Option<f64>,
    is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
struct AlertsQuery {
    #[validate(length(min = 1, max = 10), regex(path = "*SYMBOL_REGEX"))]
    symbol: Option<String>,
    is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
struct DeleteWatchlistPath {
    #[validate(length(min = 1, max = 10), regex(path = "*SYMBOL_REGEX"))]
    symbol: String,
}

#[derive(Debug, Serialize)]
struct ApiErrorBody {
    error: ApiErrorPayload,
}

#[derive(Debug, Serialize)]
struct ApiErrorPayload {
    code: &'static str,
    message: String,
    details: Option<serde_json::Value>,
}

#[derive(Debug)]
enum ApiError {
    Validation(ValidationErrors),
    NotFound(String),
    Conflict(String),
    BadRequest(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message, details) = match self {
            ApiError::Validation(errs) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "VALIDATION_ERROR",
                "Request validation failed".to_string(),
                Some(validation_errors_to_json(errs)),
            ),
            ApiError::NotFound(message) => (StatusCode::NOT_FOUND, "NOT_FOUND", message, None),
            ApiError::Conflict(message) => (StatusCode::CONFLICT, "CONFLICT", message, None),
            ApiError::BadRequest(message) => {
                (StatusCode::BAD_REQUEST, "BAD_REQUEST", message, None)
            }
        };

        (
            status,
            Json(ApiErrorBody {
                error: ApiErrorPayload {
                    code,
                    message,
                    details,
                },
            }),
        )
            .into_response()
    }
}

fn validation_errors_to_json(errs: ValidationErrors) -> serde_json::Value {
    serde_json::to_value(&errs).unwrap_or_else(|_| serde_json::json!({}))
}

type ApiResult<T> = Result<T, ApiError>;

static SYMBOL_REGEX: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"^[A-Z0-9.-]+$").expect("valid regex"));

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let state = AppState {
        symbols: vec![
            "AAPL".into(),
            "AMZN".into(),
            "GOOGL".into(),
            "MSFT".into(),
            "TSLA".into(),
            "BTC-USD".into(),
            "ETH-USD".into(),
        ],
        ..Default::default()
    };

    let app = app_router(state);

    let addr: SocketAddr = "0.0.0.0:3000".parse().expect("valid address");
    info!("API running on {addr}");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind tcp listener");

    axum::serve(listener, app)
        .await
        .expect("server crashed unexpectedly");
}

fn app_router(state: AppState) -> Router {
    Router::new()
        .route("/symbols/search", get(search_symbols))
        .route("/watchlist", post(create_watchlist).get(list_watchlist))
        .route("/watchlist/:symbol", delete(delete_watchlist))
        .route("/alerts", get(list_alerts))
        .route("/alerts/:id", put(update_alert))
        .with_state(state)
}

async fn search_symbols(
    Query(query): Query<SearchSymbolsQuery>,
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<String>>> {
    query.validate().map_err(ApiError::Validation)?;

    let needle = query.q.to_uppercase();
    let results: Vec<String> = state
        .symbols
        .iter()
        .filter(|s| s.contains(&needle))
        .cloned()
        .collect();

    Ok(Json(results))
}

async fn create_watchlist(
    State(state): State<AppState>,
    Json(dto): Json<CreateWatchlistDto>,
) -> ApiResult<impl IntoResponse> {
    dto.validate().map_err(ApiError::Validation)?;

    if !state.symbols.contains(&dto.symbol) {
        return Err(ApiError::BadRequest(format!(
            "Symbol '{}' is not supported",
            dto.symbol
        )));
    }

    let mut watchlist = state.watchlist.write().await;
    if watchlist.iter().any(|item| item.symbol == dto.symbol) {
        return Err(ApiError::Conflict(format!(
            "Symbol '{}' already exists in watchlist",
            dto.symbol
        )));
    }

    let now = Utc::now();
    watchlist.push(WatchlistItem {
        symbol: dto.symbol.clone(),
        created_at: now,
    });

    let mut alerts = state.alerts.write().await;
    let alert = Alert {
        id: Uuid::new_v4(),
        symbol: dto.symbol,
        target_price: dto.target_price,
        is_active: true,
        updated_at: now,
    };
    alerts.insert(alert.id, alert.clone());

    Ok((StatusCode::CREATED, Json(alert)))
}

async fn list_watchlist(State(state): State<AppState>) -> Json<Vec<WatchlistItem>> {
    let watchlist = state.watchlist.read().await;
    Json(watchlist.clone())
}

async fn update_alert(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(dto): Json<UpdateAlertDto>,
) -> ApiResult<Json<Alert>> {
    dto.validate().map_err(ApiError::Validation)?;

    if dto.target_price.is_none() && dto.is_active.is_none() {
        return Err(ApiError::BadRequest(
            "Provide at least one field to update".to_string(),
        ));
    }

    let alert_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest(format!("Alert id '{}' is invalid UUID", id)))?;

    let mut alerts = state.alerts.write().await;
    let alert = alerts
        .get_mut(&alert_id)
        .ok_or_else(|| ApiError::NotFound(format!("Alert '{}' not found", alert_id)))?;

    if let Some(target_price) = dto.target_price {
        alert.target_price = target_price;
    }
    if let Some(is_active) = dto.is_active {
        alert.is_active = is_active;
    }
    alert.updated_at = Utc::now();

    Ok(Json(alert.clone()))
}

async fn list_alerts(
    State(state): State<AppState>,
    Query(query): Query<AlertsQuery>,
) -> ApiResult<Json<Vec<Alert>>> {
    query.validate().map_err(ApiError::Validation)?;

    let alerts = state.alerts.read().await;
    let mut values: Vec<Alert> = alerts.values().cloned().collect();

    if let Some(symbol) = query.symbol {
        values.retain(|a| a.symbol == symbol);
    }

    if let Some(is_active) = query.is_active {
        values.retain(|a| a.is_active == is_active);
    }

    Ok(Json(values))
}

async fn delete_watchlist(
    Path(path): Path<DeleteWatchlistPath>,
    State(state): State<AppState>,
) -> ApiResult<StatusCode> {
    path.validate().map_err(ApiError::Validation)?;

    let mut watchlist = state.watchlist.write().await;
    let initial_len = watchlist.len();
    watchlist.retain(|item| item.symbol != path.symbol);

    if watchlist.len() == initial_len {
        return Err(ApiError::NotFound(format!(
            "Watchlist symbol '{}' not found",
            path.symbol
        )));
    }

    let mut alerts = state.alerts.write().await;
    alerts.retain(|_, alert| alert.symbol != path.symbol);

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn validates_symbol_search_query() {
        let app = app_router(AppState::default());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/symbols/search?q=")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn creates_and_lists_watchlist() {
        let state = AppState {
            symbols: vec!["AAPL".to_string()],
            ..Default::default()
        };
        let app = app_router(state);

        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/watchlist")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({"symbol":"AAPL", "target_price": 100.0}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(create_response.status(), StatusCode::CREATED);

        let list_response = app
            .oneshot(
                Request::builder()
                    .uri("/watchlist")
                    .method("GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(list_response.status(), StatusCode::OK);
        let body = list_response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json.as_array().unwrap().len(), 1);
    }
}
