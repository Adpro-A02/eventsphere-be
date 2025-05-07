#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use crate::models::review::{Review, ReviewStatus};
    use crate::models::review_repository::ReviewRepository;
    use crate::models::notification_service::NotificationService;
    use crate::models::review_service::ReviewService;
    use uuid::Uuid;

    #[actix_rt::test]
    async fn test_create_review() {
        let mut repo = ReviewRepository::new();
        let notification_service = NotificationService::new();
        let service = ReviewService::new(&mut repo, &notification_service);

        let review = Review {
            id: Uuid::new_v4(),
            event_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            rating: 5,
            comment: "Great event!".to_string(),
            created_date: chrono::Utc::now().naive_utc(),
            updated_date: chrono::Utc::now().naive_utc(),
            status: ReviewStatus::Approved,
        };

        let app = test::init_service(App::new().app_data(web::Data::new(service)).configure(config)).await;
        let req = test::TestRequest::post()
            .uri("/reviews")
            .set_json(&review)
            .to_request();

        let response = test::call_service(&app, req).await;

        assert_eq!(response.status(), 201);
    }

    #[actix_rt::test]
    async fn test_get_reviews_by_event_id() {
        let mut repo = ReviewRepository::new();
        let notification_service = NotificationService::new();
        let service = ReviewService::new(&mut repo, &notification_service);

        let event_id = Uuid::new_v4();
        let review = Review {
            id: Uuid::new_v4(),
            event_id: event_id.clone(),
            user_id: Uuid::new_v4(),
            rating: 4,
            comment: "Good event.".to_string(),
            created_date: chrono::Utc::now().naive_utc(),
            updated_date: chrono::Utc::now().naive_utc(),
            status: ReviewStatus::Approved,
        };
        
        repo.save(review.clone());
        
        let app = test::init_service(App::new().app_data(web::Data::new(service)).configure(config)).await;
        let req = test::TestRequest::get()
            .uri(&format!("/reviews/event/{}", event_id))
            .to_request();

        let response = test::call_service(&app, req).await;

        assert_eq!(response.status(), 200);
    }

}
