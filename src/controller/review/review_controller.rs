use actix_web::{web, HttpResponse, Responder};
use uuid::Uuid;
use crate::model::review::{Review, ReviewStatus};
use crate::service::review::review_service::{ReviewService, ServiceError};
use crate::repository::review::review_repository::ReviewRepository;
use crate::service::review::notification_service::NotificationService;
use std::sync::Arc;
use serde::{Deserialize};

// Define DTOs for creating and updating reviews
#[derive(Deserialize)]
pub struct CreateReviewDto {
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub comment: String,
}

#[derive(Deserialize)]
pub struct UpdateReviewDto {
    pub rating: i32,
    pub comment: String,
}

// Directly use the concrete type (no more trait or dynamic dispatch)
pub type ReviewServiceArc<R> = Arc<ReviewService<R>>;

// Helper function to map service errors to Actix responses
fn map_error_to_response(error: ServiceError) -> HttpResponse {
    match error {
        ServiceError::NotFound(msg) => HttpResponse::NotFound().json(serde_json::json!( {
            "status": "error",
            "message": msg
        })),
        ServiceError::InvalidInput(msg) => HttpResponse::BadRequest().json(serde_json::json!( {
            "status": "error",
            "message": msg
        })),
        ServiceError::RepositoryError(msg) => HttpResponse::InternalServerError().json(serde_json::json!( {
            "status": "error",
            "message": format!("Database error: {}", msg)
        })),
        ServiceError::InternalError(msg) => HttpResponse::InternalServerError().json(serde_json::json!( {
            "status": "error",
            "message": format!("Internal server error: {}", msg)
        })),
    }
}

// Create a new review
async fn create_review<R: ReviewRepository>(
    service: web::Data<ReviewServiceArc<R>>,
    body: web::Json<CreateReviewDto>,
) -> impl Responder {
    match service.create_review(
        body.event_id,                
        body.user_id,               
        body.rating,               
        body.comment.clone()         
    ) {
        Ok(review) => {
            let id = review.review_id.to_string();
            let location = format!("/api/reviews/{}", id);

            HttpResponse::Created()
                .insert_header(("Location", location))
                .json(review)
        },
        Err(e) => map_error_to_response(e),
    }
}

async fn list_reviews_by_event<R: ReviewRepository>(
    service: web::Data<ReviewServiceArc<R>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let event_id = path.into_inner();
    match service.list_reviews_by_event(event_id) {
        Ok(reviews) => HttpResponse::Ok().json(reviews),
        Err(e) => map_error_to_response(e),
    }
}

async fn get_review<R: ReviewRepository>(
    service: web::Data<ReviewServiceArc<R>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let review_id = path.into_inner();
    match service.get_review(review_id) {
        Ok(review) => HttpResponse::Ok().json(review),
        Err(e) => map_error_to_response(e),
    }
}

async fn update_review<R: ReviewRepository>(
    service: web::Data<ReviewServiceArc<R>>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateReviewDto>,
) -> impl Responder {
    let review_id = path.into_inner();
    match service.update_review(review_id, body.rating, body.comment.clone()) { // Clone the comment
        Ok(review) => HttpResponse::Ok().json(review),
        Err(e) => map_error_to_response(e),
    }
}

async fn delete_review<R: ReviewRepository>(
    service: web::Data<ReviewServiceArc<R>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let review_id = path.into_inner();
    match service.delete_review(review_id) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!( {
            "status": "success",
            "message": format!("Review with ID {} successfully deleted", review_id)
        })),
        Err(e) => map_error_to_response(e),
    }
}

async fn approve_review<R: ReviewRepository>(
    service: web::Data<ReviewServiceArc<R>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let review_id = path.into_inner();
    match service.approve_review(review_id) {
        Ok(review) => HttpResponse::Ok().json(review),
        Err(e) => map_error_to_response(e),
    }
}

async fn reject_review<R: ReviewRepository>(
    service: web::Data<ReviewServiceArc<R>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let review_id = path.into_inner();
    match service.reject_review(review_id) {
        Ok(review) => HttpResponse::Ok().json(review),
        Err(e) => map_error_to_response(e),
    }
}

pub fn configure_routes<R: ReviewRepository>(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::resource("/reviews")
                    .route(web::post().to(create_review::<R>))
            )
            .service(
                web::resource("/reviews/{review_id}")
                    .route(web::get().to(get_review::<R>))
                    .route(web::put().to(update_review::<R>))
                    .route(web::delete().to(delete_review::<R>))
            )
            .service(
                web::resource("/reviews/{review_id}/approve")
                    .route(web::post().to(approve_review::<R>))
            )
            .service(
                web::resource("/reviews/{review_id}/reject")
                    .route(web::post().to(reject_review::<R>))
            )
            .service(
                web::resource("/reviews/events/{event_id}")
                    .route(web::get().to(list_reviews_by_event::<R>))
            )
    );
}
