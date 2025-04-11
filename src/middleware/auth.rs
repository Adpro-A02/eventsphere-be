// src/middleware/auth.rs
use rocket::{request::{self, FromRequest, Request}, outcome::Outcome};
use rocket::http::Status;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Debug)]
pub struct JwtToken {
    pub user_id: String,
    pub role: String,
}

impl JwtToken {
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for JwtToken {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        // Get the JWT from the Authorization header
        let token = req.headers().get_one("Authorization")
            .map(|value| value.replace("Bearer ", ""));
            
        let token = match token {
            Some(token) => token,
            None => return Outcome::Failure((Status::Unauthorized, ())),
        };
        
        // Get the secret key from configuration
        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string());
        
        // Decode and validate the JWT
        let token_data = match decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        ) {
            Ok(c) => c,
            Err(_) => return Outcome::Failure((Status::Unauthorized, ())),
        };
        
        // Create a JwtToken from the decoded data
        let jwt_token = JwtToken {
            user_id: token_data.claims.sub,
            role: token_data.claims.role,
        };
        
        Outcome::Success(jwt_token)
    }
}