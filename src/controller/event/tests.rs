#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::blocking::Client;
    use rocket::http::Status;
    use rocket::serde::json::{json, Value};
    use rocket::routes;
    use std::sync::Arc;
    use crate::model::event::event::{CreateEventDto, UpdateEventDto};
    use crate::service::event::event_service::{EventService, ServiceError};
    use crate::repository::event::EventRepository;
    use crate::controller::event::event_controller::{EventServiceTrait, create_event,list_events,get_event};
    
    // Dummy event service and repository to be used in the tests
    struct DummyEventService;

    impl EventServiceTrait for DummyEventService {
        fn create_event(&self, dto: CreateEventDto) -> Result<Value, ServiceError> {
            Ok(json!({"id": "1", "title": dto.title}))
        }

        fn list_events(&self) -> Result<Value, ServiceError> {
            Ok(json!([{"id": "1", "title": "Test Event"}]))
        }

        fn get_event(&self, event_id: &str) -> Result<Value, ServiceError> {
            if event_id == "1" {
                Ok(json!({"id": "1", "title": "Test Event"}))
            } else {
                Err(ServiceError::NotFound("Event not found".into()))
            }
        }

        fn update_event(&self, _event_id: &str, _dto: UpdateEventDto) -> Result<Value, ServiceError> {
            Ok(json!({"id": "1", "title": "Updated Event"}))
        }

        fn delete_event(&self, _event_id: &str) -> Result<(), ServiceError> {
            Ok(())
        }

        fn publish_event(&self, _event_id: &str) -> Result<Value, ServiceError> {
            Ok(json!({"id": "1", "status": "published"}))
        }

        fn cancel_event(&self, _event_id: &str) -> Result<Value, ServiceError> {
            Ok(json!({"id": "1", "status": "cancelled"}))
        }

        fn complete_event(&self, _event_id: &str) -> Result<Value, ServiceError> {
            Ok(json!({"id": "1", "status": "completed"}))
        }
    }

    #[test]
    fn test_create_event() {
        let rocket = rocket::build()
            .manage(Arc::new(DummyEventService) as Arc<dyn EventServiceTrait + Send + Sync>)  // Use `.manage()` here
            .mount("/api", routes![create_event]);

        let client = Client::tracked(rocket).expect("valid rocket instance");

        // Create event request
        let create_event_payload = json!({
            "title": "Tech Talk",
            "description": "Tech Talk about ADPRO",
            "location": "Fasilkom UI",
            "event_date": "2025-12-11T18:00:00",
            "base_price": 150.0
          
        });

        let response = client.post("/api/events")
            .json(&create_event_payload)
            .dispatch();

        // Assert that the event is created and return status code 201
        assert_eq!(response.status(), Status::Created);
    }

    #[test]
    fn test_list_events() {
        let rocket = rocket::build()
            .manage(Arc::new(DummyEventService) as Arc<dyn EventServiceTrait + Send + Sync>)  // Use `.manage()` here
            .mount("/api", routes![list_events]);

        let client = Client::tracked(rocket).expect("valid rocket instance");

        // List events request
        let response = client.get("/api/events").dispatch();

        // Assert that the response status is OK
        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    fn test_get_event() {
        let rocket = rocket::build()
            .manage(Arc::new(DummyEventService) as Arc<dyn EventServiceTrait + Send + Sync>)  // Use `.manage()` here
            .mount("/api", routes![get_event]);

        let client = Client::tracked(rocket).expect("valid rocket instance");

        // Get event request with existing event ID
        let response = client.get("/api/events/1").dispatch();
        assert_eq!(response.status(), Status::Ok);

        // Get event request with non-existing event ID
        let response_not_found = client.get("/api/events/999").dispatch();
        assert_eq!(response_not_found.status(), Status::NotFound);
    }
}