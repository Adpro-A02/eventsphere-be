use rocket::{
    request::{FromRequest, Outcome},
    http::Status,
    Request,
};

/// Authentication guard for protected routes
pub struct AuthGuard {
    pub user_id: String,
    pub role: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthGuard {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // Get the authorization header
        let auth_header = request.headers().get_one("Authorization");
        
        match auth_header {
            Some(header) if header.starts_with("Bearer ") => {
                let token = header[7..].trim();
                
                // TODO: Verify JWT token and extract claims
                // This is a placeholder implementation
                if token == "valid-token" {
                    Outcome::Success(AuthGuard {
                        user_id: "user123".to_string(),
                        role: "user".to_string(),
                    })
                } else {
                    Outcome::Failure((Status::Unauthorized, ()))
                }
            },
            _ => Outcome::Failure((Status::Unauthorized, ())),
        }
    }
}

/// Role-based authorization guard
pub struct RoleGuard {
    pub roles: Vec<String>,
}

impl RoleGuard {
    pub fn new(roles: Vec<&str>) -> Self {
        Self {
            roles: roles.iter().map(|&r| r.to_string()).collect(),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RoleGuard {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // First, ensure we have a valid authentication
        let auth_outcome = request.guard::<AuthGuard>().await;
        
        match auth_outcome {
            Outcome::Success(auth) => {
                // TODO: Check if user has required role
                // This is a placeholder implementation
                if auth.role == "admin" {
                    Outcome::Success(RoleGuard {
                        roles: vec!["admin".to_string()],
                    })
                } else {
                    Outcome::Failure((Status::Forbidden, ()))
                }
            },
            _ => Outcome::Failure((Status::Unauthorized, ())),
        }
    }
}
