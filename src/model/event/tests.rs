use uuid::Uuid;
use chrono::NaiveDateTime;
use crate::model::event::{Event, EventStatus};

#[cfg(test)]
mod tests {
    use super::*;
    
    // Helper function to create a default event
    fn create_default_event() -> Event {
        let id = Uuid::new_v4();
        let title = String::from("Tech Talk");
        let description = String::from("Tech Talk about Rust");
        let event_date = NaiveDateTime::parse_from_str("2025-05-01 18:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let location = String::from("Fasilkom UI");
        let base_price = 100.0;
        let status = EventStatus::Draft;

        Event {
            id,
            title,
            description,
            event_date,
            location,
            base_price,
            status,
        }
    }

    #[test]
    fn test_create_event() {
        let event = create_default_event();
        
        assert_eq!(event.title, "Tech Talk");
        assert_eq!(event.description, "Tech Talk about Rust");
        assert_eq!(event.location, "Fasilkom UI");
        assert_eq!(event.base_price, 100.0);
        
        // Test that the enum variant matches
        match event.status {
            EventStatus::Draft => assert!(true),
            _ => assert!(false, "Status should be Draft"),
        }
    }
    
    #[test]
    fn test_event_with_zero_price() {
        let mut event = create_default_event();
        event.base_price = 0.0;
        
        assert_eq!(event.base_price, 0.0);
    }
    
    #[test]
    fn test_event_with_future_date() {
        let future_date = NaiveDateTime::parse_from_str("2026-10-15 14:30:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let mut event = create_default_event();
        event.event_date = future_date;
        
        assert_eq!(event.event_date, future_date);
    }
    
    #[test]
    fn test_different_event_statuses() {
        let mut event = create_default_event();
        
        // Test Draft status
        assert!(matches!(event.status, EventStatus::Draft));
        
        // Test Published status
        event.status = EventStatus::Published;
        assert!(matches!(event.status, EventStatus::Published));
        
        // Test Completed status
        event.status = EventStatus::Completed;
        assert!(matches!(event.status, EventStatus::Completed));
        
        // Test Cancelled status
        event.status = EventStatus::Cancelled;
        assert!(matches!(event.status, EventStatus::Cancelled));
    }
    
    #[test]
    fn test_event_with_empty_fields() {
        let mut event = create_default_event();
        
        // Test with empty title
        event.title = String::from("");
        assert_eq!(event.title, "");
        
        // Test with empty description
        event.description = String::from("");
        assert_eq!(event.description, "");
        
        // Test with empty location
        event.location = String::from("");
        assert_eq!(event.location, "");
    }
    
    #[test]
    fn test_event_equality() {
        let event1 = create_default_event();
        let mut event2 = event1.clone();
        
        // Modified event should not match the original in a business logic sense
        // even though they have the same ID
        event2.title = String::from("Modified Title");
        assert_ne!(event1.title, event2.title);
        assert_eq!(event1.id, event2.id);
    }
    
    #[test]
    fn test_negative_price() {
        let mut event = create_default_event();
        event.base_price = -50.0;
        
        // This just verifies we can set a negative price
        // In a real app, you might want validation to prevent this
        assert_eq!(event.base_price, -50.0);
    }
}