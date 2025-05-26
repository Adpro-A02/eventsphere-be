use super::auth_controller::auth_routes;
use crate::model::transaction::Balance;
use crate::model::user::User;
use crate::repository::user::user_repo::UserRepository;
use crate::service::auth::auth_service::AuthService;
use crate::service::transaction::balance_service::BalanceService;
use async_trait::async_trait;
use mockall::mock;
use mockall::predicate::*;
use rocket::http::Status;
use rocket::local::asynchronous::Client;
use serde_json;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct MockBalanceService {
    balances: Mutex<HashMap<Uuid, Balance>>,
}

impl MockBalanceService {
    pub fn new() -> Self {
        Self {
            balances: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl BalanceService for MockBalanceService {
    async fn get_user_balance(
        &self,
        user_id: Uuid,
    ) -> Result<Option<Balance>, Box<dyn Error + Send + Sync>> {
        let balances = self.balances.lock().unwrap();
        Ok(balances.get(&user_id).cloned())
    }

    async fn get_or_create_balance(
        &self,
        user_id: Uuid,
    ) -> Result<Balance, Box<dyn Error + Send + Sync>> {
        let mut balances = self.balances.lock().unwrap();
        if let Some(balance) = balances.get(&user_id) {
            return Ok(balance.clone());
        }

        let balance = Balance::new(user_id);
        balances.insert(user_id, balance.clone());
        Ok(balance)
    }

    async fn add_funds(
        &self,
        user_id: Uuid,
        amount: i64,
    ) -> Result<i64, Box<dyn Error + Send + Sync>> {
        if amount <= 0 {
            return Err("Amount must be positive".into());
        }

        let mut balances = self.balances.lock().unwrap();
        let balance = balances
            .entry(user_id)
            .or_insert_with(|| Balance::new(user_id));

        let new_balance = balance.add_funds(amount).map_err(|e| e.to_string())?;
        Ok(new_balance)
    }

    async fn withdraw_funds(
        &self,
        user_id: Uuid,
        amount: i64,
    ) -> Result<i64, Box<dyn Error + Send + Sync>> {
        if amount <= 0 {
            return Err("Amount must be positive".into());
        }

        let mut balances = self.balances.lock().unwrap();
        let balance = balances
            .entry(user_id)
            .or_insert_with(|| Balance::new(user_id));

        if balance.amount < amount {
            return Err("Insufficient funds".into());
        }

        let new_balance = balance.withdraw(amount).map_err(|e| e.to_string())?;
        Ok(new_balance)
    }

    async fn save_balance(&self, balance: &Balance) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut balances = self.balances.lock().unwrap();
        balances.insert(balance.user_id, balance.clone());
        Ok(())
    }
}

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

fn setup_test_dependencies() -> (
    Arc<dyn UserRepository>,
    Arc<AuthService>,
    Arc<dyn BalanceService + Send + Sync>,
) {
    let user_repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepo::new());
    let auth_service = Arc::new(AuthService::new(
        "test_secret".to_string(),
        "test_refresh_secret".to_string(),
        "test_pepper".to_string(),
    ));
    let balance_service: Arc<dyn BalanceService + Send + Sync> =
        Arc::new(MockBalanceService::new());
    (user_repo, auth_service, balance_service)
}

#[tokio::test]
async fn test_register_success() {
    let (user_repo, auth_service, balance_service) = setup_test_dependencies();

    let rocket = rocket::build()
        .manage(user_repo.clone())
        .manage(auth_service.clone())
        .manage(balance_service.clone())
        .mount("/", auth_routes());

    let client = Client::tracked(rocket)
        .await
        .expect("valid rocket instance");

    let register_json = r#"{
        "name":"Test User",
        "email":"test@example.com",
        "password":"password",
        "role":"Attendee"
    }"#;

    let response = client
        .post("/auth/register")
        .header(rocket::http::ContentType::JSON)
        .body(register_json)
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    let response_body: rocket::serde::json::Value = response.into_json().await.unwrap();
    assert!(response_body.get("success").unwrap().as_bool().unwrap());

