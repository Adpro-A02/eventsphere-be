use crate::metrics::MetricsState;
use rocket::http::Status;
use rocket::local::blocking::Client;
use rocket::{Build, Rocket};
use std::sync::Arc;

fn create_test_rocket() -> Rocket<Build> {
    let metrics_state = Arc::new(MetricsState::new());
    
    rocket::build()
        .manage(metrics_state.clone())
        .mount("/metrics", routes![crate::metrics::metrics])
}

#[test]
fn test_metrics_endpoint() {
    let client = Client::tracked(create_test_rocket()).expect("valid rocket instance");
    let response = client.get("/metrics").dispatch();
    
    assert_eq!(response.status(), Status::Ok);
    
    let body = response.into_string().expect("body");
    assert!(body.contains("http_requests_total"));
}

#[test]
fn test_metrics_counter_increments() {
    let metrics_state = Arc::new(MetricsState::new());
    
    // Increment a counter
    metrics_state.record_request("GET", "/test", 200);
    metrics_state.record_request("GET", "/test", 200);
    
    // Build a rocket with our metrics state
    let rocket = rocket::build()
        .manage(metrics_state.clone())
        .mount("/metrics", routes![crate::metrics::metrics]);
    
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let response = client.get("/metrics").dispatch();
    
    assert_eq!(response.status(), Status::Ok);
    
    let body = response.into_string().expect("body");
    assert!(body.contains("http_requests_total"));
    
    // The metric should have a value of at least 2
    let metric_line = body.lines()
        .find(|line| line.starts_with("http_requests_total{method=\"GET\",endpoint=\"/test\",code=\"200\"}"))
        .expect("metric should exist");
    
    // Extract the value from the metric line
    let value = metric_line.split_whitespace().last().expect("metric should have a value");
    let value: f64 = value.parse().expect("value should be a number");
    
    assert!(value >= 2.0, "Counter should have been incremented at least twice");
}
