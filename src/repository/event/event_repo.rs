
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
    fn add(&self, event: Event) -> Result<Event, String>{
        todo!();
    }
    fn delete(&self, event_id: Uuid) -> Result<(), String>{
        todo!();
    }
    fn update_event(&self, event_id: Uuid, updated_event: Event) -> Result<Event, String>{
        todo!();
    }
    fn list_events(&self) -> Result<Vec<Event>, String>{
        todo!();
    }
    fn get_by_id(&self, event_id: Uuid) -> Result<Option<Event>, String>{
        todo!();
    }
   
}



