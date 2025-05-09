// filepath: c:\eventsphere-be\src\controller\auth\auth_controller.rs
use crate::model::user::{User, UserRole};
use crate::repository::user::user_repo::UserRepository;
use crate::service::auth::auth_service::{AuthService, TokenPair};
use rocket::{State, post, put, get, serde::json::Json, http::Status, routes};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::error::Error;
use uuid::Uuid;

pub struct AuthController {
    pub user_repository: Arc<dyn UserRepository>,
    pub auth_service: Arc<AuthService>,
}

impl AuthController {
    pub fn new(user_repository: Arc<dyn UserRepository>, auth_service: Arc<AuthService>) -> Self {
        Self {
            user_repository,
            auth_service,
        }
    }
    pub fn routes() -> Vec<rocket::Route> {
        routes![
            register_handler,
            login_handler,
            get_user_handler,
            update_profile_handler,
            refresh_token_handler
        ]
    }
    
    pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse, Box<dyn Error>> {
        if let Ok(Some(_)) = self.user_repository.find_by_email(&req.email).await {
            return Err("User with this email already exists".into());
        }
        
        let hashed_password = self.auth_service.hash_password(&req.password)?;
        
        let role = req.role.unwrap_or(UserRole::Attendee);
        let user = User::new(req.name.clone(), req.email.clone(), hashed_password, role);
        
        self.user_repository.create(&user).await?;
        
        let token_pair = self.auth_service.generate_token(&user).await?;
        
        Ok(AuthResponse {
            token: token_pair.access_token,
            refresh_token: token_pair.refresh_token,
            user_id: user.id,
            name: user.name,
            email: user.email,
            role: user.role,
        })
    }
    
    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse, Box<dyn Error>> {
        let user = match self.user_repository.find_by_email(&req.email).await? {
            Some(u) => u,
            None => return Err("Invalid email or password".into()),
        };
        
        if !self.auth_service.verify_password(&user.password, &req.password)? {
            return Err("Invalid email or password".into());
        }
        
        let mut updated_user = user.clone();
        updated_user.update_last_login();
        self.user_repository.update(&updated_user).await?;
        
        let token_pair = self.auth_service.generate_token(&updated_user).await?;
        
        Ok(AuthResponse {
            token: token_pair.access_token,
            refresh_token: token_pair.refresh_token,
            user_id: updated_user.id,
            name: updated_user.name,
            email: updated_user.email,
            role: updated_user.role,
        })
    }
    
    pub async fn get_user(&self, user_id: Uuid) -> Result<UserResponse, Box<dyn Error>> {
        let user = self.user_repository.find_by_id(user_id).await?
            .ok_or("User not found")?;
            
        Ok(UserResponse {
            id: user.id,
            name: user.name,
            email: user.email,
            role: user.role,
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
            last_login: user.last_login.map(|dt| dt.to_rfc3339()),
        })
    }
    
    pub async fn update_profile(
        &self,
        user_id: Uuid,
        name: Option<String>,
        email: Option<String>,
    ) -> Result<UserResponse, Box<dyn Error>> {
        let mut user = self.user_repository.find_by_id(user_id).await?
            .ok_or("User not found")?;
            
        if let Some(ref new_email) = email {
            if new_email != &user.email {
                if let Ok(Some(_)) = self.user_repository.find_by_email(new_email).await {
                    return Err("Email already in use".into());
                }
            }
        }
        
        user.update_profile(name, email);
        self.user_repository.update(&user).await?;
        
        Ok(UserResponse {
            id: user.id,
            name: user.name,
            email: user.email,
            role: user.role,
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
            last_login: user.last_login.map(|dt| dt.to_rfc3339()),
        })
    }
    
    pub async fn change_password(
        &self,
        user_id: Uuid,
        current_password: String,
        new_password: String,
    ) -> Result<(), Box<dyn Error>> {
        let mut user = self.user_repository.find_by_id(user_id).await?
            .ok_or("User not found")?;
            
        if !self.auth_service.verify_password(&user.password, &current_password)? {
            return Err("Invalid current password".into());
        }
        
        let hashed_password = self.auth_service.hash_password(&new_password)?;
        user.password = hashed_password;
        
        self.user_repository.update(&user).await?;
        
        Ok(())
    }
    
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenPair, Box<dyn Error>> {
        self.auth_service.refresh_access_token(refresh_token).await
    }
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> 
where
    T: Serialize,
{
    pub success: bool,
    pub status_code: u16,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> 
where
    T: Serialize,
{
    pub fn success(message: &str, data: T) -> rocket::serde::json::Json<Self> {
        rocket::serde::json::Json(Self {
            success: true,
            status_code: 200,
            message: message.to_string(),
            data: Some(data),
        })
    }
    
    pub fn error(status_code: u16, message: &str) -> rocket::serde::json::Json<Self> {
        rocket::serde::json::Json(Self {
            success: false,
            status_code,
            message: message.to_string(),
            data: None,
        })
    }
}


#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: Option<UserRole>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
    pub role: UserRole,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub created_at: String,
    pub updated_at: String,
    pub last_login: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub email: Option<String>,
}

// Rocket handler functions
#[post("/auth/register", data = "<req>")]
pub async fn register_handler(
    req: Json<RegisterRequest>,
    user_repository: &State<Arc<dyn UserRepository>>,
    auth_service: &State<Arc<AuthService>>,
) -> Result<Json<ApiResponse<AuthResponse>>, Status> {
    let repo = user_repository.inner();
    let service = auth_service.inner();
    if let Ok(Some(_)) = repo.find_by_email(&req.email).await {
        return Ok(ApiResponse::error(400, "Email already registered"));
    }
    let hashed_password = match service.hash_password(&req.password) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to hash password: {:?}", e);
            return Ok(ApiResponse::error(500, "Failed to hash password"));
        }
    };
    let role = req.role.clone().unwrap_or(UserRole::Attendee);
    let user = User::new(req.name.clone(), req.email.clone(), hashed_password, role);
    if let Err(e) = repo.create(&user).await {
        eprintln!("Failed to create user: {:?}", e);
        return Ok(ApiResponse::error(500, &format!("Failed to create user: {}", e)));
    }
    let token_pair = match service.generate_token(&user).await {
        Ok(tp) => tp,
        Err(_) => return Ok(ApiResponse::error(500, "Failed to generate token")),
    };
    
    Ok(ApiResponse::success("Registration successful", AuthResponse {
        token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        user_id: user.id,
        name: user.name,
        email: user.email,
        role: user.role,
    }))
}