    let data = response_body.get("data").unwrap();
    assert_eq!(data.get("name").unwrap().as_str().unwrap(), "Test User");
    assert_eq!(
        data.get("email").unwrap().as_str().unwrap(),
        "test@example.com"
    );
    assert_eq!(data.get("role").unwrap().as_str().unwrap(), "Attendee");
    assert!(!data.get("token").unwrap().as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let (user_repo, auth_service, balance_service) = setup_test_dependencies();

    let rocket = rocket::build()
        .manage(user_repo.clone())
        .manage(auth_service.clone())
        .manage(balance_service.clone())
        .mount("/", auth_routes());

    let client = Client::tracked(rocket)
        .await
        .expect("valid rocket instance");

    let register_json1 = r#"{
        "name":"Test User",
        "email":"duplicate@example.com",
        "password":"password",
        "role":null
    }"#;

    let response1 = client
        .post("/auth/register")
        .header(rocket::http::ContentType::JSON)
        .body(register_json1)
        .dispatch()
        .await;

    assert_eq!(response1.status(), Status::Ok);

    let register_json2 = r#"{
        "name":"Another User",
        "email":"duplicate@example.com",
        "password":"different_password",
        "role":null
    }"#;

    let response2 = client
        .post("/auth/register")
        .header(rocket::http::ContentType::JSON)
        .body(register_json2)
        .dispatch()
        .await;

    assert_eq!(response2.status(), Status::Ok);

    let response_body: rocket::serde::json::Value = response2.into_json().await.unwrap();
    assert!(!response_body.get("success").unwrap().as_bool().unwrap());
    assert_eq!(
        response_body.get("message").unwrap().as_str().unwrap(),
        "Email already registered"
    );
}

#[tokio::test]
async fn test_login_success() {
    let (user_repo, auth_service, balance_service) = setup_test_dependencies();

    let rocket = rocket::build()
        .manage(user_repo.clone())
        .manage(auth_service.clone())
        .manage(balance_service.clone())
        .mount("/", auth_routes());

    let client = Client::tracked(rocket)
        .await
        .expect("valid rocket instance");

    let register_json = r#"{
        "name":"Login Test",
        "email":"login@example.com",
        "password":"correct_password",
        "role":null
    }"#;

    client
        .post("/auth/register")
        .header(rocket::http::ContentType::JSON)
        .body(register_json)
        .dispatch()
        .await;

    let login_json = r#"{
        "email":"login@example.com",
        "password":"correct_password"
    }"#;

    let response = client
        .post("/auth/login")
        .header(rocket::http::ContentType::JSON)
        .body(login_json)
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    let response_body: rocket::serde::json::Value = response.into_json().await.unwrap();
    assert!(response_body.get("success").unwrap().as_bool().unwrap());

    let data = response_body.get("data").unwrap();
    assert_eq!(
        data.get("email").unwrap().as_str().unwrap(),
        "login@example.com"
    );
    assert!(!data.get("token").unwrap().as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_login_invalid_password() {
    let (user_repo, auth_service, balance_service) = setup_test_dependencies();

    let rocket = rocket::build()
        .manage(user_repo.clone())
        .manage(auth_service.clone())
        .manage(balance_service.clone())
        .mount("/", auth_routes());

    let client = Client::tracked(rocket)
        .await
        .expect("valid rocket instance");

    let register_json = r#"{
        "name":"Login Test",
        "email":"login_fail@example.com",
        "password":"correct_password",
        "role":null
    }"#;

    client
        .post("/auth/register")
        .header(rocket::http::ContentType::JSON)
        .body(register_json)
        .dispatch()
        .await;

    let login_json = r#"{
        "email":"login_fail@example.com",
        "password":"wrong_password"
    }"#;

    let response = client
        .post("/auth/login")
        .header(rocket::http::ContentType::JSON)
        .body(login_json)
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    let response_body: rocket::serde::json::Value = response.into_json().await.unwrap();
    assert!(!response_body.get("success").unwrap().as_bool().unwrap());
    assert_eq!(
        response_body.get("message").unwrap().as_str().unwrap(),
        "Invalid email or password"
    );
}

