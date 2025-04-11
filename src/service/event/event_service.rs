use std::sync::{Arc, Mutex};
use chrono::NaiveDateTime;
use tokio::runtime::{self, Runtime};
use uuid::Uuid;

use crate::model::event::{event, Event, EventStatus};
use crate::repository::event::EventRepository;


pub trait EventService {
    fn create_event(&self, event: Event) -> Result<Event, ServiceError>;
    fn get_event(&self, event_id: &str) -> Result<Event, ServiceError>;
    fn update_event(&self, event_id: &str, event: Event) -> Result<Event, ServiceError>;
    fn delete_event(&self, event_id: &str) -> Result<(), ServiceError>;
}


#[derive(Debug)]
pub enum ServiceError {
    RepositoryError(String),
    NotFound,
    InvalidInput(String),
}

impl From<String> for ServiceError {
    fn from(err: String) -> Self {
        ServiceError::RepositoryError(err)
    }
}

pub struct DefaultEventService{
    event_repository: Arc<dyn EventRepository>,
    runtime: Runtime,
    
   
}


impl DefaultEventService{
  

    pub fn create_event(
        &self,
        title: String,
        description: String,
        event_date: NaiveDateTime,
        location: String,
        base_price: f64,
    ) -> Result<Event, ServiceError> {
        let event = Event {
            id: Uuid::new_v4(),
            title,
            description,
            event_date,
            location,
            base_price,
            status: EventStatus::Draft,
        };
    
       
        self.runtime
            .block_on(self.event_repository.add(event.clone()))
            .map_err(ServiceError::RepositoryError)?;
    
        Ok(event)
    }
    

    
    

    pub fn get_event(&self, event_id: &str) -> Result<Event, ServiceError> {
        let parsed_id = Uuid::parse_str(event_id)
            .map_err(|_| ServiceError::InvalidInput("Invalid UUID format".to_string()))?;
    
        let events = self.runtime
            .block_on(self.event_repository.list_events())
            .map_err(ServiceError::RepositoryError)?;
    
        events.into_iter()
            .find(|e| e.id == parsed_id)
            .ok_or(ServiceError::NotFound)
    }
    

    pub fn update_event(
        &mut self,
        event_id: &str,
        updated_event: Event,
    ) -> Result<Event, ServiceError> {
        let parsed_id = Uuid::parse_str(event_id)
            .map_err(|_| ServiceError::InvalidInput("Invalid UUID format".to_string()))?;
    
        if updated_event.id != parsed_id {
            return Err(ServiceError::InvalidInput(
                "ID in path doesn't match event body".to_string(),
            ));
        }
    
        let updated = self
            .runtime
            .block_on(self.event_repository.update_event(parsed_id, updated_event))
            .map_err(ServiceError::RepositoryError)?;
    
        Ok(updated)
    }
    

    pub fn delete_event(&self, event_id: &str) -> Result<(), ServiceError> {
        let parsed_id = Uuid::parse_str(event_id)
            .map_err(|_| ServiceError::InvalidInput("Invalid UUID format".to_string()))?;
    
        self.event_repository.delete(parsed_id).map_err(ServiceError::RepositoryError)
    }
}
