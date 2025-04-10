use std::collections::HashMap;
use uuid::Uuid;
use chrono::{NaiveDateTime, Utc};

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
}

pub struct ReviewRepository {
    reviews: HashMap<Uuid, Review>,
}

impl ReviewRepository {
    pub fn new() -> Self {
        ReviewRepository {
            reviews: HashMap::new(),
        }
    }

    pub fn find_by_id(&self, id: &Uuid) -> Option<&Review> {
        self.reviews.get(id)
    }

    pub fn find_all(&self) -> Vec<&Review> {
        self.reviews.values().collect()
    }

    pub fn save(&mut self, review: Review) -> &Review {
        self.reviews.insert(review.id, review);
        self.reviews.get(&review.id).unwrap()
    }

    pub fn delete(&mut self, id: &Uuid) {
        self.reviews.remove(id);
    }

    pub fn find_all_active_reviews(&self) -> Vec<&Review> {
        self.reviews
            .values()
            .filter(|review| review.status == ReviewStatus::Approved)
            .collect()
    }

    pub fn average_rating_for_event(&self, event_id: &Uuid) -> f64 {
        let event_reviews: Vec<&Review> = self.reviews.values().filter(|r| &r.event_id == event_id).collect();
        
        if event_reviews.is_empty() {
            return 0.0; // Avoid division by zero
        }

        let total_rating: i32 = event_reviews.iter().map(|r| r.rating).sum();
        total_rating as f64 / event_reviews.len() as f64
    }
}