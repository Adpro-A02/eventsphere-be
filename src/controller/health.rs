use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::metrics::MetricsState;

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    status: String,
    version: String,
    timestamp: u64,
    uptime: u64,
}

#[derive(Serialize, Deserialize)]
pub struct ServiceInfo {
    name: String,
    status: String,
}

#[derive(Serialize, Deserialize)]
pub struct DetailedHealthResponse {
    status: String,
    version: String,
    timestamp: u64,
    uptime: u64,
    services: Vec<ServiceInfo>,
}

static START_TIME: once_cell::sync::Lazy<u64> = once_cell::sync::Lazy::new(|| {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
});

#[get("/health")]
pub fn health_check(metrics_state: &State<Arc<MetricsState>>) -> Json<HealthResponse> {
    metrics_state.record_function_call("health_check");
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let uptime = now - *START_TIME;

    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: now,
        uptime,
    })
}

#[get("/health/detailed")]
pub async fn detailed_health_check(
    db_pool: &rocket::State<std::sync::Arc<sqlx::PgPool>>,
    metrics_state: &State<Arc<MetricsState>>,
) -> Result<Json<DetailedHealthResponse>, Status> {
    metrics_state.record_function_call("detailed_health_check");
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let uptime = now - *START_TIME;

    let db_status = match db_pool.acquire().await {
        Ok(_) => "ok",
        Err(_) => "error",
    };

    let services = vec![
        ServiceInfo {
            name: "database".to_string(),
            status: db_status.to_string(),
        },
    ];

    let status = if services.iter().all(|s| s.status == "ok") {
        "ok"
    } else {
        "degraded"
    };

    Ok(Json(DetailedHealthResponse {
        status: status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: now,
        uptime,
        services,
    }))
}