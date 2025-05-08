use crate::model::user::{User, UserRole};

#[cfg(test)]
pub mod model_tests {
    use super::*;

    #[test]
    fn test_user_new() {
        let name = "John Doe".to_string();
        let email = "john.doe@gmai.com".to_string();
        let password = "password123".to_string();
        let role = UserRole::Attendee;

        let user = User::new(
            name.clone(),
            email.clone(),
            password.clone(),
            role.clone()
        );

        assert_eq!(user.name, name);
        assert_eq!(user.email, email);
        assert_eq!(user.password, password);
        assert_eq!(user.role, role);
        assert!(!user.id.to_string().is_empty());
        assert!(user.created_at <= chrono::Utc::now());
        assert!(user.updated_at <= chrono::Utc::now());
        assert!(user.last_login.is_none());
    }

    #[test]
    fn test_user_update_last_login() {
        let mut user = User::new(
            "Jane Doe".to_string(),
            "jane.doe@gmail.com".to_string(),
            "password456".to_string(),
            UserRole::Organizer
        );

        // Verify initially None
        assert!(user.last_login.is_none());
        
        user.update_last_login();
        
        // Verify now Some
        assert!(user.last_login.is_some());
    }

    #[test]
    fn test_user_update_password() {
        let mut user = User::new(
            "Alice Smith".to_string(),
            "alice.smith@gmail.com".to_string(),
            "old_password".to_string(),
            UserRole::Admin
        );

        let new_password = "new_password".to_string();
        user.update_password(new_password.clone());

        assert_eq!(user.password, new_password);
        assert!(user.updated_at > user.created_at);
    }

    #[test]
    fn test_user_update_role() {
        let mut user = User::new(
            "Bob Johnson".to_string(),
            "bob.johnson@gmail.com".to_string(),
            "password789".to_string(),
            UserRole::Attendee
        );

        let new_role = UserRole::Organizer;
        user.update_role(new_role.clone());

        assert_eq!(user.role, new_role);    
        assert!(user.updated_at > user.created_at);
    }

    #[test]
    fn test_user_update_profile() {
        let mut user = User::new(
            "Jim Bob".to_string(),
            "jim.bob@gmail.com".to_string(),
            "password123".to_string(),
            UserRole::Attendee
        );

        let new_name = Some("Jim Bob Jr.".to_string());
        let new_email = Some("jimbob.jr@gmail.com".to_string());
        user.update_profile(new_name.clone(), new_email.clone());

        assert_eq!(user.name, new_name.unwrap());
        assert_eq!(user.email, new_email.unwrap());
        assert!(user.updated_at > user.created_at);
    }

    #[test]
    fn test_user_get_user_info() {
        let user = User::new(
            "Charlie Brown".to_string(),
            "charlie@gmail.com".to_string(),
            "password123".to_string(),
            UserRole::Attendee
        );

        let user_info = user.get_user_info();
        assert_eq!(user_info.name, user.name);
        assert_eq!(user_info.email, user.email);
        assert_eq!(user_info.password, user.password);
        assert_eq!(user_info.role, user.role);
        assert_eq!(user_info.created_at, user.created_at);
        assert_eq!(user_info.updated_at, user.updated_at);
        assert_eq!(user_info.last_login, user.last_login);
    }
}