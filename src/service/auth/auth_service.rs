use crate::model::user::{User, UserRole};
use argon2::{self, Config};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Serialize;
use std::error::Error;
use uuid::Uuid;

pub struct AuthService {
    jwt_secret: String,
    pepper: String,
}

#[derive(Debug, Serialize)]
struct Claims {
    sub: String,
    role: String,
    exp: i64,
}

impl AuthService {
    pub fn new(jwt_secret: String, pepper: String) -> Self {
        Self { jwt_secret, pepper }
    }

    pub fn hash_password(&self, password: &str) -> Result<String, Box<dyn Error>> {
        let salt = Uuid::new_v4().to_string();
        let config = Config::default();
        let hash = argon2::hash_encoded(
            (password.to_owned() + &self.pepper).as_bytes(),
            salt.as_bytes(),
            &config,
        )?;
        Ok(hash)
    }

    pub fn verify_password(&self, hash: &str, password: &str) -> Result<bool, Box<dyn Error>> {
        Ok(argon2::verify_encoded(
            hash,
            (password.to_owned() + &self.pepper).as_bytes(),
        )?)
    }

    pub fn generate_token(&self, user: &User) -> Result<String, Box<dyn Error>> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user.id.to_string(),
            role: format!("{:?}", user.role),
            exp: expiration,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes())
        )?;

        Ok(token)
    }
}
