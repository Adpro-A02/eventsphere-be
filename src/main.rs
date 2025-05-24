#[macro_use]
extern crate rocket;

mod controller;
mod middleware;
mod model;
mod repository;
mod service;
use dotenv::dotenv;
use rocket::fairing::AdHoc;
use rocket::{Build, Rocket};
use rocket_cors::{AllowedOrigins, CorsOptions};
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::sync::Arc;

use crate::controller::auth::auth_controller::auth_routes;
use crate::controller::transaction::transaction_controller::{
    balance_routes, transaction_routes, user_routes,
};
use crate::repository::auth::token_repo::{PostgresRefreshTokenRepository, TokenRepository};
use crate::repository::transaction::balance_repo::{
    BalanceRepository, DbBalanceRepository, PostgresBalancePersistence,
};
use crate::repository::transaction::transaction_repo::{
    DbTransactionRepository, PostgresTransactionPersistence, TransactionRepository,
};
use crate::repository::user::user_repo::{
    DbUserRepository, PostgresUserRepository, UserRepository,
};
use crate::service::auth::auth_service::AuthService;
use crate::service::transaction::balance_service::{BalanceService, DefaultBalanceService};
use crate::service::transaction::payment_service::{MockPaymentService, PaymentService};
use crate::service::transaction::transaction_service::{
    DefaultTransactionService, TransactionService,
};

struct AppState {
    db_pool: Arc<sqlx::PgPool>,
    auth_service: Arc<AuthService>,
    transaction_service: Arc<dyn TransactionService + Send + Sync>,
}

fn cors_fairing() -> rocket_cors::Cors {
    let allowed_origins = AllowedOrigins::some_exact(&["http://localhost:3000"]);

    CorsOptions {
        allowed_origins,
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("Error while building CORS")
}

#[launch]
fn rocket() -> Rocket<Build> {
    dotenv().ok();
    rocket::build()
        .attach(AdHoc::on_ignite("Database Setup", |rocket| async {
            let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:Priapta123@localhost:5432/eventsphere".to_string()
            });

            let db_pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await
                .expect("Failed to create database pool");

            let db_pool_arc = Arc::new(db_pool);

            let user_persistence = PostgresUserRepository::new(db_pool_arc.clone());
            let user_repository: Arc<dyn UserRepository> =
                Arc::new(DbUserRepository::new(user_persistence));
            let token_repository: Arc<dyn TokenRepository> =
                Arc::new(PostgresRefreshTokenRepository::new(db_pool_arc.clone()));

            let jwt_secret =
                env::var("JWT_SECRET").unwrap_or_else(|_| "dev_jwt_secret_key".to_string());
            let jwt_refresh_secret = env::var("JWT_REFRESH_SECRET")
                .unwrap_or_else(|_| "dev_jwt_refresh_secret".to_string());
            let pepper = env::var("PEPPER").unwrap_or_else(|_| "dev_password_pepper".to_string());

            let auth_service = Arc::new(
                AuthService::new(jwt_secret, jwt_refresh_secret, pepper)
                    .with_token_repository(token_repository)
                    .with_user_repository(user_repository.clone()),
            );

            let transaction_persistence =
                PostgresTransactionPersistence::new((*db_pool_arc).clone());
            let transaction_repository: Arc<dyn TransactionRepository + Send + Sync> =
                Arc::new(DbTransactionRepository::new(transaction_persistence));

            let balance_persistence = PostgresBalancePersistence::new((*db_pool_arc).clone());
            let balance_repository: Arc<dyn BalanceRepository + Send + Sync> =
                Arc::new(DbBalanceRepository::new(balance_persistence));

            let balance_service: Arc<dyn BalanceService + Send + Sync> =
                Arc::new(DefaultBalanceService::new(balance_repository.clone()));

            let payment_service: Arc<dyn PaymentService + Send + Sync> =
                Arc::new(MockPaymentService::new());

            let transaction_service: Arc<dyn TransactionService + Send + Sync> =
                Arc::new(DefaultTransactionService::new(
                    transaction_repository.clone(),
                    balance_service.clone(),
                    payment_service.clone(),
                ));

            let state = AppState {
                db_pool: db_pool_arc.clone(),
                auth_service: auth_service.clone(),
                transaction_service: transaction_service.clone(),
            };

            rocket
                .manage(state)
                .manage(user_repository.clone())
                .manage(auth_service.clone())
                .manage(transaction_service.clone())
                .manage(balance_service.clone())
                .manage(payment_service.clone())
                .manage(transaction_repository.clone())
                .manage(balance_repository.clone())
                .manage(db_pool_arc)
        }))
        .attach(cors_fairing())
        .mount("/api", auth_routes())
        .mount("/api/transactions", transaction_routes())
        .mount("/api/balance", balance_routes())
        .mount("/api/users", user_routes())
        .mount("/", routes![all_options])
}

#[options("/<_..>")]
fn all_options() -> &'static str {
    ""
}
