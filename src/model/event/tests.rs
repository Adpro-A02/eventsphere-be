#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Local};
    use crate::model::event::{Event, EventStatus}; // Adjust the path based on your project structure

    #[test]
    fn test_new_event() {
        let title = "Concert".to_string();
        let description = "A music concert".to_string();
        let event_date = Local::now().naive_local() + Duration::days(10);
        let location = "Jakarta".to_string();
        let base_price = 100.0;

        let event = Event::new(
            title.clone(),
            description.clone(),
            event_date,
            location.clone(),
            base_price,
        );

        assert_eq!(event.title, title);
        assert_eq!(event.description, description);
        assert_eq!(event.event_date, event_date);
        assert_eq!(event.location, location);
        assert_eq!(event.base_price, base_price);
        assert_eq!(event.status, EventStatus::Draft);
    }

    #[test]
    fn test_update_event() {
        let mut event = create_test_event();
        
        let new_title = "Updated Concert".to_string();
        let new_description = "Updated description".to_string();
        let new_event_date = Local::now().naive_local() + Duration::days(20);
        let new_location = "Bandung".to_string();
        let new_base_price = 150.0;

        event.update(
            Some(new_title.clone()),
            Some(new_description.clone()),
            Some(new_event_date),
            Some(new_location.clone()),
            Some(new_base_price),
        );

        assert_eq!(event.title, new_title);
        assert_eq!(event.description, new_description);
        assert_eq!(event.event_date, new_event_date);
        assert_eq!(event.location, new_location);
        assert_eq!(event.base_price, new_base_price);
    }

    #[test]
    fn test_partial_update_event() {
        let mut event = create_test_event();
        let original_title = event.title.clone();
        let original_description = event.description.clone();
        let original_event_date = event.event_date;
        
        let new_location = "Surabaya".to_string();
        let new_base_price = 200.0;

        event.update(
            None,
            None,
            None,
            Some(new_location.clone()),
            Some(new_base_price),
        );

        assert_eq!(event.title, original_title);
        assert_eq!(event.description, original_description);
        assert_eq!(event.event_date, original_event_date);
        assert_eq!(event.location, new_location);
        assert_eq!(event.base_price, new_base_price);
    }

    #[test]
    fn test_change_status() {
        let mut event = create_test_event();
        
        event.change_status(EventStatus::Published);
        assert_eq!(event.status, EventStatus::Published);
        
        event.change_status(EventStatus::Cancelled);
        assert_eq!(event.status, EventStatus::Cancelled);
        
        event.change_status(EventStatus::Completed);
        assert_eq!(event.status, EventStatus::Completed);
        
        event.change_status(EventStatus::Draft);
        assert_eq!(event.status, EventStatus::Draft);
    }

    #[test]
    fn test_publish_success() {
        let mut event = create_test_event();
        
        let result = event.publish();
        assert!(result.is_ok());
        assert_eq!(event.status, EventStatus::Published);
    }

    #[test]
    fn test_publish_empty_title() {
        let mut event = create_test_event();
        event.title = "".to_string();
        
        let result = event.publish();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Event title cannot be empty");
        assert_eq!(event.status, EventStatus::Draft);
    }

    #[test]
    fn test_publish_past_date() {
        let mut event = create_test_event();
        event.event_date = Local::now().naive_local() - Duration::days(1);
        
        let result = event.publish();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Event date must be in the future");
        assert_eq!(event.status, EventStatus::Draft);
    }

    #[test]
    fn test_cancel_success() {
        let mut event = create_test_event();
        event.status = EventStatus::Published;
        
        let result = event.cancel();
        assert!(result.is_ok());
        assert_eq!(event.status, EventStatus::Cancelled);
    }

    #[test]
    fn test_cancel_completed_event() {
        let mut event = create_test_event();
        event.status = EventStatus::Completed;
        
        let result = event.cancel();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot cancel a completed event");
        assert_eq!(event.status, EventStatus::Completed);
    }

    #[test]
    fn test_complete_success() {
        let mut event = create_test_event();
        event.status = EventStatus::Published;
        
        let result = event.complete();
        assert!(result.is_ok());
        assert_eq!(event.status, EventStatus::Completed);
    }

    #[test]
    fn test_complete_non_published_event() {
        let mut event = create_test_event();
        // Event is in Draft status
        
        let result = event.complete();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Only published events can be marked as completed");
        assert_eq!(event.status, EventStatus::Draft);
        
        // Try with cancelled event
        event.status = EventStatus::Cancelled;
        let result = event.complete();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Only published events can be marked as completed");
        assert_eq!(event.status, EventStatus::Cancelled);
    }

    #[test]
    fn test_is_free() {
        let mut event = create_test_event();
        assert!(!event.is_free());
        
        event.base_price = 0.0;
        assert!(event.is_free());
    }

    #[test]
    fn test_is_err() {
        let mut event = create_test_event();
        assert!(!event.is_err());
        
        event.base_price = -10.0;
        assert!(event.is_err());
    }

    // Helper function to create a test event
    fn create_test_event() -> Event {
        Event::new(
            "Test Event".to_string(),
            "Test Description".to_string(),
            Local::now().naive_local() + Duration::days(10),
            "Test Location".to_string(),
            100.0,
        )
    }
}