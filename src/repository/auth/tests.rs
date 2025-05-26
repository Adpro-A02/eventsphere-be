#[cfg(test)]
mod token_repository_tests {
    use super::super::token_repo::{PostgresRefreshTokenRepository, TokenRepository};
    use crate::model::auth::RefreshToken;
    use chrono::{DateTime, Utc};
    use serial_test::serial;
    use sqlx::{PgPool, postgres::PgPoolOptions};
    use std::sync::Arc;
    use uuid::Uuid;

    async fn create_test_user(pool: &Arc<PgPool>, user_id: Option<Uuid>) -> Uuid {
        let id = user_id.unwrap_or_else(Uuid::new_v4);

        sqlx::query(
            r#"
            INSERT INTO users (id, name, email, password, role, created_at, updated_at, last_login)
            VALUES ($1, $2, $3, $4, $5::user_role, $6, $7, $8)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(id)
        .bind(format!("Test User {}", id))
        .bind(format!("test{}@example.com", id))
        .bind("password123")
        .bind("Attendee")
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(Option::<DateTime<Utc>>::None)
        .execute(pool.as_ref())
        .await
        .expect("Failed to create test user");

        id
    }

    async fn setup_test_db() -> Arc<PgPool> {
        dotenv::dotenv().ok();

        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            eprintln!("TEST_DATABASE_URL environment variable not set.");
            "postgresql://postgres:postgres@localhost:5432/eventsphere".to_string()
        });

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .unwrap_or_else(|e| {
                panic!("Failed to connect to test database: {}\nMake sure PostgreSQL is running and the credentials in TEST_DATABASE_URL are correct.", e);
            });
        sqlx::query("SET session_replication_role = 'replica';")
            .execute(&pool)
            .await
            .expect("Failed to disable foreign key constraints");

        sqlx::query("DELETE FROM refresh_tokens")
            .execute(&pool)
            .await
            .expect("Failed to clean up test refresh tokens");

        sqlx::query("DELETE FROM users")
            .execute(&pool)
            .await
            .expect("Failed to clean up test users");

