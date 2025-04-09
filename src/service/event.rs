use rocket::http::Status;
use rocket::serde::json::Json;

use async_trait::async_trait;
use crate::models::event::Event;
use crate::repositories::event_repository::EventRepository;
use crate::errors::{ServiceError, RepositoryError};
use std::sync::Arc;

pub struct EventServiceImpl<R: EventRepository + Send + Sync> {
    repository: Arc<R>,
}

impl<R: EventRepository + Send + Sync> EventServiceImpl<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: EventRepository + Send + Sync> EventService for EventServiceImpl<R> {
    async fn create_event(&self, event: Event) -> Result<Event, ServiceError> {
        self.repository.create_event(&event).await.map_err(ServiceError::from)
    }

    async fn get_event(&self, event_id: &str) -> Result<Option<Event>, ServiceError> {
        self.repository.get_event(event_id).await.map_err(ServiceError::from)
    }

    async fn update_event(&self, event_id: &str, event: Event) -> Result<Event, ServiceError> {
        self.repository.update_event(event_id, &event).await.map_err(ServiceError::from)
    }

    async fn delete_event(&self, event_id: &str) -> Result<(), ServiceError> {
        self.repository.delete_event(event_id).await.map_err(ServiceError::from)
    }
}
