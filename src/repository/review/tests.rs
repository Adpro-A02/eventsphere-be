#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;

    #[test]
    fn test_create_review() {
        let mut repo = ReviewRepository::new();
        
        let review = Review::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            5,
            "Amazing event!".to_string(),
        );

        let saved_review = repo.save(review.clone());
        
        assert_eq!(saved_review.rating, 5);
        assert_eq!(saved_review.comment, "Amazing event!");
    }

    #[test]
    fn test_find_review_by_id() {
        let mut repo = ReviewRepository::new();
        
        let review = Review::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            4,
            "Good event!".to_string(),
        );
        
        repo.save(review.clone());

        let found_review = repo.find_by_id(&review.id);
        
        assert!(found_review.is_some());
        assert_eq!(found_review.unwrap().comment, "Good event!");
    }

    #[test]
    fn test_find_all_reviews() {
        let mut repo = ReviewRepository::new();
        
        let review1 = Review::new(Uuid::new_v4(), Uuid::new_v4(), 5, "Great event!".to_string());
        let review2 = Review::new(Uuid::new_v4(), Uuid::new_v4(), 3, "Okay event.".to_string());

        repo.save(review1.clone());
        repo.save(review2.clone());

        let all_reviews = repo.find_all();

        assert_eq!(all_reviews.len(), 2);
    }

    #[test]
    fn test_delete_review() {
        let mut repo = ReviewRepository::new();
        
        let review = Review::new(Uuid::new_v4(), Uuid::new_v4(), 2, "Bad event.".to_string());

        repo.save(review.clone());
        
        let review_id = review.id.clone();
        repo.delete(&review_id);

        let found_review = repo.find_by_id(&review_id);

        assert!(found_review.is_none());
    }

    #[test]
    fn test_find_all_active_reviews() {
        let mut repo = ReviewRepository::new();
        
        let review1 = Review::new(Uuid::new_v4(), Uuid::new_v4(), 5, "Great event!".to_string());
        let review2 = Review::new(Uuid::new_v4(), Uuid::new_v4(), 3, "Not bad.".to_string());
        let mut review3 = Review::new(Uuid::new_v4(), Uuid::new_v4(), 4, "Okay event.".to_string());
        review3.status = ReviewStatus::Approved;

        repo.save(review1);
        repo.save(review2);
        repo.save(review3);

        let active_reviews = repo.find_all_active_reviews();

        assert_eq!(active_reviews.len(), 1);
        assert_eq!(active_reviews[0].rating, 4);
    }

    #[test]
    fn test_average_rating_for_event() {
        let mut repo = ReviewRepository::new();
        
        let event_id = Uuid::new_v4();
        
        let review1 = Review::new(event_id, Uuid::new_v4(), 4, "Great event!".to_string());
        let review2 = Review::new(event_id, Uuid::new_v4(), 3, "Good event.".to_string());
        let review3 = Review::new(event_id, Uuid::new_v4(), 5, "Excellent event!".to_string());

        repo.save(review1);
        repo.save(review2);
        repo.save(review3);

        let average_rating = repo.average_rating_for_event(&event_id);

        assert_eq!(average_rating, 4.0);
    }
}