use std::sync::Arc;
use chrono::{Local, NaiveDateTime, Duration};
use mockall::predicate::*;
use mockall::*;
use uuid::Uuid;

use crate::model::event::Event;
use crate::model::event::event::{CreateEventDto, UpdateEventDto};
use crate::repository::event::event_repo::EventRepository;
use crate::service::event::EventService;

// Mock for EventRepository
mock! {
    pub EventRepo {}

    impl EventRepository for EventRepo {
        fn add(&self, event: Event) -> Result<Event, String>;
        fn list_events(&self) -> Result<Vec<Event>, String>;
        fn get_by_id(&self, id: Uuid) -> Result<Option<Event>, String>;
        fn update_event(&self, id: Uuid, event: Event) -> Result<Event, String>;
        fn delete(&self, id: Uuid) -> Result<(), String>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use crate::model::event::EventStatus;
    use crate::service::event::event_service::ServiceError;

    // Helper function to create a future date for tests
    fn future_date() -> NaiveDateTime {
        Local::now().naive_local() + Duration::days(7)
    }

    // Helper function to create a valid CreateEventDto
    fn valid_create_dto() -> CreateEventDto {
        CreateEventDto {
            title: "Test Event".to_string(),
            description: "Test Description".to_string(),
            event_date: future_date(),
            location: "Test Location".to_string(),
            base_price: 10.0,
        }
    }

    // Helper function to create a sample Event
    fn sample_event() -> Event {
        let mut event = Event::new(
            "Test Event".to_string(),
            "Test Description".to_string(),
            future_date(),
            "Test Location".to_string(),
            10.0,
        );
        
        event
    }

    #[test]
    fn test_create_event_success() {
        // Arrange
        let mut mock_repo = MockEventRepo::new();
        let dto = valid_create_dto();
        let mut expected_event = Event::new(
            dto.title.clone(),
            dto.description.clone(),
            dto.event_date,
            dto.location.clone(),
            dto.base_price,
        );

        mock_repo
            .expect_add()
            .with(always())
            .returning(|event| Ok(event));

        let service = EventService::new(Arc::new(mock_repo));

        // Act
        let result = service.create_event(dto);

        // Assert
        assert!(result.is_ok());
        let created_event = result.unwrap();
        assert_eq!(created_event.title, expected_event.title);
        assert_eq!(created_event.description, expected_event.description);
        assert_eq!(created_event.event_date, expected_event.event_date);
        assert_eq!(created_event.location, expected_event.location);
        assert_eq!(created_event.base_price, expected_event.base_price);
        assert_eq!(created_event.status, EventStatus::Draft);
    }

    #[test]
    fn test_create_event_empty_title() {
        // Arrange
        let mock_repo = MockEventRepo::new();
        let mut dto = valid_create_dto();
        dto.title = "".to_string();

        let service = EventService::new(Arc::new(mock_repo));

        // Act
        let result = service.create_event(dto);

        // Assert
        assert!(result.is_err());
        match result {
            Err(ServiceError::InvalidInput(msg)) => {
                assert_eq!("Title cannot be empty", msg);
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_create_event_negative_price() {
        // Arrange
        let mock_repo = MockEventRepo::new();
        let mut dto = valid_create_dto();
        dto.base_price = -10.0;

        let service = EventService::new(Arc::new(mock_repo));

        // Act
        let result = service.create_event(dto);

        // Assert
        assert!(result.is_err());
        match result {
            Err(ServiceError::InvalidInput(msg)) => {
                assert_eq!("Price cannot be negative", msg);
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_create_event_past_date() {
        // Arrange
        let mock_repo = MockEventRepo::new();
        let mut dto = valid_create_dto();
        dto.event_date = Local::now().naive_local() - Duration::days(1);

        let service = EventService::new(Arc::new(mock_repo));

        // Act
        let result = service.create_event(dto);

        // Assert
        assert!(result.is_err());
        match result {
            Err(ServiceError::InvalidInput(msg)) => {
                assert_eq!("Event date must be in the future", msg);
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_create_event_repository_error() {
        // Arrange
        let mut mock_repo = MockEventRepo::new();
        let dto = valid_create_dto();

        mock_repo
            .expect_add()
            .with(always())
            .returning(|_| Err("Database error".to_string()));

        let service = EventService::new(Arc::new(mock_repo));

        // Act
        let result = service.create_event(dto);

        // Assert
        assert!(result.is_err());
        match result {
            Err(ServiceError::RepositoryError(msg)) => {
                assert_eq!("Database error", msg);
            }
            _ => panic!("Expected RepositoryError"),
        }
    }

    #[test]
    fn test_list_events_success() {
        // Arrange
        let mut mock_repo = MockEventRepo::new();
        let events = vec![sample_event(), sample_event()];

        mock_repo
            .expect_list_events()
            .returning(move || Ok(events.clone()));

        let service = EventService::new(Arc::new(mock_repo));

        // Act
        let result = service.list_events();

        // Assert
        assert!(result.is_ok());
        assert_eq!(2, result.unwrap().len());
    }

    #[test]
    fn test_list_events_repository_error() {
        // Arrange
        let mut mock_repo = MockEventRepo::new();

        mock_repo
            .expect_list_events()
            .returning(|| Err("Database error".to_string()));

        let service = EventService::new(Arc::new(mock_repo));

        // Act
        let result = service.list_events();

        // Assert
        assert!(result.is_err());
        match result {
            Err(ServiceError::RepositoryError(msg)) => {
                assert_eq!("Database error", msg);
            }
            _ => panic!("Expected RepositoryError"),
        }
    }

    #[test]
    fn test_get_event_success() {
        // Arrange
        let mut mock_repo = MockEventRepo::new();
        let event = sample_event();
        let event_id = "00000000-0000-0000-0000-000000000001";
        let uuid = Uuid::from_str(event_id).unwrap();

        mock_repo
            .expect_get_by_id()
            .with(eq(uuid))
            .returning({
                let event = event.clone();
                move |_| Ok(Some(event.clone()))
            });

        let service = EventService::new(Arc::new(mock_repo));

        // Act
        let result = service.get_event(event_id);

        // Assert
        assert!(result.is_ok());
        let retrieved_event = result.unwrap();
        assert_eq!(event.id, retrieved_event.id);
    }

    #[test]
    fn test_get_event_invalid_uuid() {
        // Arrange
        let mock_repo = MockEventRepo::new();
        let event_id = "invalid-uuid";

        let service = EventService::new(Arc::new(mock_repo));

        // Act
        let result = service.get_event(event_id);

        // Assert
        assert!(result.is_err());
        match result {
            Err(ServiceError::InvalidInput(msg)) => {
                assert!(msg.contains("Invalid UUID"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_get_event_not_found() {
        // Arrange
        let mut mock_repo = MockEventRepo::new();
        let event_id = "00000000-0000-0000-0000-000000000001";
        let uuid = Uuid::from_str(event_id).unwrap();

        mock_repo
            .expect_get_by_id()
            .with(eq(uuid))
            .returning(|_| Ok(None));

        let service = EventService::new(Arc::new(mock_repo));

        // Act
        let result = service.get_event(event_id);

        // Assert
        assert!(result.is_err());
        match result {
            Err(ServiceError::NotFound(msg)) => {
                assert!(msg.contains("not found"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_get_event_repository_error() {
        // Arrange
        let mut mock_repo = MockEventRepo::new();
        let event_id = "00000000-0000-0000-0000-000000000001";
        let uuid = Uuid::from_str(event_id).unwrap();

        mock_repo
            .expect_get_by_id()
            .with(eq(uuid))
            .returning(|_| Err("Database error".to_string()));

        let service = EventService::new(Arc::new(mock_repo));

        // Act
        let result = service.get_event(event_id);

        // Assert
        assert!(result.is_err());
        match result {
            Err(ServiceError::RepositoryError(msg)) => {
                assert_eq!("Database error", msg);
            }
            _ => panic!("Expected RepositoryError"),
        }
    }

    #[test]
    fn test_update_event_success() {
        // Arrange
        let mut mock_repo = MockEventRepo::new();
        let event = sample_event();
        let event_id = "00000000-0000-0000-0000-000000000001";
        let uuid = Uuid::from_str(event_id).unwrap();

        let update_dto = UpdateEventDto {
            title: Some("Updated Title".to_string()),
            description: Some("Updated Description".to_string()),
            event_date: Some(future_date()),
            location: Some("Updated Location".to_string()),
            base_price: Some(20.0),
        };

        mock_repo
            .expect_get_by_id()
            .with(eq(uuid))
            .returning(move |_| Ok(Some(event.clone())));

        mock_repo
            .expect_update_event()
            .with(eq(uuid), always())
            .returning(|_, event| Ok(event));

        let service = EventService::new(Arc::new(mock_repo));

        // Act
        let result = service.update_event(event_id, update_dto);

        // Assert
        assert!(result.is_ok());
        let updated_event = result.unwrap();
        assert_eq!("Updated Title", updated_event.title);
        assert_eq!("Updated Description", updated_event.description);
        assert_eq!("Updated Location", updated_event.location);
        assert_eq!(20.0, updated_event.base_price);
    }

    #[test]
    fn test_update_event_invalid_uuid() {
        // Arrange
        let mock_repo = MockEventRepo::new();
        let event_id = "invalid-uuid";
        let update_dto = UpdateEventDto {
            title: Some("Updated Title".to_string()),
            description: None,
            event_date: None,
            location: None,
            base_price: None,
        };

        let service = EventService::new(Arc::new(mock_repo));

        // Act
        let result = service.update_event(event_id, update_dto);

        // Assert
        assert!(result.is_err());
        match result {
            Err(ServiceError::InvalidInput(msg)) => {
                assert!(msg.contains("Invalid UUID"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_update_event_not_found() {
        // Arrange
        let mut mock_repo = MockEventRepo::new();
        let event_id = "00000000-0000-0000-0000-000000000001";
        let uuid = Uuid::from_str(event_id).unwrap();
        let update_dto = UpdateEventDto {
            title: Some("Updated Title".to_string()),
            description: None,
            event_date: None,
            location: None,
            base_price: None,
        };

        mock_repo
            .expect_get_by_id()
            .with(eq(uuid))
            .returning(|_| Ok(None));

        let service = EventService::new(Arc::new(mock_repo));

        // Act
        let result = service.update_event(event_id, update_dto);

        // Assert
        assert!(result.is_err());
        match result {
            Err(ServiceError::NotFound(msg)) => {
                assert!(msg.contains("not found"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_update_event_negative_price() {
        // Arrange
        let mut mock_repo = MockEventRepo::new();
        let event = sample_event();
        let event_id = "00000000-0000-0000-0000-000000000001";
        let uuid = Uuid::from_str(event_id).unwrap();
        let update_dto = UpdateEventDto {
            title: None,
            description: None,
            event_date: None,
            location: None,
            base_price: Some(-10.0),
        };

        mock_repo
            .expect_get_by_id()
            .with(eq(uuid))
            .returning(move |_| Ok(Some(event.clone())));

        let service = EventService::new(Arc::new(mock_repo));

        // Act
        let result = service.update_event(event_id, update_dto);

        // Assert
        assert!(result.is_err());
        match result {
            Err(ServiceError::InvalidInput(msg)) => {
                assert_eq!("Price cannot be negative", msg);
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_update_event_past_date() {
        // Arrange
        let mut mock_repo = MockEventRepo::new();
        let event = sample_event();
        let event_id = "00000000-0000-0000-0000-000000000001";
        let uuid = Uuid::from_str(event_id).unwrap();
        let update_dto = UpdateEventDto {
            title: None,
            description: None,
            event_date: Some(Local::now().naive_local() - Duration::days(1)),
            location: None,
            base_price: None,
        };

        mock_repo
            .expect_get_by_id()
            .with(eq(uuid))
            .returning(move |_| Ok(Some(event.clone())));

        let service = EventService::new(Arc::new(mock_repo));

        // Act
        let result = service.update_event(event_id, update_dto);

        // Assert
        assert!(result.is_err());
        match result {
            Err(ServiceError::InvalidInput(msg)) => {
                assert_eq!("Event date must be in the future", msg);
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_delete_event_success() {
        // Arrange
        let mut mock_repo = MockEventRepo::new();
        let event = sample_event();
        let event_id = "00000000-0000-0000-0000-000000000001";
        let uuid = Uuid::from_str(event_id).unwrap();

        mock_repo
            .expect_get_by_id()
            .with(eq(uuid))
            .returning(move |_| Ok(Some(event.clone())));

        mock_repo
            .expect_delete()
            .with(eq(uuid))
            .returning(|_| Ok(()));

        let service = EventService::new(Arc::new(mock_repo));

        // Act
        let result = service.delete_event(event_id);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_event_invalid_uuid() {
        // Arrange
        let mock_repo = MockEventRepo::new();
        let event_id = "invalid-uuid";

        let service = EventService::new(Arc::new(mock_repo));

        // Act
        let result = service.delete_event(event_id);

        // Assert
        assert!(result.is_err());
        match result {
            Err(ServiceError::InvalidInput(msg)) => {
                assert!(msg.contains("Invalid UUID"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_delete_event_not_found() {
        // Arrange
        let mut mock_repo = MockEventRepo::new();
        let event_id = "00000000-0000-0000-0000-000000000001";
        let uuid = Uuid::from_str(event_id).unwrap();

        mock_repo
            .expect_get_by_id()
            .with(eq(uuid))
            .returning(|_| Ok(None));

        let service = EventService::new(Arc::new(mock_repo));

        // Act
        let result = service.delete_event(event_id);

        // Assert
        assert!(result.is_err());
        match result {
            Err(ServiceError::NotFound(msg)) => {
                assert!(msg.contains("not found"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_publish_event_success() {
        // Arrange
        let mut mock_repo = MockEventRepo::new();
        let event = sample_event();
        let event_id = "00000000-0000-0000-0000-000000000001";
        let uuid = Uuid::from_str(event_id).unwrap();

        mock_repo
            .expect_get_by_id()
            .with(eq(uuid))
            .returning(move |_| Ok(Some(event.clone())));

        mock_repo
            .expect_update_event()
            .with(eq(uuid), always())
            .returning(|_, mut event| {
                assert_eq!(EventStatus::Published, event.status);
                Ok(event)
            });

        let service = EventService::new(Arc::new(mock_repo));

        // Act
        let result = service.publish_event(event_id);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_cancel_event_success() {
        // Arrange
        let mut mock_repo = MockEventRepo::new();
        let mut event = sample_event();
        event.publish().unwrap(); // Ensure event is published first
        let event_id = "00000000-0000-0000-0000-000000000001";
        let uuid = Uuid::from_str(event_id).unwrap();

        mock_repo
            .expect_get_by_id()
            .with(eq(uuid))
            .returning(move |_| Ok(Some(event.clone())));

        mock_repo
            .expect_update_event()
            .with(eq(uuid), always())
            .returning(|_, mut event| {
                assert_eq!(EventStatus::Cancelled, event.status);
                Ok(event)
            });

        let service = EventService::new(Arc::new(mock_repo));

        // Act
        let result = service.cancel_event(event_id);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_complete_event_success() {
        // Arrange
        let mut mock_repo = MockEventRepo::new();
        let mut event = sample_event();
        event.publish().unwrap(); // Ensure event is published first
        let event_id = "00000000-0000-0000-0000-000000000001";
        let uuid = Uuid::from_str(event_id).unwrap();

        mock_repo
            .expect_get_by_id()
            .with(eq(uuid))
            .returning(move |_| Ok(Some(event.clone())));

        mock_repo
            .expect_update_event()
            .with(eq(uuid), always())
            .returning(|_, mut event| {
                assert_eq!(EventStatus::Completed, event.status);
                Ok(event)
            });

        let service = EventService::new(Arc::new(mock_repo));

        // Act
        let result = service.complete_event(event_id);

        // Assert
        assert!(result.is_ok());
    }


}