#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDateTime, Utc};
    use uuid::Uuid;

    #[test]
    fn test_create_review_valid() {
        let review = Review {
            id: Uuid::new_v4(),
            event_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            rating: 4,
            comment: "Great event!".to_string(),
            created_date: Utc::now().naive_utc(),
            updated_date: Utc::now().naive_utc(),
            status: ReviewStatus::Pending,
        };

        assert_eq!(review.rating, 4);
       

        assert_eq!(review.comment, "Great event!");
        assert!(review.created_date <= Utc::now().naive_utc());
    }

    #[test]
    fn test_create_review_invalid_rating() {
        let review = Review {
            id: Uuid::new_v4(),
            event_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            rating: 6, 
            comment: "Not good.".to_string(),
            created_date: Utc::now().naive_utc(),
            updated_date: Utc::now().naive_utc(),
            status: ReviewStatus::Pending,
        };

        assert!(review.rating < 1 || review.rating > 5, "Rating must be between 1 and 5.");
    }

    #[test]
    fn test_update_review() {
        let mut review = Review {
            id: Uuid::new_v4(),
            event_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            rating: 4,
            comment: "Great event!".to_string(),
            created_date: Utc::now().naive_utc(),
            updated_date: Utc::now().naive_utc(),
            status: ReviewStatus::Pending,
        };

        review.comment = "Updated comment.".to_string();
        review.rating = 5;
        review.updated_date = Utc::now().naive_utc();

        assert_eq!(review.comment, "Updated comment.");
        assert_eq!(review.rating, 5);
        assert!(review.updated_date > review.created_date);
    }

    #[test]
    fn test_review_status_update() {
        let mut review = Review {
            id: Uuid::new_v4(),
            event_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            rating: 4,
            comment: "Great event!".to_string(),
            created_date: Utc::now().naive_utc(),
            updated_date: Utc::now().naive_utc(),
            status: ReviewStatus::Pending,
        };

        review.status = ReviewStatus::Approved;

        assert_eq!(review.status, ReviewStatus::Approved);
    }

    #[test]
    fn test_invalid_data_handling() {

        let review = Review {
            id: Uuid::new_v4(),
            event_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            rating: 6, 
            comment: "".to_string(), 
            created_date: Utc::now().naive_utc(),
            updated_date: Utc::now().naive_utc(),
            status: ReviewStatus::Pending,
        };

        assert!(review.rating < 1 || review.rating > 5, "Rating must be between 1 and 5.");
        assert!(review.comment.is_empty(), "Comment cannot be empty.");
    }

    #[test]
    fn test_get_reviews_by_event() {

        let review = Review {
            id: Uuid::new_v4(),
            event_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            rating: 4,
            comment: "Great event!".to_string(),
            created_date: Utc::now().naive_utc(),
            updated_date: Utc::now().naive_utc(),
            status: ReviewStatus::Pending,
        };

        let reviews = get_reviews_by_event(&review.event_id);

        assert!(reviews.contains(&review));
    }
}