use axum::{
    extract::{Path, Query, State, WebSocketUpgrade, ws::{Message, WebSocket}},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::bridge::auth::{authorize_request, unauthorized_json};
use crate::bridge::broadcast::BridgeBroadcast;
use crate::bridge::state::BridgeState;
use crate::core::event::AgentEvent;
use crate::security::approval_manager::PendingApprovalInfo;
use crate::storage::{EventsRepo, SettingsRepo};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthResponse {
    ok: bool,
    version: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InfoResponse {
    hostname: String,
    active_session_count: usize,
    bridge_port: u16,
}

#[derive(Deserialize)]
struct EventsQuery {
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    100
}

#[derive(Deserialize)]
struct WsQuery {
    token: String,
}

pub fn router(state: BridgeState) -> Router {
    let public = Router::new().route("/health", get(health));

    let protected = Router::new()
        .route("/info", get(info))
        .route("/sessions", get(list_sessions))
        .route("/sessions/{id}", get(get_session))
        .route("/sessions/{id}/events", get(session_events))
        .route("/sessions/{id}/stop", post(stop_session))
        .route("/approvals/pending", get(pending_approvals))
        .route("/approvals/{id}/approve", post(approve))
        .route("/approvals/{id}/reject", post(reject))
        .route("/ws", get(ws_handler))
        .with_state(state);

    Router::new()
        .nest("/v1", protected)
        .merge(public)
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn info(
    State(state): State<BridgeState>,
    headers: HeaderMap,
) -> Result<Json<InfoResponse>, StatusCode> {
    authorize_request(&state, &headers)?;
    let sessions = state
        .core
        .list_active_sessions()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let settings = SettingsRepo::new(&state.core.db)
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(InfoResponse {
        hostname: hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "friday-desktop".into()),
        active_session_count: sessions.len(),
        bridge_port: settings.mobile_bridge.port,
    }))
}

async fn list_sessions(
    State(state): State<BridgeState>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::core::event::FridaySession>>, StatusCode> {
    authorize_request(&state, &headers)?;
    state
        .core
        .list_active_sessions()
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_session(
    State(state): State<BridgeState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<crate::core::event::FridaySession>, StatusCode> {
    authorize_request(&state, &headers)?;
    state
        .core
        .get_session(&id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

async fn session_events(
    State(state): State<BridgeState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<EventsQuery>,
) -> Result<Json<Vec<AgentEvent>>, StatusCode> {
    authorize_request(&state, &headers)?;
    let events = EventsRepo::new(&state.core.db)
        .list_for_session_limit(&id, Some(query.limit))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(events))
}

async fn pending_approvals(
    State(state): State<BridgeState>,
    headers: HeaderMap,
) -> Result<Json<Vec<PendingApprovalInfo>>, StatusCode> {
    authorize_request(&state, &headers)?;
    let pending = state.core.approval_manager.lock().await.list_pending();
    Ok(Json(pending))
}

async fn approve(
    State(state): State<BridgeState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    authorize_request(&state, &headers)?;
    state
        .core
        .approve_command(state.app.clone(), &id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|_| StatusCode::NOT_FOUND)
}

async fn reject(
    State(state): State<BridgeState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    authorize_request(&state, &headers)?;
    state
        .core
        .reject_command(state.app.clone(), &id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|_| StatusCode::NOT_FOUND)
}

async fn stop_session(
    State(state): State<BridgeState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    authorize_request(&state, &headers)?;
    state
        .core
        .stop_session(state.app.clone(), &id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|_| StatusCode::NOT_FOUND)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsQuery>,
    State(state): State<BridgeState>,
) -> impl IntoResponse {
    let expected = state.auth_token.read().await.clone();
    if expected.is_empty() || params.token != expected {
        return unauthorized_json().into_response();
    }

    ws.on_upgrade(move |socket| handle_ws(socket, state.broadcast))
        .into_response()
}

async fn handle_ws(mut socket: WebSocket, broadcast: BridgeBroadcast) {
    let mut rx = broadcast.subscribe();

    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Err(_)) => break,
                    _ => {}
                }
            }
            event = rx.recv() => {
                match event {
                    Ok(json) => {
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                }
            }
        }
    }
}
