#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, http::StatusCode, App};
    use std::sync::Arc;
    use serde_json::json;
    use crate::model::event::event::{CreateEventDto, UpdateEventDto};
    use serde_json::Value;
    use crate::service::event::event_service::{EventService, ServiceError};
    use crate::repository::event::EventRepository;
    use crate::controller::event::event_controller::{EventServiceTrait, configure_routes};
    use actix_web::{
        dev::{ServiceFactory, ServiceRequest, ServiceResponse},
        Error,
    };
    
    struct DummyEventService;

    impl EventServiceTrait for DummyEventService {
        fn create_event(&self, dto: CreateEventDto) -> Result<serde_json::Value, ServiceError> {
            Ok(json!({"id": "1", "title": dto.title}))
        }

        fn list_events(&self) -> Result<serde_json::Value, ServiceError> {
            Ok(json!([{"id": "1", "title": "Test Event"}]))
        }

        fn get_event(&self, event_id: &str) -> Result<serde_json::Value, ServiceError> {
            if event_id == "1" {
                Ok(json!({"id": "1", "title": "Test Event"}))
            } else {
                Err(ServiceError::NotFound("Event not found".into()))
            }
        }

        fn update_event(&self, _event_id: &str, _dto: UpdateEventDto) -> Result<serde_json::Value, ServiceError> {
            Ok(json!({"id": "1", "title": "Updated Event"}))
        }

        fn delete_event(&self, _event_id: &str) -> Result<(), ServiceError> {
            Ok(())
        }

        fn publish_event(&self, _event_id: &str) -> Result<serde_json::Value, ServiceError> {
            Ok(json!({"id": "1", "status": "published"}))
        }

        fn cancel_event(&self, _event_id: &str) -> Result<serde_json::Value, ServiceError> {
            Ok(json!({"id": "1", "status": "cancelled"}))
        }

        fn complete_event(&self, _event_id: &str) -> Result<serde_json::Value, ServiceError> {
            Ok(json!({"id": "1", "status": "completed"}))
        }
    }

    // Helper function untuk setup test app
fn get_test_app() -> App<impl ServiceFactory<
    ServiceRequest,
    Config = (),
    Response = ServiceResponse,
    Error = Error,
    InitError = (),
>> {
    let service = Arc::new(DummyEventService) as Arc<dyn EventServiceTrait + Send + Sync>;
    
    App::new()
        .app_data(web::Data::new(service))
        .configure(configure_routes)
}

    #[actix_web::test]
    async fn test_create_event() {
      
        let app = test::init_service(get_test_app()).await;

        // Create event request
        let create_event_payload = json!({
            "title": "Tech Talk",
            "description": "Tech Talk about ADPRO",
            "location": "Fasilkom UI",
            "event_date": "2025-12-11T18:00:00",
            "base_price": 150.0
        });

        let req = test::TestRequest::post()
            .uri("/api/events")
            .set_json(&create_event_payload)
            .to_request();

        // Kirim request dan dapatkan respons
        let resp = test::call_service(&app, req).await;

        // Assert status code adalah 201 Created
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    #[actix_web::test]
    async fn test_list_events() {
        
        let app = test::init_service(get_test_app()).await;

      
        let req = test::TestRequest::get()
            .uri("/api/events")
            .to_request();

       
        let resp = test::call_service(&app, req).await;

      
        assert_eq!(resp.status(), StatusCode::OK);
        
       
        let body = test::read_body(resp).await;
        let events: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(events.as_array().unwrap().len() > 0);
    }

    #[actix_web::test]
    async fn test_get_event() {
    
        let app = test::init_service(get_test_app()).await;

      
        let req = test::TestRequest::get()
            .uri("/api/events/1")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Get event request dengan ID event yang tidak ada
        let req_not_found = test::TestRequest::get()
            .uri("/api/events/999")
            .to_request();

        let resp_not_found = test::call_service(&app, req_not_found).await;
        assert_eq!(resp_not_found.status(), StatusCode::NOT_FOUND);
    }
    
    #[actix_web::test]
    async fn test_update_event() {
    
        let app = test::init_service(get_test_app()).await;
        
        // Update event request 
        let update_event_payload = json!({
            "title": "Updated Tech Talk",
            "description": "Updated description",
        });
        
        let req = test::TestRequest::put()
            .uri("/api/events/1")
            .set_json(&update_event_payload)
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
    
    #[actix_web::test]
    async fn test_delete_event() {
        let app = test::init_service(get_test_app()).await;
    
        let req = test::TestRequest::delete()
            .uri("/api/events/1")
            .to_request();
    
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    
        let body: Value = test::read_body_json(resp).await;
    
        assert_eq!(body["status"], "success");
        assert_eq!(body["message"], "Event dengan ID 1 berhasil dihapus");
    }
    
    #[actix_web::test]
    async fn test_publish_event() {
     
        let app = test::init_service(get_test_app()).await;
        
      
        let req = test::TestRequest::post()
            .uri("/api/events/1/publish")
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
    
    #[actix_web::test]
    async fn test_cancel_event() {
     
        let app = test::init_service(get_test_app()).await;
        
      
        let req = test::TestRequest::post()
            .uri("/api/events/1/cancel")
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
    
    #[actix_web::test]
    async fn test_complete_event() {
      
        let app = test::init_service(get_test_app()).await;
        
      
        let req = test::TestRequest::post()
            .uri("/api/events/1/complete")
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}