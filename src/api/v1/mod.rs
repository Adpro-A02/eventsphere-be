pub mod tickets;
pub mod transactions;
pub mod events;
pub mod users;
pub mod auth;

use rocket::{routes, Route};

/// Collects all API v1 routes
pub fn routes() -> Vec<Route> {
    let mut all_routes = Vec::new();
    
    // Combine routes from all API modules
    all_routes.extend(tickets::routes());
    all_routes.extend(transactions::routes());
    all_routes.extend(events::routes());
    all_routes.extend(users::routes());
    all_routes.extend(auth::routes());
    
    all_routes
}
