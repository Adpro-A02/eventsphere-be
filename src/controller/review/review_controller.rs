use actix_web::{web, HttpResponse, Responder, Error};
use crate::models::review::{Review, ReviewStatus};
use crate::models::review_service::ReviewService;
use crate::models::review_repository::ReviewRepository;
use crate::models::notification_service::NotificationService;
use uuid::Uuid;

pub async fn create_review(
    review_data: web::Json<Review>,
    service: web::Data<ReviewService>
) -> impl Responder {
    let review = review_data.into_inner();
    let created_review = service.create_review(review);
    HttpResponse::Created().json(created_review)
}

pub async fn update_review(
    review_id: web::Path<Uuid>,
    review_data: web::Json<Review>,
    service: web::Data<ReviewService>
) -> impl Responder {
    let review_id = review_id.into_inner();
    let mut review = review_data.into_inner();
    review.id = review_id; 
    let updated_review = service.update_review(review);
    HttpResponse::Ok().json(updated_review)
}

pub async fn delete_review(
    review_id: web::Path<Uuid>,
    service: web::Data<ReviewService>
) -> impl Responder {
    service.delete_review(&review_id);
    HttpResponse::NoContent().finish()
}

pub async fn get_reviews_by_event_id(
    event_id: web::Path<Uuid>,
    service: web::Data<ReviewService>
) -> impl Responder {
    let reviews = service.repository.find_all()
        .into_iter()
        .filter(|r| r.event_id == *event_id)
        .collect::<Vec<_>>();
    HttpResponse::Ok().json(reviews)
}

pub async fn get_average_rating(
    event_id: web::Path<Uuid>,
    service: web::Data<ReviewService>
) -> impl Responder {
    let avg_rating = service.calculate_event_average_rating(&event_id);
    HttpResponse::Ok().json(avg_rating)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/reviews")
            .route(web::post().to(create_review))
            .route(web::put().to(update_review))
    )
    .service(
        web::resource("/reviews/{id}")
            .route(web::delete().to(delete_review))
    )
    .service(
        web::resource("/reviews/event/{eventId}")
            .route(web::get().to(get_reviews_by_event_id))
    )
    .service(
        web::resource("/reviews/average/{eventId}")
            .route(web::get().to(get_average_rating))
    );
}