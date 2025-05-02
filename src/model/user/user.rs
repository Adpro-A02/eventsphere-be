use chrono::{DateTime, Utc};
use uuid::Uuid;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Admin,
    Organizer,
    Attendee,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

impl User {
    pub fn new(name: String, email: String, password: String, role: UserRole) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            email,
            password,
            role,
            created_at: now,
            updated_at: now,
            last_login: None,
        }
    }

    pub fn update_last_login(&mut self) {
        self.last_login = Some(Utc::now());
    }

    pub fn update_password(&mut self, new_password: String) {
        self.password = new_password;
        self.updated_at = Utc::now();
    }

    pub fn update_role(&mut self, new_role: UserRole) {
        self.role = new_role;
        self.updated_at = Utc::now();
    }

    pub fn update_profile(&mut self, name: Option<String>, email: Option<String>) {
        if let Some(new_name) = name {
            self.name = new_name;
        }
        if let Some(new_email) = email {
            self.email = new_email;
        }
        self.updated_at = Utc::now();
    }

    pub fn get_user_info(&self) -> &Self {
        self
    }
}
