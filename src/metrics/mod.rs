use prometheus::{
    Counter, CounterVec, Encoder, Gauge, Histogram, HistogramOpts, Opts, Registry, TextEncoder,
};
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
    pub function_calls_total: CounterVec,
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

        let function_calls_total = CounterVec::new(
            Opts::new("function_calls_total", "Total number of function calls"),
            &["function"],
        )
        .expect("Failed to create function_calls_total counter");

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
        registry
            .register(Box::new(function_calls_total.clone()))
            .expect("Failed to register function_calls_total");

        Self {
            registry,
            http_requests_total,
            active_connections,
            request_duration,
            database_connections,
            function_calls_total,
        }
    }

    pub fn record_function_call(&self, function_name: &str) {
        self.function_calls_total
            .with_label_values(&[function_name])
            .inc();
    }

    pub fn record_request(&self, method: &str, endpoint: &str, status_code: u16) {
        self.http_requests_total.inc();
    }

    pub fn set_active_connections(&self, count: f64) {
        self.active_connections.set(count);
    }

    pub fn set_database_connections(&self, count: f64) {
        self.database_connections.set(count);
    }

    pub fn record_request_duration(&self, duration_seconds: f64) {
        self.request_duration.observe(duration_seconds);
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
