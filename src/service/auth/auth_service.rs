use crate::model::user::User;
use crate::model::auth::RefreshToken;
use crate::repository::auth::token_repo::TokenRepository;
use crate::repository::user::user_repo::UserRepository;
use argon2::{self, Argon2, PasswordHash, PasswordVerifier};
use argon2::password_hash::PasswordHasher;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode, decode, DecodingKey, Validation};
use rocket::fairing::Result;
use serde::{Serialize, Deserialize};
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;

pub struct AuthService {
    jwt_secret: String,
    jwt_refresh_secret: String,
    pepper: String,
    token_repository: Option<Arc<dyn TokenRepository>>,
    user_repository: Option<Arc<dyn UserRepository>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    role: String,
    exp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct RefreshClaims {
    sub: String,
    jti: String,
    exp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

impl AuthService {
    pub fn new(jwt_secret: String, jwt_refresh_secret: String, pepper: String) -> Self {
        Self { 
            jwt_secret, 
            jwt_refresh_secret, 
            pepper,
            token_repository: None,
            user_repository: None,
        }
    }

    pub fn with_token_repository(mut self, repo: Arc<dyn TokenRepository>) -> Self {
        self.token_repository = Some(repo);
        self
    }

    pub fn with_user_repository(mut self, repo: Arc<dyn UserRepository>) -> Self {
        self.user_repository = Some(repo);
        self
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

    pub async fn generate_token(&self, user: &User) -> Result<TokenPair, Box<dyn Error>> {
        // Access Token
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

        // Refresh Token
        let refresh_exp = Utc::now()
            .checked_add_signed(Duration::days(7))
            .expect("valid timestamp")
            .timestamp();

        let mut refresh_token_str = Uuid::new_v4().to_string();

        // Store refresh token in database if repository is configured
        if let Some(repo) = &self.token_repository {
            let refresh_token = RefreshToken::new(
                user.id,
                refresh_token_str.clone(),
                7 // 7 days expiration
            );
            repo.create(&refresh_token).await?;
        }
        // Fall back to JWT-based refresh token if no repository
        else {
            let refresh_claims = RefreshClaims {
                sub: user.id.to_string(),
                jti: Uuid::new_v4().to_string(),
                exp: refresh_exp,
            };

            let encoded_refresh_token = encode(
                &Header::default(),
                &refresh_claims,
                &EncodingKey::from_secret(self.jwt_refresh_secret.as_bytes())
            )?;
            
            // Use the JWT as the token string instead of UUID
            refresh_token_str = encoded_refresh_token;
        }

        Ok(TokenPair {
            access_token: token,
            refresh_token: refresh_token_str,
            expires_in: expiration,
        })
    }

    pub fn verify_token(&self, token: &str) -> Result<Uuid, Box<dyn Error>> {
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_bytes());
        let validation = Validation::default();
        let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
        let user_id = Uuid::parse_str(&token_data.claims.sub)?;
        Ok(user_id)
    }

    pub async fn refresh_access_token(&self, token: &str) -> Result<TokenPair, Box<dyn Error>> {
        let user_id = if let Some(repo) = &self.token_repository {
            // Verify token in database
            let stored_token = repo.find_by_token(token).await?
                .ok_or("Invalid refresh token")?;
                
            if !stored_token.is_valid() {
                return Err("Token expired or revoked".into());
            }
            
            stored_token.user_id
        } else {
            // Fall back to JWT validation
            let decoding_key = DecodingKey::from_secret(self.jwt_refresh_secret.as_bytes());
            let validation = Validation::default();
            let token_data = decode::<RefreshClaims>(token, &decoding_key, &validation)?;
            Uuid::parse_str(&token_data.claims.sub)?
        };
        
        // Get actual user from repository if available
        let user = if let Some(repo) = &self.user_repository {
            repo.find_by_id(user_id).await?
                .ok_or("User not found")?
        } else {
            // Fallback to placeholder if no user repository
            User {
                id: user_id,
                name: String::new(),
                email: String::new(),
                password: String::new(),
                role: crate::model::user::UserRole::Attendee,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_login: None,
            }
        };
        
        self.generate_token(&user).await
    }
    
    pub async fn logout(&self, user_id: Uuid) -> Result<(), Box<dyn Error>> {
        if let Some(repo) = &self.token_repository {
            repo.revoke_all_for_user(user_id).await?;
            Ok(())
        } else {
            // No action needed for JWT-only implementation
            Ok(())
        }
    }

    pub fn get_jwt_secret(&self) -> &str {
        &self.jwt_secret
    }
}
