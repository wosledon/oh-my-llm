use crate::proxy::router::{route_request, ProxyState};
use axum::routing::{get, post};
use axum::Router;
use rusqlite::Connection;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};
use tower_http::cors::{Any, CorsLayer};

pub struct ProxyServer {
    shutdown_tx: Option<oneshot::Sender<()>>,
    pub port: u16,
}

impl ProxyServer {
    pub fn new() -> Self {
        Self {
            shutdown_tx: None,
            port: 11888,
        }
    }

    pub async fn start(&mut self, db: Arc<Mutex<Connection>>, port: u16) -> Result<(), String> {
        if self.shutdown_tx.is_some() {
            return Err("Proxy already running".to_string());
        }

        self.port = port;

        let state = ProxyState::new(db);

        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let app = Router::new()
            .route("/v1/chat/completions", post(route_request))
            .route("/v1/models", get(route_request))
            .route("/health", get(route_request))
            .layer(cors)
            .with_state(state);

        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        let (tx, rx) = oneshot::channel::<()>();
        self.shutdown_tx = Some(tx);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

        tokio::spawn(async move {
            let server = axum::serve(listener, app);
            let _ = server.with_graceful_shutdown(async {
                let _ = rx.await;
            }).await;
        });

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.shutdown_tx.is_some()
    }
}
