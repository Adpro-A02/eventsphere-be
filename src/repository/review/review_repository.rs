use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::model::review::{Review, ReviewStatus};

pub trait ReviewRepository: Send + Sync + 'static {
    fn add(&self, review: Review) -> Result<Review, String>;
    fn delete(&self, review_id: Uuid) -> Result<(), String>;
    fn update_review(&self, review_id: Uuid, updated_review: Review) -> Result<Review, String>;
    fn list_reviews(&self) -> Result<Vec<Review>, String>;
    fn get_by_id(&self, review_id: Uuid) -> Result<Option<Review>, String>;
    fn get_by_event_id(&self, event_id: Uuid) -> Result<Vec<Review>, String>;
}

// In-memory implementation of ReviewRepository
pub struct InMemoryReviewRepository {
    reviews: Mutex<HashMap<Uuid, Review>>,
}

impl InMemoryReviewRepository {
    pub fn new() -> Self {
        InMemoryReviewRepository {
            reviews: Mutex::new(HashMap::new()),
        }
    }
}

impl ReviewRepository for InMemoryReviewRepository {
    fn add(&self, review: Review) -> Result<Review, String> {
        let mut reviews = self.reviews.lock().map_err(|e| e.to_string())?;
        let review_clone = review.clone();
        reviews.insert(review.review_id, review);
        Ok(review_clone)
    }

    fn delete(&self, review_id: Uuid) -> Result<(), String> {
        let mut reviews = self.reviews.lock().map_err(|e| e.to_string())?;
        
        if reviews.remove(&review_id).is_none() {
            return Err(format!("Review with ID {} not found", review_id));
        }
        
        Ok(())
    }

    fn update_review(&self, review_id: Uuid, updated_review: Review) -> Result<Review, String> {
        let mut reviews = self.reviews.lock().map_err(|e| e.to_string())?;
        
        if !reviews.contains_key(&review_id) {
            return Err(format!("Review with ID {} not found", review_id));
        }
        
        let review_clone = updated_review.clone();
        reviews.insert(review_id, updated_review);
        Ok(review_clone)
    }

    fn list_reviews(&self) -> Result<Vec<Review>, String> {
        let reviews = self.reviews.lock().map_err(|e| e.to_string())?;
        Ok(reviews.values().cloned().collect())
    }

    fn get_by_id(&self, review_id: Uuid) -> Result<Option<Review>, String> {
        let reviews = self.reviews.lock().map_err(|e| e.to_string())?;
        Ok(reviews.get(&review_id).cloned())
    }

    fn get_by_event_id(&self, event_id: Uuid) -> Result<Vec<Review>, String> {
        let reviews = self.reviews.lock().map_err(|e| e.to_string())?;
        let filtered_reviews: Vec<Review> = reviews.values()
            .filter(|&review| review.event_id == event_id)
            .cloned()
            .collect();
        Ok(filtered_reviews)
    }

    
}
