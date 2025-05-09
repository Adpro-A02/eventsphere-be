use std::sync::Arc;
use uuid::Uuid;

use crate::model::review::{Review, ReviewStatus};
use crate::repository::review::review_repository::ReviewRepository;
use crate::service::review::NotificationService;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository error: {0}")]
    RepositoryError(String),
    
    #[error("Review not found: {0}")]
    NotFound(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

pub struct ReviewService<R: ReviewRepository> {
    repository: Arc<R>,
    notification_service: Arc<NotificationService>,
}

impl<R: ReviewRepository> ReviewService<R> {
    pub fn new(repository: Arc<R>, notification_service: Arc<NotificationService>) -> Self {
        ReviewService { repository, notification_service }
    }

    // Create a review
    pub fn create_review(&self, event_id: Uuid, user_id: Uuid, rating: i32, comment: String) -> Result<Review, ServiceError> {
        if rating < 1 || rating > 5 {
            return Err(ServiceError::InvalidInput("Rating must be between 1 and 5".to_string()));
        }

        if comment.trim().is_empty() {
            return Err(ServiceError::InvalidInput("Comment cannot be empty".to_string()));
        }

        let review = Review::new(event_id, user_id, rating, comment);
        
        self.repository.add(review.clone())
            .map_err(|e| ServiceError::RepositoryError(e))?;

        self.notification_service.notify_created(&review)
            .map_err(|e| ServiceError::InternalError(e))?;

        Ok(review)
    }

    // Get a specific review
    pub fn get_review(&self, review_id: Uuid) -> Result<Review, ServiceError> {
        let review = self.repository.get_by_id(review_id)
            .map_err(|e| ServiceError::RepositoryError(e))?
            .ok_or_else(|| ServiceError::NotFound(format!("Review with ID {} not found", review_id)))?;
        
        Ok(review)
    }

    // Update an existing review
    pub fn update_review(&self, review_id: Uuid, rating: i32, comment: String) -> Result<Review, ServiceError> {
        let mut review = self.repository.get_by_id(review_id)
            .map_err(|e| ServiceError::RepositoryError(e))?
            .ok_or_else(|| ServiceError::NotFound(format!("Review with ID {} not found", review_id)))?;
        
        review.update(rating, comment);
        
        self.repository.update_review(review_id, review.clone())
            .map_err(|e| ServiceError::RepositoryError(e))?;
        
        self.notification_service.notify_updated(&review)
            .map_err(|e| ServiceError::InternalError(e))?;

        Ok(review)
    }

    // Delete a review
    pub fn delete_review(&self, review_id: Uuid) -> Result<(), ServiceError> {
        let review = self.repository.get_by_id(review_id)
            .map_err(|e| ServiceError::RepositoryError(e))?
            .ok_or_else(|| ServiceError::NotFound(format!("Review with ID {} not found", review_id)))?;

        self.repository.delete(review_id)
            .map_err(|e| ServiceError::RepositoryError(e))?;
        
        self.notification_service.notify_deleted(&review)
            .map_err(|e| ServiceError::InternalError(e))?;

        Ok(())
    }

    // Approve a review
    pub fn approve_review(&self, review_id: Uuid) -> Result<Review, ServiceError> {
        let mut review = self.repository.get_by_id(review_id)
            .map_err(|e| ServiceError::RepositoryError(e))?
            .ok_or_else(|| ServiceError::NotFound(format!("Review with ID {} not found", review_id)))?;
        
        review.change_status(ReviewStatus::Approved);
        
        self.repository.update_review(review_id, review.clone())
            .map_err(|e| ServiceError::RepositoryError(e))?;
        
        self.notification_service.notify_approved(&review)
            .map_err(|e| ServiceError::InternalError(e))?;

        Ok(review)
    }

    // Reject a review
    pub fn reject_review(&self, review_id: Uuid) -> Result<Review, ServiceError> {
        let mut review = self.repository.get_by_id(review_id)
            .map_err(|e| ServiceError::RepositoryError(e))?
            .ok_or_else(|| ServiceError::NotFound(format!("Review with ID {} not found", review_id)))?;
        
        review.change_status(ReviewStatus::Rejected);
        
        self.repository.update_review(review_id, review.clone())
            .map_err(|e| ServiceError::RepositoryError(e))?;
        
        self.notification_service.notify_rejected(&review)
            .map_err(|e| ServiceError::InternalError(e))?;

        Ok(review)
    }

    // List all reviews for an event
    pub fn list_reviews_by_event(&self, event_id: Uuid) -> Result<Vec<Review>, ServiceError> {
        let reviews = self.repository.get_by_event_id(event_id)
            .map_err(|e| ServiceError::RepositoryError(e))?;
        
        Ok(reviews)
    }
}
