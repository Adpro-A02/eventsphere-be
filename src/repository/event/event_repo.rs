
use crate::model::event::Event;
 // Import the EventRepository trait

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;




// Ensure the EventRepository trait is defined if not already present
pub trait EventRepository {
    fn add(&self, event: Event) -> Result<Event, String>;
    fn delete(&self, event_id: Uuid) -> Result<(), String>;
    fn update_event(&self, event_id: Uuid, updated_event: Event) -> Result<Event, String>;
    fn list_events(&self) -> Result<Vec<Event>, String>;
    fn get_by_id(&self, event_id: Uuid) -> Result<Option<Event>, String>;
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
    fn add(&self, event: Event) -> Result<Event, String> {
        let mut events = self.events.lock().map_err(|_| "Lock poisoned".to_string())?;
        let id_str = event.id.to_string();
        if events.contains_key(&id_str) {
            return Err("Event already exists".to_string());
        }
        events.insert(id_str, event.clone());
        Ok(event)
    }
    fn delete(&self, event_id: Uuid) -> Result<(), String> {
        let mut events = self.events.lock().map_err(|_| "Lock poisoned".to_string())?;
        let id_str = event_id.to_string();
        if events.remove(&id_str).is_some() {
            Ok(())
        } else {
            Err("Event not found".to_string())
        }
    }
    fn update_event(&self, event_id: Uuid, updated_event: Event) -> Result<Event, String> {
        let mut events = self.events.lock().map_err(|_| "Lock poisoned".to_string())?;
        let id_str = event_id.to_string();
        if events.contains_key(&id_str) {
            events.insert(id_str, updated_event.clone());
            Ok(updated_event)
        } else {
            Err("Event not found".to_string())
        }
    }
    fn list_events(&self) -> Result<Vec<Event>, String> {
        let events = self.events.lock().map_err(|_| "Lock poisoned".to_string())?;
        Ok(events.values().cloned().collect())
    }

    fn get_by_id(&self, event_id: Uuid) -> Result<Option<Event>, String> {
        let events = self.events.lock().map_err(|_| "Lock poisoned".to_string())?;
        Ok(events.get(&event_id.to_string()).cloned())
    }
   
}



