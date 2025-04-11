use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::model::event::Event;
use crate::repository::event::EventRepository;


pub trait EventService {
    fn create_event(&self, event: Event) -> Result<Event, ServiceError>;
    fn list_events(&self) -> Result<Vec<Event>, ServiceError>;
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

pub struct DefaultEventService<R: EventRepository> {
    repository: Arc<R>,
    runtime: Runtime,
    events: Vec<Event>, 
}


impl<R: EventRepository> DefaultEventService<R>{
    pub fn new() -> Self {
        todo!()
    }

    pub fn create_event(&self, event: Event) -> Result<Event, ServiceError> {
        todo!()
    }

    pub fn list_events(&self) -> Result<Vec<Event>, ServiceError> {
        todo!()
    }

    pub fn get_event(&self, event_id: &str) -> Result<Event, ServiceError> {
        todo!()
    }

    pub fn update_event(&self, event_id: &str, event: Event) -> Result<Event, ServiceError> {
        todo!()
    }

    pub fn delete_event(&self, event_id: &str) -> Result<(), ServiceError> {
        todo!()
    }
}
