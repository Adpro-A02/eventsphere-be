#[cfg(test)]
mod tests {
    use super::super::auth_service::AuthService;
    use crate::model::auth::RefreshToken;
    use crate::model::user::{User, UserRole};
    use crate::repository::auth::token_repo::TokenRepository;
    use crate::repository::user::user_repo::UserRepository;
    use async_trait::async_trait;
    use chrono::Utc;
    use mockall::mock;
    use mockall::predicate::*;
    use std::error::Error;
    use std::sync::Arc;
    use uuid::Uuid;
    
    mock! {
        pub TokenRepo {}
        #[async_trait]
        impl TokenRepository for TokenRepo {
            async fn create(&self, token: &RefreshToken) -> Result<(), Box<dyn Error>>;
            async fn find_by_token(&self, token: &str) -> Result<Option<RefreshToken>, Box<dyn Error>>;
            async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<RefreshToken>, Box<dyn Error>>;
            async fn revoke(&self, token_id: Uuid) -> Result<(), Box<dyn Error>>;
            async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<(), Box<dyn Error>>;
        }
    }

    mock! {
        pub UserRepo {}
        #[async_trait]
        impl UserRepository for UserRepo {
            async fn find_by_email(&self, email: &str) -> Result<Option<User>, Box<dyn Error>>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Box<dyn Error>>;
            async fn create(&self, user: &User) -> Result<(), Box<dyn Error>>;
            async fn update(&self, user: &User) -> Result<(), Box<dyn Error>>;
            async fn delete(&self, id: Uuid) -> Result<(), Box<dyn Error>>;
            async fn find_all(&self) -> Result<Vec<User>, Box<dyn Error>>;
        }
    }    
    
    #[test]
    fn test_hash_password() {
        let auth_service = AuthService::new("test_secret".to_string(), "test_refresh_secret".to_string(), "test_pepper".to_string());
        let password = "test_password";

        let hash = auth_service.hash_password(password).expect("Failed to hash password");
        assert!(!hash.is_empty(), "Hash should not be empty");
        
        let hash2 = auth_service.hash_password(password).expect("Failed to hash password");
        assert_ne!(hash, hash2, "Hashes should be different due to salt");
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
        
        let is_invalid = auth_service
            .verify_password(&hash, "wrong_password")
            .expect("Failed to verify password");
        assert!(!is_invalid, "Wrong password should fail validation");
        
        let auth_service2 = AuthService::new("test_secret".to_string(), "test_refresh_secret".to_string(), "different_pepper".to_string());
        let is_invalid2 = auth_service2
            .verify_password(&hash, password)
            .expect("Failed to verify password");
        assert!(!is_invalid2, "Password with wrong pepper should fail validation");
    }

    #[tokio::test]
    async fn test_generate_token() {
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
            .await
            .expect("Failed to generate token");
        
        assert!(!token_pair.access_token.is_empty(), "Access token should not be empty");
        assert!(!token_pair.refresh_token.is_empty(), "Refresh token should not be empty");
        assert!(token_pair.expires_in > 0, "Token should have expiration time");
    }
    
    #[tokio::test]
    async fn test_verify_token() {
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
            .await
            .expect("Failed to generate token");
            
        let user_id = auth_service
            .verify_token(&token_pair.access_token)
            .expect("Failed to verify token");
            
        assert_eq!(user_id, user.id, "Token should verify to correct user ID");
        let verify_result = auth_service.verify_token("invalid-token");
        assert!(verify_result.is_err(), "Invalid token should fail verification");
    }    #[tokio::test]
    
    async fn test_refresh_access_token_with_repository() {
        let mut mock_token_repo = MockTokenRepo::new();
        let mut mock_user_repo = MockUserRepo::new();
        let user_id = Uuid::new_v4();
        let refresh_token_str = "valid-refresh-token";
        
        let user = User {
            id: user_id,
            role: UserRole::Admin,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password: "test_password_hash".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_login: None,
        };
        
        let refresh_token = RefreshToken {
            id: Uuid::new_v4(),
            user_id,
            token: refresh_token_str.to_string(),
            expires_at: Utc::now() + chrono::Duration::days(7),
            is_revoked: false,
            created_at: Utc::now(),
        };
        
        mock_token_repo.expect_find_by_token()
            .with(eq(refresh_token_str))
            .returning(move |_| Ok(Some(refresh_token.clone())));
            
        mock_user_repo.expect_find_by_id()
            .with(eq(user_id))
            .returning(move |_| Ok(Some(user.clone())));
        
        mock_token_repo.expect_create()
            .returning(|_| Ok(()));
            
        let auth_service = AuthService::new("test_secret".to_string(), "test_refresh_secret".to_string(), "test_pepper".to_string())
            .with_token_repository(Arc::new(mock_token_repo))
            .with_user_repository(Arc::new(mock_user_repo));
            
        let token_pair = auth_service
            .refresh_access_token(refresh_token_str)
            .await
            .expect("Failed to refresh token");
            
        assert!(!token_pair.access_token.is_empty(), "New access token should not be empty");
        assert!(!token_pair.refresh_token.is_empty(), "New refresh token should not be empty");
    }
    
    #[tokio::test]
    async fn test_refresh_with_invalid_token() {
        let mut mock_token_repo = MockTokenRepo::new();
        
        mock_token_repo.expect_find_by_token()
            .with(eq("invalid-token"))
            .returning(move |_| Ok(None));
            
        let auth_service = AuthService::new("test_secret".to_string(), "test_refresh_secret".to_string(), "test_pepper".to_string())
            .with_token_repository(Arc::new(mock_token_repo));
            
        let result = auth_service.refresh_access_token("invalid-token").await;
        assert!(result.is_err(), "Invalid token should fail refresh");
    }
    
    #[tokio::test]
    async fn test_logout() {
        let mut mock_token_repo = MockTokenRepo::new();
        let user_id = Uuid::new_v4();
        
        mock_token_repo.expect_revoke_all_for_user()
            .with(eq(user_id))
            .returning(|_| Ok(()));
            
        let auth_service = AuthService::new("test_secret".to_string(), "test_refresh_secret".to_string(), "test_pepper".to_string())
            .with_token_repository(Arc::new(mock_token_repo));
            
        let result = auth_service.logout(user_id).await;
        assert!(result.is_ok(), "Logout should succeed");
    }
}
