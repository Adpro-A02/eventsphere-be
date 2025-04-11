use async_trait::async_trait;
use crate::model::event::Event;
 // Import the EventRepository trait

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;




// Ensure the EventRepository trait is defined if not already present
pub trait EventRepository {
    fn create_event(&self, event: &Event) -> Result<(), String>;
    fn list_events(&self) -> Result<Vec<Event>, String>;
    fn update_event(&self, event_id: &str, updated_event: &Event) -> Result<(), String>;
    fn delete_event(&self, event_id: &str) -> Result<(), String>;
}

pub struct InMemoryEventRepo {
    pub events: Arc<Mutex<HashMap<String, Event>>>,
}

impl InMemoryEventRepo {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl EventRepository for InMemoryEventRepo {
    fn create_event(&self, event: &Event) -> Result<(), String> {
        let mut events = self.events.lock().unwrap();
        let id = event.id.to_string();

        if events.contains_key(&id) {
            return Err("Event already exists".into());
        }

        events.insert(id, event.clone());
        Ok(())
    }

    fn list_events(&self) -> Result<Vec<Event>, String> {
        let events = self.events.lock().unwrap();
        Ok(events.values().cloned().collect())
    }

    fn update_event(&self, event_id: &str, updated_event: &Event) -> Result<(), String> {
        let mut events = self.events.lock().unwrap();

        if !events.contains_key(event_id) {
            return Err("Event not found".into());
        }

        events.insert(event_id.to_string(), updated_event.clone());
        Ok(())
    }

    fn delete_event(&self, event_id: &str) -> Result<(), String> {
        let mut events = self.events.lock().unwrap();

        if events.remove(event_id).is_some() {
            Ok(())
        } else {
            Err("Event not found".into())
        }
    }
}



