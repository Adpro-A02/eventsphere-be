use async_trait::async_trait;
use chrono::{NaiveDateTime, Local, Duration};
use mockall::predicate::*;
use mockall::*;
use uuid::Uuid;

use crate::model::event::{Event, EventStatus};
use crate::repository::event::EventRepository;

// Create a mock implementation for testing
mock! {
    pub EventRepo {}
    
    #[async_trait]
    impl EventRepository for EventRepo {
        fn create_event(&self, event: &Event) -> Result<(), String>;
        fn list_events(&self) -> Result<Vec<Event>, String>;
        fn update_event(&self, event_id: &str, updated_event: &Event) -> Result<(), String>;
        fn delete_event(&self, event_id: &str) -> Result<(), String>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Helper function to create a test event
    fn create_test_event() -> Event {
        let future_date = Local::now().naive_local() + Duration::days(30);
        Event {
            id: Uuid::new_v4(),
            title: String::from("Test Event"),
            description: String::from("Description for test event"),
            event_date: future_date,
            location: String::from("Test Location"),
            base_price: 100.0,
            status: EventStatus::Draft,
        }
    }
    
    // Tests for create_event
    #[tokio::test]
    async fn test_create_event_success() {
        let mut mock_repo = MockEventRepo::new();
        
        // Create test event
        let event = create_test_event();
        
        // Set up expectations
        mock_repo
            .expect_create_event()
            .with(eq(event.clone()))
            .times(1)
            .returning(|_| Ok(()));
        
        // Execute the method
        let result = mock_repo.create_event(&event);
        
        // Assert
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_create_event_failure() {
        let mut mock_repo = MockEventRepo::new();
        
        // Create test event
        let event = create_test_event();
        
        // Set up expectations for failure
        mock_repo
            .expect_create_event()
            .with(eq(event.clone()))
            .times(1)
            .returning(|_| Err(String::from("Failed to create event")));
        
        // Execute the method
        let result = mock_repo.create_event(&event);
        
        // Assert
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Failed to create event");
    }
    
    // Tests for list_events
    #[tokio::test]
    async fn test_list_events_success() {
        let mut mock_repo = MockEventRepo::new();
        
        // Create events for the expected result
        let event1 = create_test_event();
        let event2 = {
            let mut e = create_test_event();
            e.title = String::from("Another Event");
            e
        };
        let expected_events = vec![event1, event2];
        
        // Set up expectations
        mock_repo
            .expect_list_events()
            .times(1)
            .returning(move || Ok(expected_events.clone()));
        
        // Execute the method
        let result = mock_repo.list_events();
        
        // Assert
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].title, "Test Event");
        assert_eq!(events[1].title, "Another Event");
    }
    
    #[tokio::test]
    async fn test_list_events_empty() {
        let mut mock_repo = MockEventRepo::new();
        
        // Set up expectations for empty list
        mock_repo
            .expect_list_events()
            .times(1)
            .returning(|| Ok(vec![]));
        
        // Execute the method
        let result = mock_repo.list_events();
        
        // Assert
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 0);
    }
    
    #[tokio::test]
    async fn test_list_events_failure() {
        let mut mock_repo = MockEventRepo::new();
        
        // Set up expectations for failure
        mock_repo
            .expect_list_events()
            .times(1)
            .returning(|| Err(String::from("Database connection error")));
        
        // Execute the method
        let result = mock_repo.list_events();
        
        // Assert
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Database connection error");
    }
    
    // Tests for update_event
    #[tokio::test]
    async fn test_update_event_success() {
        let mut mock_repo = MockEventRepo::new();
        
        // Create test event
        let event = create_test_event();
        let event_id = event.id.to_string();
        
        // Set up expectations
        mock_repo
            .expect_update_event()
            .with(eq(event_id.clone()), eq(event.clone()))
            .times(1)
            .returning(|_, _| Ok(()));
        
        // Execute the method
        let result = mock_repo.update_event(&event_id, &event);
        
        // Assert
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_update_nonexistent_event() {
        let mut mock_repo = MockEventRepo::new();
        
        // Create test event
        let event = create_test_event();
        let nonexistent_id = Uuid::new_v4().to_string();
        
        // Set up expectations for non-existent event
        mock_repo
            .expect_update_event()
            .with(eq(nonexistent_id.clone()), eq(event.clone()))
            .times(1)
            .returning(|_, _| Err(String::from("Event not found")));
        
        // Execute the method
        let result = mock_repo.update_event(&nonexistent_id, &event);
        
        // Assert
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Event not found");
    }
    
    #[tokio::test]
    async fn test_update_event_invalid_id_format() {
        let mut mock_repo = MockEventRepo::new();
        
        // Create test event
        let event = create_test_event();
        let invalid_id = "not-a-uuid";
        
        // Set up expectations for invalid ID format
        mock_repo
            .expect_update_event()
            .with(eq(invalid_id), eq(event.clone()))
            .times(1)
            .returning(|_, _| Err(String::from("Invalid UUID format")));
        
        // Execute the method
        let result = mock_repo.update_event(invalid_id, &event);
        
        // Assert
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid UUID format");
    }
    
    // Tests for delete_event
    #[tokio::test]
    async fn test_delete_event_success() {
        let mut mock_repo = MockEventRepo::new();
        
        // Create test event ID
        let event_id = Uuid::new_v4().to_string();
        
        // Set up expectations
        mock_repo
            .expect_delete_event()
            .with(eq(event_id.clone()))
            .times(1)
            .returning(|_| Ok(()));
        
        // Execute the method
        let result = mock_repo.delete_event(&event_id);
        
        // Assert
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_delete_nonexistent_event() {
        let mut mock_repo = MockEventRepo::new();
        
        // Create test event ID
        let nonexistent_id = Uuid::new_v4().to_string();
        
        // Set up expectations for non-existent event
        mock_repo
            .expect_delete_event()
            .with(eq(nonexistent_id.clone()))
            .times(1)
            .returning(|_| Err(String::from("Event not found")));
        
        // Execute the method
        let result = mock_repo.delete_event(&nonexistent_id);
        
        // Assert
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Event not found");
    }
    
    #[tokio::test]
    async fn test_delete_event_invalid_id_format() {
        let mut mock_repo = MockEventRepo::new();
        
        // Invalid UUID
        let invalid_id = "not-a-uuid";
        
        // Set up expectations for invalid ID format
        mock_repo
            .expect_delete_event()
            .with(eq(invalid_id))
            .times(1)
            .returning(|_| Err(String::from("Invalid UUID format")));
        
        // Execute the method
        let result = mock_repo.delete_event(invalid_id);
        
        // Assert
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid UUID format");
    }
}