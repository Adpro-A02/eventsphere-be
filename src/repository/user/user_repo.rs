use crate::model::user::User;
use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::RwLock;

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