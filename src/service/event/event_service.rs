use std::sync::Arc;
use uuid::Uuid;

use crate::model::event::{Event};
use crate::model::event::event::{CreateEventDto, UpdateEventDto};
use crate::repository::event::event_repo::EventRepository;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository error: {0}")]
    RepositoryError(String),
    
    #[error("Event not found: {0}")]
    NotFound(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

pub struct EventService<R: EventRepository> {
    repository: Arc<R>,
}

impl<R: EventRepository> EventService<R> {
    pub fn new(repository: Arc<R>) -> Self {
        EventService { repository }
    }

    pub fn create_event(&self, dto: CreateEventDto) -> Result<Event, ServiceError> {
        // Validate input
        if dto.title.is_empty() {
            return Err(ServiceError::InvalidInput("Title cannot be empty".to_string()));
        }
        
        if dto.base_price < 0.0 {
            return Err(ServiceError::InvalidInput("Price cannot be negative".to_string()));
        }
        
        let now = chrono::Local::now().naive_local();
        if dto.event_date <= now {
            return Err(ServiceError::InvalidInput("Event date must be in the future".to_string()));
        }
        
        // Create new event
        let event = Event::new(
            dto.title,
            dto.description,
            dto.event_date,
            dto.location,
            dto.base_price,
        );
        
        // Save to repository
        self.repository.add(event)
            .map_err(|e| ServiceError::RepositoryError(e))
    }

    pub fn list_events(&self) -> Result<Vec<Event>, ServiceError> {
        self.repository.list_events()
            .map_err(|e| ServiceError::RepositoryError(e))
    }

    pub fn get_event(&self, event_id: &str) -> Result<Event, ServiceError> {
        let uuid = Uuid::parse_str(event_id)
            .map_err(|_| ServiceError::InvalidInput(format!("Invalid UUID: {}", event_id)))?;
        
        let event = self.repository.get_by_id(uuid)
            .map_err(|e| ServiceError::RepositoryError(e))?
            .ok_or_else(|| ServiceError::NotFound(format!("Event with ID {} not found", event_id)))?;
        
        Ok(event)
    }

    pub fn update_event(&self, event_id: &str, dto: UpdateEventDto) -> Result<Event, ServiceError> {
        let uuid = Uuid::parse_str(event_id)
            .map_err(|_| ServiceError::InvalidInput(format!("Invalid UUID: {}", event_id)))?;
        
        // Get existing event
        let mut event = self.repository.get_by_id(uuid)
            .map_err(|e| ServiceError::RepositoryError(e))?
            .ok_or_else(|| ServiceError::NotFound(format!("Event with ID {} not found", event_id)))?;
        
        // Validate event date if provided
        if let Some(event_date) = dto.event_date {
            let now = chrono::Local::now().naive_local();
            if event_date <= now {
                return Err(ServiceError::InvalidInput("Event date must be in the future".to_string()));
            }
        }
        
        // Validate price if provided
        if let Some(price) = dto.base_price {
            if price < 0.0 {
                return Err(ServiceError::InvalidInput("Price cannot be negative".to_string()));
            }
        }
        
        // Update event
        event.update(
            dto.title,
            dto.description,
            dto.event_date,
            dto.location,
            dto.base_price,
        );
        
        // Save updated event
        self.repository.update_event(uuid, event)
            .map_err(|e| ServiceError::RepositoryError(e))
    }

    pub fn delete_event(&self, event_id: &str) -> Result<(), ServiceError> {
        let uuid = Uuid::parse_str(event_id)
            .map_err(|_| ServiceError::InvalidInput(format!("Invalid UUID: {}", event_id)))?;
        
        // Check if event exists
        let exists = self.repository.get_by_id(uuid)
            .map_err(|e| ServiceError::RepositoryError(e))?
            .is_some();
        
        if !exists {
            return Err(ServiceError::NotFound(format!("Event with ID {} not found", event_id)));
        }
        
        // Delete event
        self.repository.delete(uuid)
            .map_err(|e| ServiceError::RepositoryError(e))
    }
    
    pub fn publish_event(&self, event_id: &str) -> Result<Event, ServiceError> {
        let uuid = Uuid::parse_str(event_id)
            .map_err(|_| ServiceError::InvalidInput(format!("Invalid UUID: {}", event_id)))?;
        
        // Get existing event
        let mut event = self.repository.get_by_id(uuid)
            .map_err(|e| ServiceError::RepositoryError(e))?
            .ok_or_else(|| ServiceError::NotFound(format!("Event with ID {} not found", event_id)))?;
        
        // Publish event
        event.publish()
            .map_err(|e| ServiceError::InvalidInput(e.to_string()))?;
        
        // Save updated event
        self.repository.update_event(uuid, event)
            .map_err(|e| ServiceError::RepositoryError(e))
    }
    
    pub fn cancel_event(&self, event_id: &str) -> Result<Event, ServiceError> {
        let uuid = Uuid::parse_str(event_id)
            .map_err(|_| ServiceError::InvalidInput(format!("Invalid UUID: {}", event_id)))?;
        
        // Get existing event
        let mut event = self.repository.get_by_id(uuid)
            .map_err(|e| ServiceError::RepositoryError(e))?
            .ok_or_else(|| ServiceError::NotFound(format!("Event with ID {} not found", event_id)))?;
        
        // Cancel event
        event.cancel()
            .map_err(|e| ServiceError::InvalidInput(e.to_string()))?;
        
        // Save updated event
        self.repository.update_event(uuid, event)
            .map_err(|e| ServiceError::RepositoryError(e))
    }
    
    pub fn complete_event(&self, event_id: &str) -> Result<Event, ServiceError> {
        let uuid = Uuid::parse_str(event_id)
            .map_err(|_| ServiceError::InvalidInput(format!("Invalid UUID: {}", event_id)))?;
        
        // Get existing event
        let mut event = self.repository.get_by_id(uuid)
            .map_err(|e| ServiceError::RepositoryError(e))?
            .ok_or_else(|| ServiceError::NotFound(format!("Event with ID {} not found", event_id)))?;
        
        // Complete event
        event.complete()
            .map_err(|e| ServiceError::InvalidInput(e.to_string()))?;
        
        // Save updated event
        self.repository.update_event(uuid, event)
            .map_err(|e| ServiceError::RepositoryError(e))
    }
}