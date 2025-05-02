#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::{model::event::{Event, EventStatus}, repository::event::{EventRepository, InMemoryEventRepository}};
    use chrono::{Duration, Local, NaiveDateTime};
    use uuid::Uuid;

    // Helper function to create a test event
    fn create_test_event(title: &str) -> Event {
        Event::new(
            title.to_string(),
            "Test Description".to_string(),
            Local::now().naive_local() + Duration::days(10),
            "Test Location".to_string(),
            100.0,
        )
    }

    // Helper function to create a repository with some test events
    fn create_test_repository_with_events(count: usize) -> (InMemoryEventRepository, Vec<Event>) {
        let repo = InMemoryEventRepository::new();
        let mut events = Vec::new();

        for i in 0..count {
            let event = create_test_event(&format!("Test Event {}", i));
            repo.add(event.clone()).unwrap();
            events.push(event);
        }

        (repo, events)
    }

    #[test]
    fn test_add_event() {
        let repo = InMemoryEventRepository::new();
        let event = create_test_event("Test Event");

        let result = repo.add(event.clone());
        assert!(result.is_ok());
        
        let added_event = result.unwrap();
        assert_eq!(added_event.id, event.id);
        assert_eq!(added_event.title, event.title);

        // Verify the event was actually added to the repository
        let events = repo.list_events().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, event.id);
    }

    #[test]
    fn test_delete_event_success() {
        let (repo, events) = create_test_repository_with_events(3);
        let event_id = events[1].id;

        // Delete the second event
        let result = repo.delete(event_id);
        assert!(result.is_ok());

        // Verify the event was deleted
        let remaining_events = repo.list_events().unwrap();
        assert_eq!(remaining_events.len(), 2);
        assert!(!remaining_events.iter().any(|e| e.id == event_id));
    }

    #[test]
    fn test_delete_event_not_found() {
        let (repo, _) = create_test_repository_with_events(2);
        let non_existent_id = Uuid::new_v4();

        // Try to delete a non-existent event
        let result = repo.delete(non_existent_id);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            format!("Event with ID {} not found", non_existent_id)
        );

        // Verify no events were deleted
        let events = repo.list_events().unwrap();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_update_event_success() {
        let (repo, events) = create_test_repository_with_events(3);
        let event_id = events[1].id;

        // Create an updated version of the event
        let mut updated_event = events[1].clone();
        updated_event.title = "Updated Title".to_string();
        updated_event.description = "Updated Description".to_string();
        updated_event.base_price = 200.0;

        // Update the event
        let result = repo.update_event(event_id, updated_event.clone());
        assert!(result.is_ok());
        
        let returned_event = result.unwrap();
        assert_eq!(returned_event.id, event_id);
        assert_eq!(returned_event.title, "Updated Title");
        assert_eq!(returned_event.description, "Updated Description");
        assert_eq!(returned_event.base_price, 200.0);

        // Verify the event was actually updated in the repository
        let retrieved_event = repo.get_by_id(event_id).unwrap().unwrap();
        assert_eq!(retrieved_event.title, "Updated Title");
        assert_eq!(retrieved_event.description, "Updated Description");
        assert_eq!(retrieved_event.base_price, 200.0);
    }

    #[test]
    fn test_update_event_not_found() {
        let (repo, events) = create_test_repository_with_events(2);
        let non_existent_id = Uuid::new_v4();
        
        // Try to update a non-existent event
        let result = repo.update_event(non_existent_id, events[0].clone());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            format!("Event with ID {} not found", non_existent_id)
        );
    }

    #[test]
    fn test_list_events_empty() {
        let repo = InMemoryEventRepository::new();
        
        let events = repo.list_events().unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn test_list_events_multiple() {
        let (repo, events) = create_test_repository_with_events(5);
        
        let listed_events = repo.list_events().unwrap();
        assert_eq!(listed_events.len(), 5);
        
        // Verify all events are in the list
        for event in &events {
            assert!(listed_events.iter().any(|e| e.id == event.id));
        }
    }

    #[test]
    fn test_get_by_id_found() {
        let (repo, events) = create_test_repository_with_events(3);
        let event_id = events[1].id;
        
        let result = repo.get_by_id(event_id).unwrap();
        assert!(result.is_some());
        
        let event = result.unwrap();
        assert_eq!(event.id, event_id);
        assert_eq!(event.title, events[1].title);
    }

    #[test]
    fn test_get_by_id_not_found() {
        let (repo, _) = create_test_repository_with_events(2);
        let non_existent_id = Uuid::new_v4();
        
        let result = repo.get_by_id(non_existent_id).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_concurrent_operations() {
        use std::thread;
        
        let repo = Arc::new(InMemoryEventRepository::new());
        let mut handles = vec![];
        
        // Create 10 threads that each add an event
        for i in 0..10 {
            let repo_clone = Arc::clone(&repo);
            let handle = thread::spawn(move || {
                let event = create_test_event(&format!("Concurrent Event {}", i));
                repo_clone.add(event).unwrap()
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify all events were added
        let events = repo.list_events().unwrap();
        assert_eq!(events.len(), 10);
    }
}