use crate::models::review::{Review, ReviewStatus};
use crate::models::review_repository::ReviewRepository;
use crate::models::notification_service::NotificationService;
use uuid::Uuid;

pub struct ReviewService<'a> {
    repository: &'a mut ReviewRepository,
    notification_service: &'a NotificationService,
}

impl<'a> ReviewService<'a> {
    pub fn new(repository: &'a mut ReviewRepository, notification_service: &'a NotificationService) -> Self {
        ReviewService {
            repository,
            notification_service,
        }
    }

    pub fn create_review(&mut self, review: Review) -> Review {
        if self.validate_review(&review) {
            let saved_review = self.repository.save(review.clone());
            self.notification_service.send_review_created_notification(&saved_review);
            saved_review
        } else {
            panic!("Invalid review data")
        }
    }

    pub fn update_review(&mut self, review: Review) -> Review {
        if self.validate_review(&review) {
            let updated_review = self.repository.save(review.clone());
            match updated_review.status {
                ReviewStatus::Approved => {
                    self.notification_service.send_review_approved_notification(&updated_review);
                }
                ReviewStatus::Rejected => {
                    self.notification_service.send_review_rejected_notification(&updated_review);
                }
                _ => {

                }
            }
            updated_review
        } else {
            panic!("Invalid review data")
        }
    }

    pub fn delete_review(&mut self, id: &Uuid) {
        self.repository.delete(id);

    }

    pub fn validate_review(&self, review: &Review) -> bool {
        review.rating >= 1 && review.rating <= 5 && !review.comment.is_empty()
    }

    pub fn calculate_event_average_rating(&self, event_id: &Uuid) -> f64 {
        self.repository.average_rating_for_event(event_id)
    }
}