#[post("/auth/login", data = "<req>")]
pub async fn login_handler(
    req: Json<LoginRequest>,
    user_repository: &State<Arc<dyn UserRepository>>,
    auth_service: &State<Arc<AuthService>>,
) -> Result<Json<ApiResponse<AuthResponse>>, Status> {
    let repo = user_repository.inner();
    let service = auth_service.inner();
    let user = match repo.find_by_email(&req.email).await {
        Ok(Some(u)) => u,
        _ => return Ok(ApiResponse::error(400, "Invalid email or password")),
    };
    if !service.verify_password(&user.password, &req.password).unwrap_or(false) {
        return Ok(ApiResponse::error(400, "Invalid email or password"));
    }
    let mut updated_user = user.clone();
    updated_user.update_last_login();
    if let Err(_) = repo.update(&updated_user).await {
        return Ok(ApiResponse::error(500, "Failed to update user login"));
    }
    let token_pair = match service.generate_token(&updated_user).await {
        Ok(tp) => tp,
        Err(_) => return Ok(ApiResponse::error(500, "Failed to generate token")),
    };
    
    Ok(ApiResponse::success("Login successful", AuthResponse {
        token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        user_id: updated_user.id,
        name: updated_user.name,
        email: updated_user.email,
        role: updated_user.role,
    }))
}

#[get("/auth/user/<user_id>")]
pub async fn get_user_handler(
    user_id: &str,
    user_repository: &State<Arc<dyn UserRepository>>,
) -> Result<Json<ApiResponse<UserResponse>>, Status> {
    let uuid = match Uuid::parse_str(user_id) {
        Ok(id) => id,
        Err(_) => return Ok(ApiResponse::error(400, "Invalid UUID format")),
    };
    
    let repo = user_repository.inner();
    let user = match repo.find_by_id(uuid).await {
        Ok(Some(u)) => u,
        _ => return Ok(ApiResponse::error(404, "User not found")),
    };
    Ok(ApiResponse::success("User found", UserResponse {
        id: user.id,
        name: user.name,
        email: user.email,
        role: user.role,
        created_at: user.created_at.to_rfc3339(),
        updated_at: user.updated_at.to_rfc3339(),
        last_login: user.last_login.map(|dt| dt.to_rfc3339()),
    }))
}

#[put("/auth/profile/<user_id>", data = "<req>")]
pub async fn update_profile_handler(
    user_id: &str,
    req: Json<UpdateProfileRequest>,
    user_repository: &State<Arc<dyn UserRepository>>,
) -> Result<Json<ApiResponse<UserResponse>>, Status> {
    let uuid = match Uuid::parse_str(user_id) {
        Ok(id) => id,
        Err(_) => return Ok(ApiResponse::error(400, "Invalid UUID format")),
    };  
    
    let repo = user_repository.inner();
    let mut user = match repo.find_by_id(uuid).await {
        Ok(Some(u)) => u,
        _ => return Ok(ApiResponse::error(404, "User not found")),
    };
    if let Some(ref new_email) = req.email {
        if new_email != &user.email {
            if let Ok(Some(_)) = repo.find_by_email(new_email).await {
                return Ok(ApiResponse::error(400, "Email already in use"));
            }
        }
    }
    user.update_profile(req.name.clone(), req.email.clone());
    if let Err(_) = repo.update(&user).await {
        return Ok(ApiResponse::error(500, "Failed to update user"));
    }
    Ok(ApiResponse::success("Profile updated", UserResponse {
        id: user.id,
        name: user.name,
        email: user.email,
        role: user.role,
        created_at: user.created_at.to_rfc3339(),
        updated_at: user.updated_at.to_rfc3339(),
        last_login: user.last_login.map(|dt| dt.to_rfc3339()),
    }))
}

#[post("/auth/refresh", data = "<req>")]
pub async fn refresh_token_handler(
    req: Json<RefreshTokenRequest>,
    auth_service: &State<Arc<AuthService>>,
) -> Result<Json<ApiResponse<TokenPair>>, Status> {
    let service = auth_service.inner();
    match service.refresh_access_token(&req.refresh_token).await {
        Ok(token_pair) => Ok(ApiResponse::success("Token refreshed", token_pair)),
        Err(_) => Ok(ApiResponse::error(400, "Invalid refresh token")),
    }
}
