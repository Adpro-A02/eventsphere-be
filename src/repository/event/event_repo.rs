use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::model::event::Event;

pub trait EventRepository: Send + Sync + 'static {
    fn add(&self, event: Event) -> Result<Event, String>;
    fn delete(&self, event_id: Uuid) -> Result<(), String>;
    fn update_event(&self, event_id: Uuid, updated_event: Event) -> Result<Event, String>;
    fn list_events(&self) -> Result<Vec<Event>, String>;
    fn get_by_id(&self, event_id: Uuid) -> Result<Option<Event>, String>;
}

// In-memory implementation of EventRepository
pub struct InMemoryEventRepository {
    events: Mutex<HashMap<Uuid, Event>>,
}

impl InMemoryEventRepository {
    pub fn new() -> Self {
        InMemoryEventRepository {
            events: Mutex::new(HashMap::new()),
        }
    }
}

impl EventRepository for InMemoryEventRepository {
    fn add(&self, event: Event) -> Result<Event, String> {
        let mut events = self.events.lock().map_err(|e| e.to_string())?;
        let event_clone = event.clone();
        events.insert(event.id, event);
        Ok(event_clone)
    }

    fn delete(&self, event_id: Uuid) -> Result<(), String> {
        let mut events = self.events.lock().map_err(|e| e.to_string())?;
        
        if events.remove(&event_id).is_none() {
            return Err(format!("Event with ID {} not found", event_id));
        }
        
        Ok(())
    }

    fn update_event(&self, event_id: Uuid, updated_event: Event) -> Result<Event, String> {
        let mut events = self.events.lock().map_err(|e| e.to_string())?;
        
        if !events.contains_key(&event_id) {
            return Err(format!("Event with ID {} not found", event_id));
        }
        
        let event_clone = updated_event.clone();
        events.insert(event_id, updated_event);
        Ok(event_clone)
    }

    fn list_events(&self) -> Result<Vec<Event>, String> {
        let events = self.events.lock().map_err(|e| e.to_string())?;
        Ok(events.values().cloned().collect())
    }

    fn get_by_id(&self, event_id: Uuid) -> Result<Option<Event>, String> {
        let events = self.events.lock().map_err(|e| e.to_string())?;
        Ok(events.get(&event_id).cloned())
    }
}