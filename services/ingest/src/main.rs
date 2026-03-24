use axum::{
    extract::{Path, Query, State},
    http::{HeaderValue, Method, StatusCode},
    response::Json,
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db: PgPool,
    redis: redis::Client,
}

// ----- Models -----

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Event {
    id: Uuid,
    event_type: String,
    source: String,
    payload: serde_json::Value,
    severity: String,
    timestamp: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateEvent {
    event_type: String,
    source: String,
    payload: Option<serde_json::Value>,
    severity: String,
    timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Source {
    id: Uuid,
    name: String,
    description: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateSource {
    name: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EventsQuery {
    event_type: Option<String>,
    source: Option<String>,
    severity: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Debug, Serialize)]
struct StatsResponse {
    total: i64,
    by_severity: Vec<CountRow>,
    by_event_type: Vec<CountRow>,
    last_hour: i64,
    last_day: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct CountRow {
    label: Option<String>,
    count: Option<i64>,
}

// ----- Handlers -----

async fn health() -> &'static str {
    "ok"
}

async fn create_event(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateEvent>,
) -> Result<(StatusCode, Json<Event>), (StatusCode, String)> {
    let valid_severities = ["info", "warn", "error", "critical"];
    if !valid_severities.contains(&input.severity.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("severity must be one of: {}", valid_severities.join(", ")),
        ));
    }

    let id = Uuid::new_v4();
    let ts = input.timestamp.unwrap_or_else(Utc::now);
    let payload = input.payload.unwrap_or(serde_json::json!({}));

    let event = sqlx::query_as::<_, Event>(
        r#"INSERT INTO events (id, event_type, source, payload, severity, timestamp)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, event_type, source, payload, severity, timestamp, created_at"#,
    )
    .bind(id)
    .bind(&input.event_type)
    .bind(&input.source)
    .bind(&payload)
    .bind(&input.severity)
    .bind(ts)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Publish to Redis for realtime subscribers
    if let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await {
        let json_str = serde_json::to_string(&event).unwrap_or_default();
        let _: Result<(), _> = conn.publish("events:live", json_str).await;
    }

    Ok((StatusCode::CREATED, Json(event)))
}

async fn list_events(
    State(state): State<Arc<AppState>>,
    Query(params): Query<EventsQuery>,
) -> Result<Json<Vec<Event>>, (StatusCode, String)> {
    let limit = params.limit.unwrap_or(50).min(200);
    let offset = params.offset.unwrap_or(0);

    // Build dynamic query
    let mut conditions: Vec<String> = Vec::new();
    let mut idx = 1u32;

    if params.event_type.is_some() {
        conditions.push(format!("event_type = ${idx}"));
        idx += 1;
    }
    if params.source.is_some() {
        conditions.push(format!("source = ${idx}"));
        idx += 1;
    }
    if params.severity.is_some() {
        conditions.push(format!("severity = ${idx}"));
        idx += 1;
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let query_str = format!(
        "SELECT id, event_type, source, payload, severity, timestamp, created_at FROM events {} ORDER BY timestamp DESC LIMIT ${} OFFSET ${}",
        where_clause,
        idx,
        idx + 1
    );

    let mut query = sqlx::query_as::<_, Event>(&query_str);

    if let Some(ref et) = params.event_type {
        query = query.bind(et);
    }
    if let Some(ref src) = params.source {
        query = query.bind(src);
    }
    if let Some(ref sev) = params.severity {
        query = query.bind(sev);
    }

    query = query.bind(limit).bind(offset);

    let events = query
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(events))
}

async fn get_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Event>, (StatusCode, String)> {
    let event = sqlx::query_as::<_, Event>(
        "SELECT id, event_type, source, payload, severity, timestamp, created_at FROM events WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match event {
        Some(e) => Ok(Json(e)),
        None => Err((StatusCode::NOT_FOUND, "Event not found".to_string())),
    }
}

async fn list_sources(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Source>>, (StatusCode, String)> {
    let sources = sqlx::query_as::<_, Source>(
        "SELECT id, name, description, created_at FROM sources ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(sources))
}

async fn create_source(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateSource>,
) -> Result<(StatusCode, Json<Source>), (StatusCode, String)> {
    let source = sqlx::query_as::<_, Source>(
        r#"INSERT INTO sources (id, name, description)
           VALUES ($1, $2, $3)
           RETURNING id, name, description, created_at"#,
    )
    .bind(Uuid::new_v4())
    .bind(&input.name)
    .bind(&input.description)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("duplicate key") {
            (StatusCode::CONFLICT, "Source name already exists".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    Ok((StatusCode::CREATED, Json(source)))
}

async fn stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<StatsResponse>, (StatusCode, String)> {
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events")
        .fetch_one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let by_severity = sqlx::query_as::<_, CountRow>(
        "SELECT severity AS label, COUNT(*) AS count FROM events GROUP BY severity",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let by_event_type = sqlx::query_as::<_, CountRow>(
        "SELECT event_type AS label, COUNT(*) AS count FROM events GROUP BY event_type ORDER BY count DESC LIMIT 20",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let last_hour: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM events WHERE timestamp > NOW() - INTERVAL '1 hour'")
            .fetch_one(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let last_day: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM events WHERE timestamp > NOW() - INTERVAL '1 day'")
            .fetch_one(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(StatsResponse {
        total: total.0,
        by_severity,
        by_event_type,
        last_hour: last_hour.0,
        last_day: last_day.0,
    }))
}

// ----- Main -----

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // Connect to Postgres
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    // Run migrations
    tracing::info!("Running migrations...");
    sqlx::query(include_str!("../migrations/001_initial.sql"))
        .execute(&db)
        .await
        .expect("Failed to run migrations");
    tracing::info!("Migrations complete.");

    // Connect to Redis
    let redis = redis::Client::open(redis_url).expect("Invalid Redis URL");

    let state = Arc::new(AppState { db, redis });

    // CORS layer
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/events", get(list_events).post(create_event))
        .route("/api/events/{id}", get(get_event))
        .route("/api/sources", get(list_sources).post(create_source))
        .route("/api/stats", get(stats))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081")
        .await
        .expect("Failed to bind");
    tracing::info!("Ingest service listening on 0.0.0.0:8081");
    axum::serve(listener, app).await.expect("Server error");
}
