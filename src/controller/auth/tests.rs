use super::auth_controller::{
    AuthController, LoginRequest, RegisterRequest,
};
use crate::model::user::{User, UserRole};
use crate::repository::user::user_repo::UserRepository;
use crate::service::auth::auth_service::AuthService;
use async_trait::async_trait;
use mockall::mock;
use mockall::predicate::*;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use warp::{http::StatusCode, test::request};

mock! {
    pub UserRepo {}

    #[async_trait]
    impl UserRepository for UserRepo {
        async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Box<dyn Error>>;
        async fn find_by_email(&self, email: &str) -> Result<Option<User>, Box<dyn Error>>;
        async fn create(&self, user: &User) -> Result<(), Box<dyn Error>>;
        async fn update(&self, user: &User) -> Result<(), Box<dyn Error>>;
        async fn delete(&self, id: Uuid) -> Result<(), Box<dyn Error>>;
        async fn find_all(&self) -> Result<Vec<User>, Box<dyn Error>>;
    }
}

struct InMemoryUserRepo {
    users: Mutex<HashMap<Uuid, User>>,
    users_by_email: Mutex<HashMap<String, Uuid>>,
}

impl InMemoryUserRepo {
    fn new() -> Self {
        Self {
            users: Mutex::new(HashMap::new()),
            users_by_email: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepo {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Box<dyn Error>> {
        let users = self.users.lock().unwrap();
        Ok(users.get(&id).cloned())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, Box<dyn Error>> {
        let users_by_email = self.users_by_email.lock().unwrap();
        let users = self.users.lock().unwrap();

        match users_by_email.get(email) {
            Some(user_id) => Ok(users.get(user_id).cloned()),
            None => Ok(None),
        }
    }

    async fn create(&self, user: &User) -> Result<(), Box<dyn Error>> {
        let mut users = self.users.lock().unwrap();
        let mut users_by_email = self.users_by_email.lock().unwrap();

        users.insert(user.id, user.clone());
        users_by_email.insert(user.email.clone(), user.id);

        Ok(())
    }

    async fn update(&self, user: &User) -> Result<(), Box<dyn Error>> {
        let mut users = self.users.lock().unwrap();
        let mut users_by_email = self.users_by_email.lock().unwrap();

        if let Some(existing_user) = users.get(&user.id) {
            if existing_user.email != user.email {
                users_by_email.remove(&existing_user.email);
                users_by_email.insert(user.email.clone(), user.id);
            }
        }

        users.insert(user.id, user.clone());

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Box<dyn Error>> {
        let mut users = self.users.lock().unwrap();
        let mut users_by_email = self.users_by_email.lock().unwrap();

        if let Some(user) = users.remove(&id) {
            users_by_email.remove(&user.email);
            Ok(())
        } else {
            Err("User not found".into())
        }
    }

    async fn find_all(&self) -> Result<Vec<User>, Box<dyn Error>> {
        let users = self.users.lock().unwrap();
        Ok(users.values().cloned().collect())
    }
}

fn setup_controller() -> Arc<AuthController> {
    let user_repo = Arc::new(InMemoryUserRepo::new());
    let auth_service = Arc::new(AuthService::new("test_secret".to_string(), "test_refresh_secret".to_string(), "test_pepper".to_string()));
    Arc::new(AuthController::new(user_repo, auth_service))
}

#[tokio::test]
async fn test_register_success() {
    let controller = setup_controller();
    let req = RegisterRequest {
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        password: "password".to_string(),
        role: Some(UserRole::Attendee),
    };

    let result = controller.register(req).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.name, "Test User");
    assert_eq!(response.email, "test@example.com");
    assert_eq!(response.role, UserRole::Attendee);
    assert!(!response.token.is_empty());
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let controller = setup_controller();

    let req1 = RegisterRequest {
        name: "Test User".to_string(),
        email: "duplicate@example.com".to_string(),
        password: "password".to_string(),
        role: None,
    };
    let _ = controller.register(req1).await.unwrap();

    let req2 = RegisterRequest {
        name: "Another User".to_string(),
        email: "duplicate@example.com".to_string(),
        password: "different_password".to_string(),
        role: None,
    };
    let result = controller.register(req2).await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "User with this email already exists"
    );
}

#[tokio::test]
async fn test_login_success() {
    let controller = setup_controller();

    let register_req = RegisterRequest {
        name: "Login Test".to_string(),
        email: "login@example.com".to_string(),
        password: "correct_password".to_string(),
        role: None,
    };
    let _ = controller.register(register_req).await.unwrap();

    let login_req = LoginRequest {
        email: "login@example.com".to_string(),
        password: "correct_password".to_string(),
    };

    let result = controller.login(login_req).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.email, "login@example.com");
    assert!(!response.token.is_empty());
}

#[tokio::test]
async fn test_login_invalid_password() {
    let controller = setup_controller();

    let register_req = RegisterRequest {
        name: "Login Test".to_string(),
        email: "login_fail@example.com".to_string(),
        password: "correct_password".to_string(),
        role: None,
    };
    let _ = controller.register(register_req).await.unwrap();

    let login_req = LoginRequest {
        email: "login_fail@example.com".to_string(),
        password: "wrong_password".to_string(),
    };

    let result = controller.login(login_req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Invalid email or password");
}

#[tokio::test]
async fn test_get_user() {
    let controller = setup_controller();

    let register_req = RegisterRequest {
        name: "Get User Test".to_string(),
        email: "get_user@example.com".to_string(),
        password: "password".to_string(),
        role: None,
    };
    let register_response = controller.register(register_req).await.unwrap();

    let result = controller.get_user(register_response.user_id).await;
    assert!(result.is_ok());

    let user = result.unwrap();
    assert_eq!(user.name, "Get User Test");
    assert_eq!(user.email, "get_user@example.com");
}

#[tokio::test]
async fn test_update_profile() {
    let controller = setup_controller();

    let register_req = RegisterRequest {
        name: "Update Test".to_string(),
        email: "update@example.com".to_string(),
        password: "password".to_string(),
        role: None,
    };
    let register_response = controller.register(register_req).await.unwrap();

    let result = controller
        .update_profile(
            register_response.user_id,
            Some("Updated Name".to_string()),
            Some("updated@example.com".to_string()),
        )
        .await;

    assert!(result.is_ok());

    let updated_user = result.unwrap();
    assert_eq!(updated_user.name, "Updated Name");
    assert_eq!(updated_user.email, "updated@example.com");
}

#[tokio::test]
async fn test_change_password() {
    let controller = setup_controller();

    let register_req = RegisterRequest {
        name: "Password Test".to_string(),
        email: "password@example.com".to_string(),
        password: "correct_password".to_string(),
        role: None,
    };
    let register_response = controller.register(register_req).await.unwrap();

    let result = controller
        .change_password(
            register_response.user_id,
            "correct_password".to_string(),
            "new_password".to_string(),
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_change_password_invalid_current() {
    let controller = setup_controller();

    let register_req = RegisterRequest {
        name: "Password Test".to_string(),
        email: "password_fail@example.com".to_string(),
        password: "correct_password".to_string(),
        role: None,
    };
    let register_response = controller.register(register_req).await.unwrap();

    let result = controller
        .change_password(
            register_response.user_id,
            "wrong_password".to_string(),
            "new_password".to_string(),
        )
        .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Invalid current password");
}

#[tokio::test]
async fn test_register_route() {
    let controller = setup_controller();
    let api = AuthController::routes(controller);

    let register_json = r#"{
        "name":"API Test User",
        "email":"api_test@example.com",
        "password":"password",
        "role":null
    }"#;

    let resp = request()
        .method("POST")
        .path("/auth/register")
        .body(register_json)
        .reply(&api)
        .await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = String::from_utf8(resp.body().to_vec()).unwrap();
    assert!(body.contains("success"));
    assert!(body.contains("api_test@example.com"));
}

#[tokio::test]
async fn test_login_route() {
    let controller = setup_controller();
    let api = AuthController::routes(controller);

    let register_json = r#"{
        "name":"Login API Test",
        "email":"login_api@example.com",
        "password":"correct_password",
        "role":null
    }"#;
    let _ = request()
        .method("POST")
        .path("/auth/register")
        .body(register_json)
        .reply(&api)
        .await;

    let login_json = r#"{
        "email":"login_api@example.com",
        "password":"correct_password"
    }"#;
    let resp = request()
        .method("POST")
        .path("/auth/login")
        .body(login_json)
        .reply(&api)
        .await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = String::from_utf8(resp.body().to_vec()).unwrap();
    assert!(body.contains("success"));
    assert!(body.contains("login_api@example.com"));
}

#[tokio::test]
async fn test_get_user_route() {
    let controller = setup_controller();
    let api = AuthController::routes(controller);

    let register_json = r#"{
        "name":"User Route Test",
        "email":"user_route@example.com",
        "password":"correct_password",
        "role":null
    }"#;
    let register_resp = request()
        .method("POST")
        .path("/auth/register")
        .body(register_json)
        .reply(&api)
        .await;

    let register_body = String::from_utf8(register_resp.body().to_vec()).unwrap();

    let token_start = register_body.find("token\":\"").unwrap() + 8;
    let token_end = register_body[token_start..].find("\"").unwrap() + token_start;
    let token = &register_body[token_start..token_end];

    let resp = request()
        .method("GET")
        .path("/auth/user")
        .header("authorization", format!("Bearer {}", token))
        .reply(&api)
        .await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = String::from_utf8(resp.body().to_vec()).unwrap();
    assert!(body.contains("success"));
    assert!(body.contains("user_route@example.com"));
}