#[tokio::test]
async fn test_get_user() {
    let (user_repo, auth_service, balance_service) = setup_test_dependencies();

    let rocket = rocket::build()
        .manage(user_repo.clone())
        .manage(auth_service.clone())
        .manage(balance_service.clone())
        .mount("/", auth_routes());

    let client = Client::tracked(rocket)
        .await
        .expect("valid rocket instance");

    let register_json = r#"{
        "name":"Get User Test",
        "email":"get_user@example.com",
        "password":"password",
        "role":null
    }"#;

    let register_response = client
        .post("/auth/register")
        .header(rocket::http::ContentType::JSON)
        .body(register_json)
        .dispatch()
        .await;

    let register_body = register_response
        .into_json::<rocket::serde::json::Value>()
        .await
        .unwrap();
    let user_id = register_body["data"]["user_id"].as_str().unwrap();
    let token = register_body["data"]["token"].as_str().unwrap(); // Now get the user using the token
    let response = client
        .get(format!("/auth/user/{}", user_id))
        .header(rocket::http::Header::new(
            "Authorization",
            format!("Bearer {}", token),
        ))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    let response_body = response
        .into_json::<rocket::serde::json::Value>()
        .await
        .unwrap();
    assert!(response_body["success"].as_bool().unwrap());

    let data = &response_body["data"];
    assert_eq!(data["name"].as_str().unwrap(), "Get User Test");
    assert_eq!(data["email"].as_str().unwrap(), "get_user@example.com");
}

#[tokio::test]
async fn test_update_profile() {
    let (user_repo, auth_service, balance_service) = setup_test_dependencies();

    let rocket = rocket::build()
        .manage(user_repo.clone())
        .manage(auth_service.clone())
        .manage(balance_service.clone())
        .mount("/", auth_routes());

    let client = Client::tracked(rocket)
        .await
        .expect("valid rocket instance");

    let register_json = r#"{
        "name":"Update Test",
        "email":"update@example.com",
        "password":"password",
        "role":null
    }"#;

    let register_response = client
        .post("/auth/register")
        .header(rocket::http::ContentType::JSON)
        .body(register_json)
        .dispatch()
        .await;

    let register_body = register_response
        .into_json::<rocket::serde::json::Value>()
        .await
        .unwrap();
    let user_id = register_body["data"]["user_id"].as_str().unwrap();
    let token = register_body["data"]["token"].as_str().unwrap();

    let update_json = r#"{
        "name": "Updated Name",
        "email": "updated@example.com"
    }"#;

    let response = client
        .put(format!("/auth/profile/{}", user_id))
        .header(rocket::http::ContentType::JSON)
        .header(rocket::http::Header::new(
            "Authorization",
            format!("Bearer {}", token),
        ))
        .body(update_json)
        .dispatch()
        .await;
    assert_eq!(response.status(), Status::Ok);

    let response_body = response
        .into_json::<rocket::serde::json::Value>()
        .await
        .unwrap();
    assert!(response_body["success"].as_bool().unwrap());

    let data = &response_body["data"];
    assert_eq!(data["name"].as_str().unwrap(), "Updated Name");
    assert_eq!(data["email"].as_str().unwrap(), "updated@example.com");
}

#[tokio::test]
async fn test_login_with_incorrect_password() {
    let (user_repo, auth_service, balance_service) = setup_test_dependencies();

    let rocket = rocket::build()
        .manage(user_repo.clone())
        .manage(auth_service.clone())
        .manage(balance_service.clone())
        .mount("/", auth_routes());

    let client = Client::tracked(rocket)
        .await
        .expect("valid rocket instance");

    let register_json = r#"{
        "name":"Password Test",
        "email":"password_fail@example.com",
        "password":"correct_password",
        "role":null
    }"#;

    client
        .post("/auth/register")
        .header(rocket::http::ContentType::JSON)
        .body(register_json)
        .dispatch()
        .await;

    let login_json = r#"{
        "email":"password_fail@example.com",
        "password":"wrong_password"
    }"#;

    let response = client
        .post("/auth/login")
        .header(rocket::http::ContentType::JSON)
        .body(login_json)
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    let response_body = response
        .into_json::<rocket::serde::json::Value>()
        .await
        .unwrap();
    assert!(!response_body["success"].as_bool().unwrap());
    assert_eq!(
        response_body["message"].as_str().unwrap(),
        "Invalid email or password"
    );
}

#[tokio::test]
async fn test_register_route() {
    let (user_repo, auth_service, balance_service) = setup_test_dependencies();
    let rocket = rocket::build()
        .manage(user_repo.clone())
        .manage(auth_service.clone())
        .manage(balance_service.clone())
        .mount("/", auth_routes());
    let client = Client::tracked(rocket)
        .await
        .expect("valid rocket instance");

    let register_json = r#"{
        "name":"API Test User",
        "email":"api_test@example.com",
        "password":"password",
        "role":null
    }"#;

    let response = client
        .post("/auth/register")
        .header(rocket::http::ContentType::JSON)
        .body(register_json)
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().await.unwrap();
    assert!(body.contains("success"));
    assert!(body.contains("api_test@example.com"));
}

