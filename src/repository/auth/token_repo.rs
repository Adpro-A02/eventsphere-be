use crate::model::auth::RefreshToken;
use async_trait::async_trait;
use sqlx::PgPool;
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;

#[async_trait]
pub trait TokenRepository: Send + Sync {
    async fn create(&self, token: &RefreshToken) -> Result<(), Box<dyn Error>>;
    async fn find_by_token(&self, token: &str) -> Result<Option<RefreshToken>, Box<dyn Error>>;
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<RefreshToken>, Box<dyn Error>>;
    async fn revoke(&self, token_id: Uuid) -> Result<(), Box<dyn Error>>;
    async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<(), Box<dyn Error>>;
}

pub struct PostgresRefreshTokenRepository {
    pool: Arc<PgPool>,
}

impl PostgresRefreshTokenRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TokenRepository for PostgresRefreshTokenRepository {
    async fn create(&self, token: &RefreshToken) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (id, user_id, token, expires_at, is_revoked, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(token.id)
        .bind(token.user_id)
        .bind(&token.token)
        .bind(token.expires_at)
        .bind(token.is_revoked)
        .bind(token.created_at)
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_token(&self, token: &str) -> Result<Option<RefreshToken>, Box<dyn Error>> {
        let result = sqlx::query_as!(
            RefreshToken,
            "SELECT id, user_id, token, expires_at, is_revoked, created_at FROM refresh_tokens WHERE token = $1",
            token
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<RefreshToken>, Box<dyn Error>> {
        let result = sqlx::query_as!(
            RefreshToken,
            "SELECT id, user_id, token, expires_at, is_revoked, created_at FROM refresh_tokens WHERE user_id = $1",
            user_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(result)
    }

    async fn revoke(&self, token_id: Uuid) -> Result<(), Box<dyn Error>> {
        sqlx::query("UPDATE refresh_tokens SET is_revoked = TRUE WHERE id = $1")
            .bind(token_id)
            .execute(&*self.pool)
            .await?;

        Ok(())
    }

    async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<(), Box<dyn Error>> {
        sqlx::query("UPDATE refresh_tokens SET is_revoked = TRUE WHERE user_id = $1")
            .bind(user_id)
            .execute(&*self.pool)
            .await?;

        Ok(())
    }
}