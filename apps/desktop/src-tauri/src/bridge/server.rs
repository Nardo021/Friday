use std::sync::Arc;

use axum::Router;
use tauri::AppHandle;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

use crate::bridge::auth::SharedAuthToken;
use crate::bridge::broadcast::BridgeBroadcast;
use crate::bridge::routes::router;
use crate::bridge::state::BridgeState;
use crate::core::AgentCore;

static BRIDGE_HANDLE: std::sync::OnceLock<tokio::sync::Mutex<Option<BridgeServerHandle>>> =
    std::sync::OnceLock::new();

struct BridgeServerHandle {
    shutdown: tokio::sync::oneshot::Sender<()>,
    join: tauri::async_runtime::JoinHandle<()>,
}

pub fn start_bridge(
    app: AppHandle,
    core: Arc<AgentCore>,
    broadcast: BridgeBroadcast,
    port: u16,
    auth_token: String,
) {
    stop_bridge();

    let token_store: SharedAuthToken = Arc::new(RwLock::new(auth_token));
    let state = BridgeState {
        core,
        app,
        auth_token: token_store.clone(),
        broadcast,
    };

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    let join = tauri::async_runtime::spawn(async move {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let app = Router::new().merge(router(state)).layer(cors);
        let addr = format!("0.0.0.0:{port}");
        let listener = match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Friday bridge failed to bind {addr}: {e}");
                return;
            }
        };

        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
            .ok();
    });

    let handle = BridgeServerHandle {
        shutdown: shutdown_tx,
        join,
    };

    let slot = BRIDGE_HANDLE.get_or_init(|| tokio::sync::Mutex::new(None));
    tauri::async_runtime::block_on(async {
        *slot.lock().await = Some(handle);
    });
}

pub fn stop_bridge() {
    let Some(slot) = BRIDGE_HANDLE.get() else {
        return;
    };
    tauri::async_runtime::block_on(async {
        if let Some(handle) = slot.lock().await.take() {
            let _ = handle.shutdown.send(());
            let _ = handle.join.await;
        }
    });
}

pub fn update_bridge_token(token: String) {
    // Token is stored in BridgeState via SharedAuthToken; callers should restart bridge.
    let _ = token;
}

pub fn local_ip() -> Option<String> {
    local_ip_address::local_ip()
        .ok()
        .map(|ip| ip.to_string())
}

pub fn bridge_url(port: u16) -> String {
    let ip = local_ip().unwrap_or_else(|| "127.0.0.1".into());
    format!("http://{ip}:{port}")
}
