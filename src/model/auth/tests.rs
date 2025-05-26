#[cfg(test)]
mod token_tests {
    use chrono::Utc;
    use uuid::Uuid;
    use crate::model::auth::RefreshToken;

    #[test]
    fn test_create_refresh_token() {
        let user_id = Uuid::new_v4();
        let token_str = "test-token-string";
        let expires_in_days = 7;
        
        let token = RefreshToken::new(user_id, token_str.to_string(), expires_in_days);
        
        assert_eq!(token.user_id, user_id);
        assert_eq!(token.token, token_str);
        assert!(!token.is_revoked);
        
        let expected_expiry = token.created_at + chrono::Duration::days(expires_in_days);
        assert_eq!(token.expires_at, expected_expiry);
    }

    #[test]
    fn test_token_validity() {
        let user_id = Uuid::new_v4();
        
        let valid_token = RefreshToken {
            id: Uuid::new_v4(),
            user_id,
            token: "valid-token".to_string(),
            expires_at: Utc::now() + chrono::Duration::days(1),
            is_revoked: false,
            created_at: Utc::now(),
        };
        assert!(valid_token.is_valid());
        
        let expired_token = RefreshToken {
            id: Uuid::new_v4(),
            user_id,
            token: "expired-token".to_string(),
            expires_at: Utc::now() - chrono::Duration::hours(1),
            is_revoked: false,
            created_at: Utc::now() - chrono::Duration::days(7),
        };
        assert!(!expired_token.is_valid());
        
        let revoked_token = RefreshToken {
            id: Uuid::new_v4(),
            user_id,
            token: "revoked-token".to_string(),
            expires_at: Utc::now() + chrono::Duration::days(1),
            is_revoked: true,
            created_at: Utc::now(),
        };
        assert!(!revoked_token.is_valid());
        
        let expired_revoked_token = RefreshToken {
            id: Uuid::new_v4(),
            user_id,
            token: "expired-revoked-token".to_string(),
            expires_at: Utc::now() - chrono::Duration::hours(1),
            is_revoked: true,
            created_at: Utc::now() - chrono::Duration::days(7),
        };
        assert!(!expired_revoked_token.is_valid());
    }
}