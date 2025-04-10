use chrono::{NaiveDateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Review {
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub comment: String,
    pub created_date: NaiveDateTime,
    pub updated_date: NaiveDateTime,
    pub status: ReviewStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReviewStatus {
    Pending,
    Approved,
    Rejected,
}

impl Review {
    pub fn new(event_id: Uuid, user_id: Uuid, rating: i32, comment: String) -> Self {
        let now = Utc::now().naive_utc();
        Review {
            id: Uuid::new_v4(),
            event_id,
            user_id,
            rating,
            comment,
            created_date: now,
            updated_date: now,
            status: ReviewStatus::Pending,
        }
    }

    pub fn update(&mut self, rating: i32, comment: String) {
        self.rating = rating;
        self.comment = comment;
        self.updated_date = Utc::now().naive_utc();
    }

    pub fn change_status(&mut self, status: ReviewStatus) {
        self.status = status;
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.rating < 1 || self.rating > 5 {
            return Err("Rating must be between 1 and 5.".to_string());
        }
        if self.comment.trim().is_empty() {
            return Err("Comment cannot be empty.".to_string());
        }
        Ok(())
    }
}

