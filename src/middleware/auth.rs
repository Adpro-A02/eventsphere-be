use rocket::{request::{self, FromRequest, Request}, outcome::Outcome, State};
use rocket::http::Status;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use crate::service::auth::auth_service::AuthService;
use std::sync::Arc;

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
        self.role.to_lowercase() == "admin"
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for JwtToken {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let token = req.headers().get_one("Authorization")
            .map(|value| value.replace("Bearer ", ""));
            
        let token = match token {
            Some(token) => token,
            None => return Outcome::Error((Status::Unauthorized, ())),
        };
        
        let auth_service_ref = match req.guard::<&State<Arc<AuthService>>>().await {
            Outcome::Success(auth) => auth,
            _ => {
                return Outcome::Error((Status::InternalServerError, ()));
            }
        };

        let auth_service = auth_service_ref.inner();
        let secret = auth_service.get_jwt_secret();
        
        let token_data = match decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        ) {
            Ok(c) => c,
            Err(e) => {
                return Outcome::Error((Status::Unauthorized, ()));
            },
        };
        
        let jwt_token = JwtToken {
            user_id: token_data.claims.sub,
            role: token_data.claims.role,
        };
        
        Outcome::Success(jwt_token)
    }
}