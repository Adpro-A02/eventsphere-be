use prometheus::{Counter, Encoder, Gauge, Histogram, HistogramOpts, Registry, TextEncoder};
use rocket::{Route, State, get, routes};
use std::sync::Arc;

pub mod fairing;
pub use fairing::MetricsFairing;

pub struct MetricsState {
    pub registry: Arc<Registry>,
    pub http_requests_total: Counter,
    pub active_connections: Gauge,
    pub request_duration: Histogram,
    pub database_connections: Gauge,
}

impl MetricsState {
    pub fn new() -> Self {
        let registry = Arc::new(Registry::new());

        let http_requests_total =
            Counter::new("http_requests_total", "Total number of HTTP requests")
                .expect("Failed to create http_requests_total counter");

        let active_connections = Gauge::new("active_connections", "Number of active connections")
            .expect("Failed to create active_connections gauge");

        let request_duration = Histogram::with_opts(HistogramOpts::new(
            "request_duration_seconds",
            "Duration of HTTP requests in seconds",
        ))
        .expect("Failed to create request_duration histogram");

        let database_connections = Gauge::new(
            "database_connections",
            "Number of active database connections",
        )
        .expect("Failed to create database_connections gauge");

        registry
            .register(Box::new(http_requests_total.clone()))
            .expect("Failed to register http_requests_total");
        registry
            .register(Box::new(active_connections.clone()))
            .expect("Failed to register active_connections");
        registry
            .register(Box::new(request_duration.clone()))
            .expect("Failed to register request_duration");
        registry
            .register(Box::new(database_connections.clone()))
            .expect("Failed to register database_connections");

        Self {
            registry,
            http_requests_total,
            active_connections,
            request_duration,
            database_connections,
        }
    }
}

#[get("/metrics")]
pub fn metrics_handler(metrics_state: &State<Arc<MetricsState>>) -> String {
    let encoder = TextEncoder::new();
    let metric_families = metrics_state.registry.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

pub fn metrics_routes() -> Vec<Route> {
    routes![metrics_handler]
}
