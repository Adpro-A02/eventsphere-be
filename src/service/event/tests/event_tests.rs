#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::model::event::{Event, EventStatus};
    use crate::repository::event::EventRepository;
    use crate::service::event::event_service::{DefaultEventService, EventService, ServiceError};
    
    use mockall::{mock, predicate::*};
    
    use uuid::Uuid;
    use chrono::{Duration, Local, NaiveDateTime};
    use super::*;
    use chrono::NaiveDate;
    
    fn sample_event_data() -> (String, String, NaiveDateTime, String, f64) {
        (
            "Sample Event".to_string(),
            "This is a sample description".to_string(),
            NaiveDate::from_ymd_opt(2025, 5, 1).unwrap().and_hms_opt(10, 0, 0).unwrap(),
            "Jakarta".to_string(),
            100.0,
        )
    }

    fn setup_service() -> DefaultEventService<MockRepo> {
        DefaultEventService::new(Arc::new(MockRepo {}))
    }

    struct MockRepo;
    impl EventRepository for MockRepo {
        fn create_event(&self, event: &Event) -> Result<(), String> {
            todo!()
        }
    
        fn list_events(&self) -> Result<Vec<Event>, String> {
            todo!()
        }
    
        fn update_event(&self, event_id: &str, updated_event: &Event) -> Result<(), String> {
            todo!()
        }
    
        fn delete_event(&self, event_id: &str) -> Result<(), String> {
            todo!()
        }
    }

    #[test]
    fn test_create_event() {
        let mut service = setup_service();
        let (title, description, date, location, price) = sample_event_data();

        let result = service.create_event(title.clone(), description.clone(), date, location.clone(), price);
        assert!(result.is_ok());

        let event = result.unwrap();
        assert_eq!(event.title, title);
        assert_eq!(event.location, location);
        assert_eq!(event.base_price, price);
    }

    #[test]
    fn test_list_events() {
        let mut service = setup_service();
        let data = sample_event_data();
        service.create_event(data.0, data.1, data.2, data.3, data.4).unwrap();

        let list = service.list_events().unwrap();
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_get_event_success() {
        let mut service = setup_service();
        let data = sample_event_data();
        let created = service.create_event(data.0, data.1, data.2, data.3, data.4).unwrap();

        let found = service.get_event(&created.id.to_string());
        assert!(found.is_ok());
        assert_eq!(found.unwrap().id, created.id);
    }

    #[test]
    fn test_get_event_not_found() {
        let service = setup_service();
        let not_found = service.get_event(&Uuid::new_v4().to_string());
        assert!(matches!(not_found, Err(ServiceError::NotFound)));
    }

    #[test]
    fn test_update_event() {
        let mut service = setup_service();
        let data = sample_event_data();
        let created = service.create_event(data.0.clone(), data.1.clone(), data.2, data.3.clone(), data.4).unwrap();

        let updated = Event {
            id: created.id,
            title: "Updated Title".to_string(),
            description: "Updated Desc".to_string(),
            event_date: data.2,
            location: "Bandung".to_string(),
            base_price: 200.0,
            status: EventStatus::Published,
        };

        let result = service.update_event(&created.id.to_string(), updated.clone());
        assert!(result.is_ok());

        let event = result.unwrap();
        assert_eq!(event.title, "Updated Title");
        assert_eq!(event.status, EventStatus::Published);
    }

    #[test]
    fn test_delete_event_success() {
        let mut service = setup_service();
        let data = sample_event_data();
        let created = service.create_event(data.0, data.1, data.2, data.3, data.4).unwrap();

        let result = service.delete_event(&created.id.to_string());
        assert!(result.is_ok());

        let get_after = service.get_event(&created.id.to_string());
        assert!(get_after.is_err());
    }

    #[test]
    fn test_delete_event_not_found() {
        let mut service = setup_service();
        let result = service.delete_event(&Uuid::new_v4().to_string());
        assert!(matches!(result, Err(ServiceError::NotFound)));
    }
}