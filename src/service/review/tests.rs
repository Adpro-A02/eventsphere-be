#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::{NaiveDateTime, Utc};
    use std::collections::HashMap;

    #[derive(Clone)]
    pub struct MockReviewRepository {
        pub reviews: HashMap<Uuid, Review>,
    }

    impl MockReviewRepository {
        pub fn new() -> Self {
            MockReviewRepository {
                reviews: HashMap::new(),
            }
        }

        pub fn save(&mut self, review: Review) -> &Review {
            self.reviews.insert(review.id, review);
            self.reviews.get(&review.id).unwrap()
        }

        pub fn find_by_id(&self, id: &Uuid) -> Option<&Review> {
            self.reviews.get(id)
        }

        pub fn delete(&mut self, id: &Uuid) {
            self.reviews.remove(id);
        }

        pub fn average_rating_for_event(&self, event_id: &Uuid) -> f64 {
            let event_reviews: Vec<&Review> = self.reviews.values().filter(|r| &r.event_id == event_id).collect();
            if event_reviews.is_empty() {
                return 0.0;
            }
            let total_rating: i32 = event_reviews.iter().map(|r| r.rating).sum();
            total_rating as f64 / event_reviews.len() as f64
        }
    }

    pub struct MockNotificationService;

    impl MockNotificationService {
        pub fn new() -> Self {
            MockNotificationService
        }

        pub fn send_review_created_notification(&self, review: &Review) {
            println!("Mock Notification: Review created with ID {}", review.id);
        }

        pub fn send_review_approved_notification(&self, review: &Review) {
            println!("Mock Notification: Review with ID {} has been approved.", review.id);
        }

        pub fn send_review_rejected_notification(&self, review: &Review) {
            println!("Mock Notification: Review with ID {} has been rejected.", review.id);
        }
    }

    #[test]
    fn test_create_review() {
        let mut mock_repo = MockReviewRepository::new();
        let mock_notification_service = MockNotificationService::new();
        let mut service = ReviewService::new(&mut mock_repo, &mock_notification_service);

        let review = Review {
            id: Uuid::new_v4(),
            event_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            rating: 5,
            comment: "Excellent event!".to_string(),
            created_date: Utc::now().naive_utc(),
            updated_date: Utc::now().naive_utc(),
            status: ReviewStatus::Approved,
        };

        let saved_review = service.create_review(review.clone());

        assert_eq!(saved_review.rating, 5);
        assert_eq!(saved_review.comment, "Excellent event!");
    }

    #[test]
    fn test_update_review_approved() {
        let mut mock_repo = MockReviewRepository::new();
        let mock_notification_service = MockNotificationService::new();
        let mut service = ReviewService::new(&mut mock_repo, &mock_notification_service);

        let review = Review {
            id: Uuid::new_v4(),
            event_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            rating: 3,
            comment: "Good event!".to_string(),
            created_date: Utc::now().naive_utc(),
            updated_date: Utc::now().naive_utc(),
            status: ReviewStatus::Pending,
        };

        let saved_review = service.create_review(review.clone());

        let updated_review = Review {
            rating: 4,
            comment: "Better event!".to_string(),
            status: ReviewStatus::Approved, 
            ..saved_review.clone()
        };

        let updated_review_result = service.update_review(updated_review);

        assert_eq!(updated_review_result.rating, 4);
        assert_eq!(updated_review_result.comment, "Better event!");
    }

    #[test]
    fn test_update_review_rejected() {
        let mut mock_repo = MockReviewRepository::new();
        let mock_notification_service = MockNotificationService::new();
        let mut service = ReviewService::new(&mut mock_repo, &mock_notification_service);

        let review = Review {
            id: Uuid::new_v4(),
            event_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            rating: 2,
            comment: "Bad event.".to_string(),
            created_date: Utc::now().naive_utc(),
            updated_date: Utc::now().naive_utc(),
            status: ReviewStatus::Pending,
        };

        let saved_review = service.create_review(review.clone());

        let updated_review = Review {
            rating: 1,
            comment: "Poor event.".to_string(),
            status: ReviewStatus::Rejected, 
            ..saved_review.clone()
        };

        let updated_review_result = service.update_review(updated_review);

        assert_eq!(updated_review_result.rating, 1);
        assert_eq!(updated_review_result.comment, "Poor event!");
    }
}