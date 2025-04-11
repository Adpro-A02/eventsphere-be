use uuid::Uuid;
use chrono::NaiveDateTime;
use crate::model::event::{Event, EventStatus};

#[cfg(test)]
mod tests {
    use super::*;
    
    
    fn create_default_event() -> Event {
        let id = Uuid::new_v4();
        let title = String::from("Tech Talk");
        let description = String::from("Tech Talk about ADPRO");
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
        
       
        assert!(matches!(event.status, EventStatus::Draft));
      
        event.status = EventStatus::Published;
        assert!(matches!(event.status, EventStatus::Published));
        
        
        event.status = EventStatus::Completed;
        assert!(matches!(event.status, EventStatus::Completed));
        
        
        event.status = EventStatus::Cancelled;
        assert!(matches!(event.status, EventStatus::Cancelled));
    }
    
    #[test]
    fn test_event_with_empty_fields() {
        let mut event = create_default_event();
        
        
        event.title = String::from("");
        assert_eq!(event.title, "");
        
      
        event.description = String::from("");
        assert_eq!(event.description, "");
        
     
        event.location = String::from("");
        assert_eq!(event.location, "");
    }
    
    #[test]
    fn test_event_equality() {
        let event1 = create_default_event();
        let mut event2 = event1.clone();
        
       
        event2.title = String::from("Modified Title");
        assert_ne!(event1.title, event2.title);
        assert_eq!(event1.id, event2.id);
    }
    
    #[test]
    fn test_negative_price() {
        let mut event = create_default_event();
        event.base_price = -50.0;
        
        
        assert_eq!(event.base_price, -50.0);
    }
    fn test_update_event() {
        let mut event = create_test_event();
        let new_date = chrono::Local::now().naive_local() + Duration::days(60);
        
        event.update(
            Some(String::from("Updated Tech Talk")),
            Some(String::from("Updated description")),
            Some(new_date),
            Some(String::from("New Location")),
            Some(150.0)
        );
        
        assert_eq!(event.title, "Updated Tech Talk");
        assert_eq!(event.description, "Updated description");
        assert_eq!(event.location, "New Location");
        assert_eq!(event.base_price, 150.0);
        assert_eq!(event.event_date, new_date);
    }
    
    #[test]
    fn test_partial_update_event() {
        let mut event = create_test_event();
        let original_date = event.event_date;
        let original_location = event.location.clone();
        
      
        event.update(
            Some(String::from("Updated Title Only")),
            None,
            None,
            None,
            Some(200.0)
        );
        
        assert_eq!(event.title, "Updated Title Only");
        assert_eq!(event.description, "Tech Talk about ADPRO"); 
        assert_eq!(event.location, original_location); 
        assert_eq!(event.base_price, 200.0);
        assert_eq!(event.event_date, original_date); 
    }
    
    #[test]
    fn test_change_status() {
        let mut event = create_test_event();
        
        event.change_status(EventStatus::Published);
        assert!(matches!(event.status, EventStatus::Published));
        
        event.change_status(EventStatus::Cancelled);
        assert!(matches!(event.status, EventStatus::Cancelled));
    }
    
    #[test]
    fn test_publish_event() {
        let mut event = create_test_event();
        
        let result = event.publish();
        assert!(result.is_ok());
        assert!(matches!(event.status, EventStatus::Published));
    }
    
    #[test]
    fn test_publish_event_with_empty_title() {
        let mut event = create_test_event();
        event.title = String::from("");
        
        let result = event.publish();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Event title cannot be empty");
        assert!(matches!(event.status, EventStatus::Draft)); 
    }
    
    #[test]
    fn test_publish_event_with_past_date() {
        let mut event = create_test_event();
        let past_date = chrono::Local::now().naive_local() - Duration::days(1);
        event.event_date = past_date;
        
        let result = event.publish();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Event date must be in the future");
        assert!(matches!(event.status, EventStatus::Draft)); 
    }
    
    #[test]
    fn test_cancel_event() {
        let mut event = create_test_event();
        event.change_status(EventStatus::Published);
        
        let result = event.cancel();
        assert!(result.is_ok());
        assert!(matches!(event.status, EventStatus::Cancelled));
    }
    
    #[test]
    fn test_cancel_completed_event() {
        let mut event = create_test_event();
        event.change_status(EventStatus::Completed);
        
        let result = event.cancel();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot cancel a completed event");
        assert!(matches!(event.status, EventStatus::Completed)); 
    }
    
    #[test]
    fn test_complete_event() {
        let mut event = create_test_event();
        event.change_status(EventStatus::Published);
        
        let result = event.complete();
        assert!(result.is_ok());
        assert!(matches!(event.status, EventStatus::Completed));
    }
    
    #[test]
    fn test_complete_draft_event() {
        let mut event = create_test_event();
         
        
        let result = event.complete();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Only published events can be marked as completed");
        assert!(matches!(event.status, EventStatus::Draft)); 
    }
    
    #[test]
    fn test_is_free() {
        let mut event = create_test_event();
        assert!(!event.is_free());  
        
        event.base_price = 0.0;
        assert!(event.is_free()); 
        
        event.base_price = -1.0;
        assert!(event.is_err()); 
    }
}