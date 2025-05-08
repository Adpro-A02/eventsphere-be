use crate::model::user::User;
use argon2::{self, Argon2, PasswordHash, PasswordVerifier};
use argon2::password_hash::PasswordHasher;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode, decode, DecodingKey, Validation};
use serde::{Serialize, Deserialize};
use std::error::Error;
use uuid::Uuid;

pub struct AuthService {
    jwt_secret: String,
    pepper: String,
}

#[derive(Debug, Serialize, Deserialize)]
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
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_with_pepper = format!("{}{}", password, self.pepper);
        let password_hash = argon2.hash_password(password_with_pepper.as_bytes(), &salt)?.to_string();
        Ok(password_hash)
    }

    pub fn verify_password(&self, hash: &str, password: &str) -> Result<bool, Box<dyn Error>> {
        let parsed_hash = PasswordHash::new(hash)?;
        let argon2 = Argon2::default();
        let password_with_pepper = format!("{}{}", password, self.pepper);
        Ok(argon2.verify_password(password_with_pepper.as_bytes(), &parsed_hash).is_ok())
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

    pub fn verify_token(&self, token: &str) -> Result<Uuid, Box<dyn Error>> {
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_bytes());
        let validation = Validation::default();
        let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
        let user_id = Uuid::parse_str(&token_data.claims.sub)?;
        Ok(user_id)
    }
}
