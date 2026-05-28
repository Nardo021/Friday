use std::sync::Arc;

use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use tokio::sync::RwLock;

use crate::bridge::state::BridgeState;

pub type SharedAuthToken = Arc<RwLock<String>>;

pub fn extract_bearer_token(header: &str) -> Option<&str> {
    header.strip_prefix("Bearer ").map(str::trim)
}

pub fn unauthorized_json() -> Response {
    (
        axum::http::StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "unauthorized" })),
    )
        .into_response()
}

pub fn authorize_request(
    state: &BridgeState,
    headers: &HeaderMap,
) -> Result<(), axum::http::StatusCode> {
    let expected = state
        .auth_token
        .try_read()
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    if expected.is_empty() {
        return Err(axum::http::StatusCode::UNAUTHORIZED);
    }

    let authorized = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(extract_bearer_token)
        .is_some_and(|token| token == *expected);

    if authorized {
        Ok(())
    } else {
        Err(axum::http::StatusCode::UNAUTHORIZED)
    }
}
