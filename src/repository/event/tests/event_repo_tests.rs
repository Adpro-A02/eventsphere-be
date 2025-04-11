#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{model::event::{Event, EventStatus}, repository::event::{EventRepository, InMemoryEventRepo}};

    use super::*;
    use uuid::Uuid;
    use chrono::NaiveDateTime;
    fn test_create_event() -> Event {
        Event {
            id: Uuid::new_v4(),
            title: "Contoh Event".to_string(),
            description: "Deskripsi".to_string(),
            event_date: chrono::Utc::now().naive_utc(),
            location: "Jakarta".to_string(),
            base_price: 100.0,
            status: EventStatus::Draft,
        }
    }
    
    

    #[test]
    fn test_add_event_success() {
        let repo = InMemoryEventRepo::new();
        let event = test_create_event();
        
        let result = repo.add(event.clone());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, event.id);
    }

    #[test]
    fn test_add_event_duplicate() {
        let repo = InMemoryEventRepo::new();
        let event = test_create_event();
        
        repo.add(event.clone()).unwrap();
        let result = repo.add(event);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Event already exists");
    }

    #[test]
    fn test_get_by_id_found() {
        let repo = InMemoryEventRepo::new();
        let event = test_create_event();
        repo.add(event.clone()).unwrap();
        
        let result = repo.get_by_id(event.id);
        
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_get_by_id_not_found() {
        let repo = InMemoryEventRepo::new();
        let result = repo.get_by_id(Uuid::new_v4());
        
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_list_events() {
        let repo = InMemoryEventRepo::new();
        let event1 = test_create_event();
        let event2 = test_create_event();
        
        repo.add(event1.clone()).unwrap();
        repo.add(event2.clone()).unwrap();
        
        let result = repo.list_events();
        
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 2);
        assert!(events.iter().any(|e| e.id == event1.id));
        assert!(events.iter().any(|e| e.id == event2.id));
    }

    #[test]
    fn test_list_events_empty() {
        let repo = InMemoryEventRepo::new();
        let result = repo.list_events();
        
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_update_event_success() {
        let repo = InMemoryEventRepo::new();
        let mut event = test_create_event();
        repo.add(event.clone()).unwrap();
        
        event.title = "Updated Title".to_string();
        let result = repo.update_event(event.id, event.clone());
        
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.title, "Updated Title");
        
        // Verify the update persisted
        let fetched = repo.get_by_id(event.id).unwrap().unwrap();
        assert_eq!(fetched.title, "Updated Title");
    }

    #[test]
    fn test_update_event_not_found() {
        let repo = InMemoryEventRepo::new();
        let event = test_create_event();
        
        let result = repo.update_event(event.id, event);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Event not found");
    }

    #[test]
    fn test_delete_event_success() {
        let repo = InMemoryEventRepo::new();
        let event = test_create_event();
        repo.add(event.clone()).unwrap();
        
        let result = repo.delete(event.id);
        
        assert!(result.is_ok());
        assert!(repo.get_by_id(event.id).unwrap().is_none());
    }

    #[test]
    fn test_delete_event_not_found() {
        let repo = InMemoryEventRepo::new();
        let result = repo.delete(Uuid::new_v4());
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Event not found");
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;
        
        let repo = Arc::new(InMemoryEventRepo::new());
        let event = test_create_event();
        
        // Add from main thread
        repo.add(event.clone()).unwrap();
        
        // Spawn a thread to read
        let repo_clone = Arc::clone(&repo);
        let handle = thread::spawn(move || {
            repo_clone.get_by_id(event.id)
        });
        
        let result = handle.join().unwrap();
        
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }
}