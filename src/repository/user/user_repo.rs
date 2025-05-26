use crate::model::user::User;
use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use sqlx::{PgPool, Row};
use crate::model::user::UserRole;
use std::str::FromStr;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, Box<dyn Error>>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Box<dyn Error>>;
    async fn create(&self, user: &User) -> Result<(), Box<dyn Error>>;
    async fn update(&self, user: &User) -> Result<(), Box<dyn Error>>;
    async fn delete(&self, id: Uuid) -> Result<(), Box<dyn Error>>;
    async fn find_all(&self) -> Result<Vec<User>, Box<dyn Error>>;
}

#[async_trait]
pub trait UserPersistenceStrategy: Send + Sync {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, Box<dyn Error>>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Box<dyn Error>>;
    async fn create(&self, user: &User) -> Result<(), Box<dyn Error>>;
    async fn update(&self, user: &User) -> Result<(), Box<dyn Error>>;
    async fn delete(&self, id: Uuid) -> Result<(), Box<dyn Error>>;
    async fn find_all(&self) -> Result<Vec<User>, Box<dyn Error>>;
}

pub struct InMemoryUserPersistence {
    users: RwLock<HashMap<Uuid, User>>,
}

impl InMemoryUserPersistence {
    pub fn new() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl UserPersistenceStrategy for InMemoryUserPersistence {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, Box<dyn Error>> {
        let users = self.users.read().unwrap();
        let user = users.values().find(|u| u.email == email).cloned();
        Ok(user)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Box<dyn Error>> {
        let users = self.users.read().unwrap();
        Ok(users.get(&id).cloned())
    }

    async fn create(&self, user: &User) -> Result<(), Box<dyn Error>> {
        let mut users = self.users.write().unwrap();
        users.insert(user.id, user.clone());
        Ok(())
    }

    async fn update(&self, user: &User) -> Result<(), Box<dyn Error>> {
        let mut users = self.users.write().unwrap();
        
        if users.contains_key(&user.id) {
            users.insert(user.id, user.clone());
            Ok(())
        } else {
            Err("User not found".into())
        }
    }

    async fn delete(&self, id: Uuid) -> Result<(), Box<dyn Error>> {
        let mut users = self.users.write().unwrap();
        
        if users.remove(&id).is_some() {
            Ok(())
        } else {
            Err("User not found".into())
        }
    }

    async fn find_all(&self) -> Result<Vec<User>, Box<dyn Error>> {
        let users = self.users.read().unwrap();
        let all_users = users.values().cloned().collect();
        Ok(all_users)
    }
}

pub struct DbUserRepository<S: UserPersistenceStrategy> {
    strategy: S,
}

impl<S: UserPersistenceStrategy> DbUserRepository<S> {
    pub fn new(strategy: S) -> Self {
        Self { strategy }
    }
}

#[async_trait]
impl<S: UserPersistenceStrategy + Send + Sync> UserRepository for DbUserRepository<S> {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, Box<dyn Error>> {
        self.strategy.find_by_email(email).await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Box<dyn Error>> {
        self.strategy.find_by_id(id).await
    }

    async fn create(&self, user: &User) -> Result<(), Box<dyn Error>> {
        self.strategy.create(user).await
    }

    async fn update(&self, user: &User) -> Result<(), Box<dyn Error>> {
        self.strategy.update(user).await
    }

    async fn delete(&self, id: Uuid) -> Result<(), Box<dyn Error>> {
        self.strategy.delete(id).await
    }

    async fn find_all(&self) -> Result<Vec<User>, Box<dyn Error>> {
        self.strategy.find_all().await
    }
}

pub struct PostgresUserRepository {
    pool: Arc<PgPool>,
}

impl PostgresUserRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserPersistenceStrategy for PostgresUserRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, Box<dyn Error>> {
        // Modified query to cast role to text
        let query = "SELECT id, name, email, password, role::text as role, created_at, updated_at, last_login FROM users WHERE email = $1";
        
        let row = sqlx::query(query)
            .bind(email)
            .fetch_optional(&*self.pool)
            .await?;
        
        let user = row.map(|row| User {
            id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
            password: row.get("password"),
            role: UserRole::from_str(row.get::<&str, _>("role")).unwrap_or(UserRole::Attendee),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            last_login: row.get("last_login"),
        });
        
        Ok(user)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Box<dyn Error>> {
        let query = "SELECT id, name, email, password, role::text as role, created_at, updated_at, last_login FROM users WHERE id = $1";
        
        let row = sqlx::query(query)
            .bind(id)
            .fetch_optional(&*self.pool)
            .await?;
        
        let user = row.map(|row| User {
            id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
            password: row.get("password"),
            role: UserRole::from_str(row.get::<&str, _>("role")).unwrap_or(UserRole::Attendee),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            last_login: row.get("last_login"),
        });
        
        Ok(user)
    }
    
    async fn create(&self, user: &User) -> Result<(), Box<dyn Error>> {
        let query = "INSERT INTO users (id, name, email, password, role, created_at, updated_at, last_login) VALUES ($1, $2, $3, $4, $5::user_role, $6, $7, $8)";
        
        sqlx::query(query)
            .bind(user.id)
            .bind(&user.name)
            .bind(&user.email)
            .bind(&user.password)
            .bind(user.role.to_string())
            .bind(user.created_at)
            .bind(user.updated_at)
            .bind(user.last_login)
            .execute(&*self.pool)
            .await?;
        
        Ok(())
    }

    async fn update(&self, user: &User) -> Result<(), Box<dyn Error>> {
        let query = "UPDATE users SET name = $1, email = $2, password = $3, role = $4::user_role, updated_at = $5, last_login = $6 WHERE id = $7";
        
        let result = sqlx::query(query)
            .bind(&user.name)
            .bind(&user.email)
            .bind(&user.password)
            .bind(user.role.to_string())
            .bind(user.updated_at)
            .bind(user.last_login)
            .bind(user.id)
            .execute(&*self.pool)
            .await?;
        
        if result.rows_affected() == 0 {
            return Err("User not found".into());
        }
        
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Box<dyn Error>> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&*self.pool)
            .await?;
            
        if result.rows_affected() == 0 {
            return Err("User not found".into());
        }
        
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<User>, Box<dyn Error>> {
        // Modified query to cast role to text
        let query = "SELECT id, name, email, password, role::text as role, created_at, updated_at, last_login FROM users";
        
        let rows = sqlx::query(query)
            .fetch_all(&*self.pool)
            .await?;
        
        let users = rows.iter()
            .map(|row| User {
                id: row.get("id"),
                name: row.get("name"),
                email: row.get("email"),
                password: row.get("password"),
                role: UserRole::from_str(row.get::<&str, _>("role")).unwrap_or(UserRole::Attendee),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                last_login: row.get("last_login"),
            })
            .collect();
        
        Ok(users)
    }
}