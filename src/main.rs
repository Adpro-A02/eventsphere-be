#[macro_use] extern crate rocket;

mod model;
mod repository;
mod service;
mod controller;
mod middleware;

use rocket::State;
use std::sync::Arc;
use rocket::fs::{FileServer, relative};
use rocket::{Build, Rocket};
use std::env;
use sqlx::postgres::PgPoolOptions;
use rocket::fairing::AdHoc;

use crate::repository::user::user_repo::{UserRepository, DbUserRepository, PostgresUserRepository};
use crate::repository::auth::token_repo::{TokenRepository, PostgresRefreshTokenRepository};
use crate::service::auth::auth_service::AuthService;
use crate::controller::auth::auth_controller::auth_routes;

struct AppState {
    db_pool: Arc<sqlx::PgPool>,
    auth_service: Arc<AuthService>,
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .attach(AdHoc::on_ignite("Database Setup", |rocket| async {
            let database_url = env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:adalahjojojo@localhost:5432/eventsphere".to_string());
                
            let db_pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await
                .expect("Failed to create database pool");
                
            let db_pool_arc = Arc::new(db_pool);

            let user_persistence = PostgresUserRepository::new(db_pool_arc.clone());
            let user_repository: Arc<dyn UserRepository> = Arc::new(DbUserRepository::new(user_persistence));
            let token_repository: Arc<dyn TokenRepository> = Arc::new(PostgresRefreshTokenRepository::new(db_pool_arc.clone()));
            
            let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "dev_jwt_secret_key".to_string());
            let jwt_refresh_secret = env::var("JWT_REFRESH_SECRET").unwrap_or_else(|_| "dev_jwt_refresh_secret".to_string());
            let pepper = env::var("PEPPER").unwrap_or_else(|_| "dev_password_pepper".to_string());
            
            let auth_service = Arc::new(
                AuthService::new(jwt_secret, jwt_refresh_secret, pepper)
                    .with_token_repository(token_repository)
                    .with_user_repository(user_repository.clone())
            );
            
            let state = AppState {
                db_pool: db_pool_arc,
                auth_service: auth_service.clone(),
            };
            
            rocket
                .manage(state)
                .manage(user_repository.clone())
                .manage(auth_service.clone())
        }))
        .mount("/api", auth_routes())
}