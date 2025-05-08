use crate::model::user::{User, UserRole};
use crate::repository::user::user_repo::UserRepository;
use crate::service::auth::auth_service::{AuthService, TokenPair};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;
use warp::{Filter, Rejection, Reply};

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

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            status: "success".to_string(),
            message: None,
            data: Some(data),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            status: "error".to_string(),
            message: Some(message),
            data: None,
        }
    }
}

#[derive(Debug)]
pub struct ApiError {
    pub message: String,
}

impl ApiError {
    pub fn from_error<E: std::fmt::Display>(err: E) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

impl warp::reject::Reject for ApiError {}

#[derive(Clone)]
pub struct AuthController {
    user_repository: Arc<dyn UserRepository>,
    auth_service: Arc<AuthService>,
}

impl AuthController {
    pub fn new(user_repository: Arc<dyn UserRepository>, auth_service: Arc<AuthService>) -> Self {
        Self {
            user_repository,
            auth_service,
        }
    }

    pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse, Box<dyn Error>> {
        if let Some(_) = self.user_repository.find_by_email(&req.email).await? {
            return Err("User with this email already exists".into());
        }

        let hashed_password = self.auth_service.hash_password(&req.password)?;

        let role = req.role.unwrap_or(UserRole::Attendee);
        let user = User::new(req.name, req.email, hashed_password, role);

        self.user_repository.create(&user).await?;

        let token_pair = self.auth_service.generate_token(&user)?;
        let token = token_pair.access_token;

        Ok(AuthResponse {
            token,
            user_id: user.id,
            name: user.name,
            email: user.email,
            role: user.role,
        })
    }

    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse, Box<dyn Error>> {
        let user = match self.user_repository.find_by_email(&req.email).await? {
            Some(user) => user,
            None => return Err("Invalid email or password".into()),
        };

        if !self.auth_service.verify_password(&user.password, &req.password)? {
            return Err("Invalid email or password".into());
        }

        let mut updated_user = user.clone();
        updated_user.update_last_login();
        self.user_repository.update(&updated_user).await?;

        let token_pair = self.auth_service.generate_token(&updated_user)?;
        let token = token_pair.access_token;

        Ok(AuthResponse {
            token,
            user_id: updated_user.id,
            name: updated_user.name,
            email: updated_user.email,
            role: updated_user.role,
        })
    }

    pub async fn get_user(&self, user_id: Uuid) -> Result<UserResponse, Box<dyn Error>> {
        let user = match self.user_repository.find_by_id(user_id).await? {
            Some(user) => user,
            None => return Err("User not found".into()),
        };

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
        let mut user = match self.user_repository.find_by_id(user_id).await? {
            Some(user) => user,
            None => return Err("User not found".into()),
        };

        if let Some(ref new_email) = email {
            if new_email != &user.email {
                if let Some(_) = self.user_repository.find_by_email(new_email).await? {
                    return Err("Email is already in use".into());
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
        old_password: String,
        new_password: String,
    ) -> Result<(), Box<dyn Error>> {
        let mut user = match self.user_repository.find_by_id(user_id).await? {
            Some(user) => user,
            None => return Err("User not found".into()),
        };

        if !self.auth_service.verify_password(&user.password, &old_password)? {
            return Err("Invalid current password".into());
        }

        let hashed_new_password = self.auth_service.hash_password(&new_password)?;

        user.update_password(hashed_new_password);

        self.user_repository.update(&user).await?;

        Ok(())
    }

    pub async fn refresh_token(
        &self,
        refresh_token: String,
    ) -> Result<TokenPair, Box<dyn Error>> {
        self.auth_service.refresh_access_token(&refresh_token)
    }

    pub fn routes(controller: Arc<AuthController>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let register = warp::path!("auth" / "register")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_auth_controller(controller.clone()))
            .and_then(Self::register_handler);

        let login = warp::path!("auth" / "login")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_auth_controller(controller.clone()))
            .and_then(Self::login_handler);

        let get_user = warp::path!("auth" / "user")
            .and(warp::get())
            .and(warp::header::<String>("authorization"))
            .and(with_auth_controller(controller.clone()))
            .and_then(Self::get_user_handler);

        let update_profile = warp::path!("auth" / "profile")
            .and(warp::put())
            .and(warp::body::json())
            .and(warp::header::<String>("authorization"))
            .and(with_auth_controller(controller.clone()))
            .and_then(Self::update_profile_handler);

        let change_password = warp::path!("auth" / "password")
            .and(warp::put())
            .and(warp::body::json())
            .and(warp::header::<String>("authorization"))
            .and(with_auth_controller(controller.clone()))
            .and_then(Self::change_password_handler);

        let refresh_token = warp::path!("auth" / "refresh")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_auth_controller(controller.clone()))
            .and_then(Self::refresh_token_handler);

        register
            .or(login)
            .or(get_user)
            .or(update_profile)
            .or(change_password)
            .or(refresh_token)
    }

    fn handle_result<T: Serialize>(result: Result<T, Box<dyn Error>>) -> Result<impl Reply, Rejection> {
        match result {
            Ok(data) => Ok(warp::reply::json(&ApiResponse::success(data))),
            Err(e) => Err(warp::reject::custom(ApiError::from_error(e))),
        }
    }

    async fn register_handler(
        req: RegisterRequest,
        controller: Arc<AuthController>,
    ) -> Result<impl Reply, Rejection> {
        Self::handle_result(controller.register(req).await)
    }

    async fn login_handler(
        req: LoginRequest,
        controller: Arc<AuthController>,
    ) -> Result<impl Reply, Rejection> {
        Self::handle_result(controller.login(req).await)
    }

    async fn get_user_handler(
        auth_header: String,
        controller: Arc<AuthController>,
    ) -> Result<impl Reply, Rejection> {
        let user_id = Self::extract_user_id_from_token(auth_header, &controller)?;
        Self::handle_result(controller.get_user(user_id).await)
    }

    async fn update_profile_handler(
        req: UpdateProfileRequest,
        auth_header: String,
        controller: Arc<AuthController>,
    ) -> Result<impl Reply, Rejection> {
        let user_id = Self::extract_user_id_from_token(auth_header, &controller)?;
        Self::handle_result(controller.update_profile(user_id, req.name, req.email).await)
    }

    async fn change_password_handler(
        req: ChangePasswordRequest,
        auth_header: String,
        controller: Arc<AuthController>,
    ) -> Result<impl Reply, Rejection> {
        let user_id = Self::extract_user_id_from_token(auth_header, &controller)?;
        Self::handle_result(controller.change_password(user_id, req.old_password, req.new_password).await)
    }

    async fn refresh_token_handler(
        req: RefreshTokenRequest,
        controller: Arc<AuthController>,
    ) -> Result<impl Reply, Rejection> {
        Self::handle_result(controller.refresh_token(req.refresh_token).await)
    }

    fn extract_user_id_from_token(
        auth_header: String,
        controller: &AuthController,
    ) -> Result<Uuid, Rejection> {
        let token = auth_header.replace("Bearer ", "");

        match controller.auth_service.verify_token(&token) {
            Ok(user_id) => Ok(user_id),
            Err(_) => Err(warp::reject::custom(ApiError {
                message: "Invalid or expired token".to_string(),
            })),
        }
    }
}

fn with_auth_controller(
    controller: Arc<AuthController>,
) -> impl Filter<Extract = (Arc<AuthController>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || controller.clone())
}