#[tokio::test]
async fn test_login_route() {
    let (user_repo, auth_service, balance_service) = setup_test_dependencies();
    let rocket = rocket::build()
        .manage(user_repo.clone())
        .manage(auth_service.clone())
        .manage(balance_service.clone())
        .mount("/", auth_routes());
    let client = Client::tracked(rocket)
        .await
        .expect("valid rocket instance");

    let register_json = r#"{
        "name":"Login API Test",
        "email":"login_api@example.com",
        "password":"correct_password",
        "role":null
    }"#;

    client
        .post("/auth/register")
        .header(rocket::http::ContentType::JSON)
        .body(register_json)
        .dispatch()
        .await;

    let login_json = r#"{
        "email":"login_api@example.com",
        "password":"correct_password"
    }"#;

    let response = client
        .post("/auth/login")
        .header(rocket::http::ContentType::JSON)
        .body(login_json)
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().await.unwrap();
    assert!(body.contains("success"));
    assert!(body.contains("login_api@example.com"));
}

#[tokio::test]
async fn test_get_user_route() {
    let (user_repo, auth_service, balance_service) = setup_test_dependencies();
    let rocket = rocket::build()
        .manage(user_repo.clone())
        .manage(auth_service.clone())
        .manage(balance_service.clone())
        .mount("/", auth_routes());
    let client = Client::tracked(rocket)
        .await
        .expect("valid rocket instance");

    let register_json = r#"{
        "name":"User Route Test",
        "email":"user_route@example.com",
        "password":"correct_password",
        "role":null
    }"#;

    let register_response = client
        .post("/auth/register")
        .header(rocket::http::ContentType::JSON)
        .body(register_json)
        .dispatch()
        .await;

    let register_body = register_response
        .into_json::<serde_json::Value>()
        .await
        .unwrap();

    let user_id = register_body["data"]["user_id"].as_str().unwrap();
    let token = register_body["data"]["token"].as_str().unwrap();

    let response = client
        .get(format!("/auth/user/{}", user_id))
        .header(rocket::http::Header::new(
            "Authorization",
            format!("Bearer {}", token),
        ))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    let response_body = response.into_json::<serde_json::Value>().await.unwrap();
    assert!(response_body["success"].as_bool().unwrap());
    assert!(
        response_body["data"]["email"]
            .as_str()
            .unwrap()
            .contains("user_route@example.com")
    );
}

#[tokio::test]
async fn test_refresh_token_route() {
    let (user_repo, auth_service, balance_service) = setup_test_dependencies();
    let rocket = rocket::build()
        .manage(user_repo.clone())
        .manage(auth_service.clone())
        .manage(balance_service.clone())
        .mount("/", auth_routes());
    let client = Client::tracked(rocket)
        .await
        .expect("valid rocket instance");

    let register_json = r#"{
        "name":"Refresh Token Test",
        "email":"refresh_test@example.com",
        "password":"password123",
        "role":null
    }"#;

    let response = client
        .post("/auth/register")
        .header(rocket::http::ContentType::JSON)
        .body(register_json)
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().await.unwrap();

    let response_json: serde_json::Value =
        serde_json::from_str(&body).expect("Invalid JSON response");

    let refresh_token = response_json["data"]["refresh_token"]
        .as_str()
        .expect("Refresh token not found in response");

    let refresh_json = format!(r#"{{"refresh_token":"{}"}}"#, refresh_token);

    let response = client
        .post("/auth/refresh")
        .header(rocket::http::ContentType::JSON)
        .body(refresh_json)
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().await.unwrap();
    assert!(body.contains("success"));
    assert!(
        body.contains("access_token"),
        "Response body should contain access_token: {}",
        body
    );
    assert!(body.contains("refresh_token"));
}

