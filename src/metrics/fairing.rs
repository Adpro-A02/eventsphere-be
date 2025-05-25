use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Request, Response, Data};
use std::time::Instant;
use crate::metrics::MetricsState;

pub struct MetricsFairing;

#[rocket::async_trait]
impl Fairing for MetricsFairing {
    fn info(&self) -> Info {
        Info {
            name: "Metrics Collection",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
        request.local_cache(|| Instant::now());
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, _response: &mut Response<'r>) {
        if let Some(metrics_state) = request.rocket().state::<MetricsState>() {
            // Increment request counter
            metrics_state.http_requests_total.inc();

            // Record request duration
            let start_time = request.local_cache(|| Instant::now());
            let duration = start_time.elapsed();
            metrics_state.request_duration.observe(duration.as_secs_f64());
        }
    }
}

// docker run -p 9090:9090 -v "${PWD}\prometheus.yml:/etc/prometheus/prometheus.yml" prom/prometheus