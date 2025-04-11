use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::model::event::Event;
use crate::repository::event_repository::EventRepository;

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

pub struct EventService<R: EventRepository> {
    repository: Arc<R>,
    runtime: Runtime,
}

impl<R: EventRepository> EventService<R> {
    pub fn new(repository: Arc<R>) -> Self {
    }

    pub fn create_event(&self, event: Event) -> Result<Event, ServiceError> {
    }

    pub fn list_events(&self) -> Result<Vec<Event>, ServiceError> {
    }

    pub fn get_event(&self, event_id: &str) -> Result<Event, ServiceError> {
    }

    pub fn update_event(&self, event_id: &str, event: Event) -> Result<Event, ServiceError> {
    }

    pub fn delete_event(&self, event_id: &str) -> Result<(), ServiceError> {
        
    }
}