        Arc::new(pool)
    }

    async fn cleanup_test_db(pool: &PgPool) {
        sqlx::query("DELETE FROM refresh_tokens")
            .execute(pool)
            .await
            .expect("Failed to clean up test refresh tokens");

        sqlx::query("DELETE FROM users")
            .execute(pool)
            .await
            .expect("Failed to clean up test users");

        sqlx::query("SET session_replication_role = 'origin';")
            .execute(pool)
            .await
            .expect("Failed to restore foreign key constraints");
    }

    #[tokio::test]
    #[serial]
    async fn test_create_token() {
        let pool = setup_test_db().await;
        let repo = PostgresRefreshTokenRepository::new(pool.clone());

        let user_id = create_test_user(&pool, None).await;

        let token = RefreshToken::new(user_id, "test-token".to_string(), 7);

        let result = repo.create(&token).await;
        assert!(result.is_ok(), "Failed to create token: {:?}", result.err());

        let found = repo
            .find_by_token(&token.token)
            .await
            .expect("Query failed");
        assert!(found.is_some(), "Token should be found after creation");
        let found_token = found.unwrap();
        assert_eq!(
            found_token.token, token.token,
            "Retrieved token should match"
        );
        assert_eq!(found_token.user_id, token.user_id, "User ID should match");

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_find_by_token() {
        let pool = setup_test_db().await;
        let repo = PostgresRefreshTokenRepository::new(pool.clone());

        let user_id = create_test_user(&pool, None).await;

        let token = RefreshToken::new(user_id, "find-token".to_string(), 7);

        repo.create(&token).await.expect("Failed to insert token");

        let result = repo.find_by_token("find-token").await;
        assert!(result.is_ok(), "Find by token query failed");
        let found = result.unwrap();
        assert!(found.is_some(), "Token should be found");
        assert_eq!(found.unwrap().token, "find-token");

        let result = repo.find_by_token("non-existent").await;
        assert!(result.is_ok(), "Find by token query failed");
        assert!(
            result.unwrap().is_none(),
            "Non-existent token should not be found"
        );

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_find_by_user_id() {
        let pool = setup_test_db().await;
        let repo = PostgresRefreshTokenRepository::new(pool.clone());

        let user_id = create_test_user(&pool, None).await;

        let token1 = RefreshToken::new(user_id, "token1".to_string(), 7);

        let token2 = RefreshToken::new(user_id, "token2".to_string(), 7);

        repo.create(&token1).await.expect("Failed to insert token1");
        repo.create(&token2).await.expect("Failed to insert token2");

        let result = repo.find_by_user_id(user_id).await;
        assert!(result.is_ok(), "Find by user ID query failed");

        let result_tokens = result.unwrap();
        assert_eq!(result_tokens.len(), 2, "Should find 2 tokens for user");
        assert_eq!(result_tokens[0].user_id, user_id, "User ID should match");
        assert_eq!(result_tokens[1].user_id, user_id, "User ID should match");

        let other_user_id = create_test_user(&pool, None).await;
        let result = repo.find_by_user_id(other_user_id).await;
        assert!(result.is_ok(), "Find by user ID query failed");
        let result_tokens = result.unwrap();
        assert!(result_tokens.is_empty(), "User should have 0 tokens");
    }

    #[tokio::test]
    #[serial]
    async fn test_revoke() {
        let pool = setup_test_db().await;
        let repo = PostgresRefreshTokenRepository::new(pool.clone());

        let user_id = create_test_user(&pool, None).await;

        let token = RefreshToken::new(user_id, "revoke-token".to_string(), 7);

        repo.create(&token).await.expect("Failed to insert token");

        let result = repo.revoke(token.id).await;
        assert!(result.is_ok(), "Revoke query failed");

        let found = repo
            .find_by_token(&token.token)
            .await
            .expect("Query failed")
            .unwrap();
        assert!(found.is_revoked, "Token should be revoked");

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_revoke_all_for_user() {
        let pool = setup_test_db().await;
        let repo = PostgresRefreshTokenRepository::new(pool.clone());

        let user_id = create_test_user(&pool, None).await;

        let token1 = RefreshToken::new(user_id, "user-token1".to_string(), 7);

        let token2 = RefreshToken::new(user_id, "user-token2".to_string(), 7);

        let other_user_id = create_test_user(&pool, None).await;
        let other_token = RefreshToken::new(other_user_id, "other-user-token".to_string(), 7);

        repo.create(&token1).await.expect("Failed to insert token1");
        repo.create(&token2).await.expect("Failed to insert token2");
        repo.create(&other_token)
            .await
            .expect("Failed to insert other token");

        let result = repo.revoke_all_for_user(user_id).await;
        assert!(result.is_ok(), "Revoke all for user query failed");

        let user_tokens = repo.find_by_user_id(user_id).await.expect("Query failed");
        for token in user_tokens {
            assert!(token.is_revoked, "User token should be revoked");
        }

        let other_user_tokens = repo
            .find_by_user_id(other_user_id)
            .await
            .expect("Query failed");
        assert_eq!(
            other_user_tokens.len(),
            1,
            "Should find 1 token for other user"
        );
        assert!(
            !other_user_tokens[0].is_revoked,
            "Other user token should not be revoked"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_token_validity() {
        let pool = setup_test_db().await;
        let repo = PostgresRefreshTokenRepository::new(pool.clone());

        let user_id = create_test_user(&pool, None).await;
        let mut token = RefreshToken::new(user_id, "expired-token".to_string(), 7);
        token.expires_at = Utc::now() - chrono::Duration::days(1);

        repo.create(&token)
            .await
            .expect("Failed to insert expired token");

        let found = repo
            .find_by_token(&token.token)
            .await
            .expect("Query failed")
            .unwrap();
        assert!(!found.is_valid(), "Expired token should not be valid");

        let mut revoked_token = RefreshToken::new(user_id, "revoked-token".to_string(), 7);
        revoked_token.is_revoked = true;

        repo.create(&revoked_token)
            .await
            .expect("Failed to insert revoked token");

        let found = repo
            .find_by_token(&revoked_token.token)
            .await
            .expect("Query failed")
            .unwrap();
        assert!(!found.is_valid(), "Revoked token should not be valid");

        let valid_token = RefreshToken::new(user_id, "valid-token".to_string(), 7);

        repo.create(&valid_token)
            .await
            .expect("Failed to insert valid token");

        let found = repo
            .find_by_token(&valid_token.token)
            .await
            .expect("Query failed")
            .unwrap();
        assert!(found.is_valid(), "Valid token should be valid");

        cleanup_test_db(&pool).await;
    }
}
