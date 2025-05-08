#[cfg(test)]
mod tests {
    use super::super::auth_service::AuthService;
    use crate::model::user::{User, UserRole};
    use uuid::Uuid;

    #[test]
    fn test_hash_password() {
        let auth_service = AuthService::new("test_secret".to_string(), "test_refresh_secret".to_string(), "test_pepper".to_string());
        let password = "test_password";

        let hash = auth_service.hash_password(password).expect("Failed to hash password");
        assert!(!hash.is_empty(), "Hash should not be empty");
    }

    #[test]
    fn test_verify_password() {
        let auth_service = AuthService::new("test_secret".to_string(), "test_refresh_secret".to_string(), "test_pepper".to_string());
        let password = "test_password";

        let hash = auth_service.hash_password(password).expect("Failed to hash password");
        let is_valid = auth_service
            .verify_password(&hash, password)
            .expect("Failed to verify password");

        assert!(is_valid, "Password verification should succeed");
    }

    #[test]
    fn test_generate_token() {
        let auth_service = AuthService::new("test_secret".to_string(), "test_refresh_secret".to_string(), "test_pepper".to_string());
        let user = User {
            id: Uuid::new_v4(),
            role: UserRole::Admin,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password: "test_password_hash".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_login: None,
        };

        let token_pair = auth_service
            .generate_token(&user)
            .expect("Failed to generate token");
        let token = &token_pair.access_token;

        assert!(!token.is_empty(), "Token should not be empty");
    }
}
