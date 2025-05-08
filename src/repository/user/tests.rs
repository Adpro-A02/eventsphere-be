use crate::model::user::{User, UserRole};
use crate::repository::user::user_repo::{InMemoryUserPersistence, UserRepository, DbUserRepository};

#[tokio::test]
async fn test_create_user() {
    let repo = create_test_repo();
    let user = create_test_user("test@danilliman.com");
    
    let result = repo.create(&user).await;
    assert!(result.is_ok());
    
    let found = repo.find_by_id(user.id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().email, "test@danilliman.com");
}

#[tokio::test]
async fn test_find_by_email() {
    let repo = create_test_repo();
    let user = create_test_user("email_test@danilliman.com");
    
    repo.create(&user).await.unwrap();
    
    let found = repo.find_by_email("email_test@danilliman.com").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, user.id);
}

#[tokio::test]
async fn test_update_user() {
    let repo = create_test_repo();
    let mut user = create_test_user("update@danilliman.com");
    
    repo.create(&user).await.unwrap();
    
    user.name = "Updated Name".to_string();
    let result = repo.update(&user).await;
    assert!(result.is_ok());
    
    let found = repo.find_by_id(user.id).await.unwrap().unwrap();
    assert_eq!(found.name, "Updated Name");
}

#[tokio::test]
async fn test_delete_user() {
    let repo = create_test_repo();
    let user = create_test_user("delete@danilliman.com");
    let user_id = user.id;
    
    repo.create(&user).await.unwrap();
    let result = repo.delete(user_id).await;
    assert!(result.is_ok());
    
    let found = repo.find_by_id(user_id).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_find_all() {
    let repo = create_test_repo();
    
    for i in 0..3 {
        let user = create_test_user(&format!("user{}@danilliman.com", i));
        repo.create(&user).await.unwrap();
    }
    
    let all_users = repo.find_all().await.unwrap();
    assert_eq!(all_users.len(), 3);
}

fn create_test_repo() -> impl UserRepository {
    let persistence = InMemoryUserPersistence::new();
    DbUserRepository::new(persistence)
}

fn create_test_user(email: &str) -> User {
    User::new(
        "Test User".to_string(),
        email.to_string(),
        "password123".to_string(),
        UserRole::Attendee,
    )
}