#[tokio::test]
async fn test_refresh_token() {
    let (user_repo, auth_service, balance_service) = setup_test_dependencies();
    let rocket = rocket::build()
        .manage(user_repo.clone())
        .manage(auth_service.clone())
        .manage(balance_service.clone())
        .mount("/", auth_routes());
    let client = Client::tracked(rocket)
        .await
        .expect("valid rocket instance");

    let register_json = r#"{
        "name":"Refresh Token Test",
        "email":"refresh_direct@example.com",
        "password":"password123",
        "role":null
    }"#;

    let response = client
        .post("/auth/register")
        .header(rocket::http::ContentType::JSON)
        .body(register_json)
        .dispatch()
        .await;

    let register_body = response
        .into_json::<rocket::serde::json::Value>()
        .await
        .unwrap();
    let refresh_token = register_body["data"]["refresh_token"].as_str().unwrap();

    let refresh_json = format!(r#"{{"refresh_token":"{}"}}"#, refresh_token);
    let response = client
        .post("/auth/refresh")
        .header(rocket::http::ContentType::JSON)
        .body(refresh_json)
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    let refresh_body = response
        .into_json::<rocket::serde::json::Value>()
        .await
        .unwrap();
    assert!(refresh_body["success"].as_bool().unwrap());
    assert!(
        !refresh_body["data"]["access_token"]
            .as_str()
            .unwrap()
            .is_empty()
    );
    assert!(
        !refresh_body["data"]["refresh_token"]
            .as_str()
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn test_refresh_token_invalid() {
    let (user_repo, auth_service, balance_service) = setup_test_dependencies();
    let rocket = rocket::build()
        .manage(user_repo.clone())
        .manage(auth_service.clone())
        .manage(balance_service.clone())
        .mount("/", auth_routes());
    let client = Client::tracked(rocket)
        .await
        .expect("valid rocket instance");

    let refresh_json = r#"{"refresh_token":"invalid_token_that_doesnt_exist"}"#;
    let response = client
        .post("/auth/refresh")
        .header(rocket::http::ContentType::JSON)
        .body(refresh_json)
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    let refresh_body = response
        .into_json::<rocket::serde::json::Value>()
        .await
        .unwrap();
    assert!(!refresh_body["success"].as_bool().unwrap());
    assert_eq!(
        refresh_body["message"].as_str().unwrap(),
        "Invalid refresh token"
    );
}

#[tokio::test]
async fn test_balance_created_during_registration() {
    let (user_repo, auth_service, balance_service) = setup_test_dependencies();

    let rocket = rocket::build()
        .manage(user_repo.clone())
        .manage(auth_service.clone())
        .manage(balance_service.clone())
        .mount("/", auth_routes());

    let client = Client::tracked(rocket)
        .await
        .expect("valid rocket instance");

    let register_json = r#"{
        "name":"Balance Test User",
        "email":"balance_test@example.com",
        "password":"password",
        "role":"Attendee"
    }"#;

    let response = client
        .post("/auth/register")
        .header(rocket::http::ContentType::JSON)
        .body(register_json)
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    let response_body: rocket::serde::json::Value = response.into_json().await.unwrap();
    let user_id = response_body["data"]["user_id"].as_str().unwrap();
    let user_uuid = Uuid::parse_str(user_id).unwrap();

    // Now check if a balance was created for this user
    let balance_result = balance_service.get_user_balance(user_uuid).await;

    assert!(balance_result.is_ok(), "Should be able to retrieve balance");
    let balance_option = balance_result.unwrap();
    assert!(
        balance_option.is_some(),
        "Balance should have been created during registration"
    );
    let balance = balance_option.unwrap();
    assert_eq!(balance.user_id, user_uuid);
    assert_eq!(balance.amount, 0, "Initial balance should be zero");
}

#[tokio::test]
async fn test_retrieve_user_balance() {
    let (user_repo, auth_service, balance_service) = setup_test_dependencies();

    let rocket = rocket::build()
        .manage(user_repo.clone())
        .manage(auth_service.clone())
        .manage(balance_service.clone())
        .mount("/", auth_routes());

    let client = Client::tracked(rocket)
        .await
        .expect("valid rocket instance");

    // First register a user
    let register_json = r#"{
        "name":"Balance Retrieval Test",
        "email":"balance_retrieval@example.com",
        "password":"password",
        "role":"Attendee"
    }"#;

    let response = client
        .post("/auth/register")
        .header(rocket::http::ContentType::JSON)
        .body(register_json)
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let response_body: rocket::serde::json::Value = response.into_json().await.unwrap();
    let user_id = response_body["data"]["user_id"].as_str().unwrap();
    // We're not using the token in this test, but it's here for documentation
    let _token = response_body["data"]["token"].as_str().unwrap();

    // Directly check the balance exists via balance service
    let user_uuid = Uuid::parse_str(user_id).unwrap();
    let balance_option = balance_service.get_user_balance(user_uuid).await.unwrap();
    assert!(balance_option.is_some());

    // Verify that the balance was created with an initial amount of 0
    let balance = balance_option.unwrap();
    assert_eq!(balance.amount, 0);
}
