use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    routing::get,
    Router,
};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<String>,
}

async fn health() -> &'static str {
    "ok"
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.tx.subscribe();

    // Spawn a task to forward broadcast messages to this websocket client
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if socket.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    send_task.await.ok();
}

async fn redis_subscriber(tx: broadcast::Sender<String>, redis_url: String) {
    loop {
        match redis_subscribe_loop(&tx, &redis_url).await {
            Ok(_) => {
                tracing::warn!("Redis subscription ended, reconnecting...");
            }
            Err(e) => {
                tracing::error!("Redis subscription error: {}, reconnecting in 2s...", e);
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    }
}

async fn redis_subscribe_loop(
    tx: &broadcast::Sender<String>,
    redis_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = redis::Client::open(redis_url)?;
    let mut pubsub = client.get_async_pubsub().await?;
    pubsub.subscribe("events:live").await?;

    tracing::info!("Subscribed to Redis channel events:live");

    loop {
        let msg = pubsub.on_message().next().await;
        match msg {
            Some(msg) => {
                let payload: String = msg.get_payload()?;
                // Ignore send errors (no subscribers yet)
                let _ = tx.send(payload);
            }
            None => {
                return Ok(());
            }
        }
    }
}

// Need this for the async stream
use futures_util::StreamExt;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    let (tx, _rx) = broadcast::channel::<String>(256);

    // Spawn Redis subscriber
    let redis_tx = tx.clone();
    let redis_url_clone = redis_url.clone();
    tokio::spawn(async move {
        redis_subscriber(redis_tx, redis_url_clone).await;
    });

    let state = Arc::new(AppState { tx });

    let app = Router::new()
        .route("/health", get(health))
        .route("/ws", get(ws_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8082")
        .await
        .expect("Failed to bind");
    tracing::info!("Realtime service listening on 0.0.0.0:8082");
    axum::serve(listener, app).await.expect("Server error");
}
