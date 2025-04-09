use async_trait::async_trait;
use crate::models::event::Event;

#[async_trait]
pub trait EventRepository {
    async fn create_event(&self, event: &Event) -> Result<(), String>;
    async fn list_events(&self) -> Result<Vec<Event>, String>;
    async fn update_event(&self, event_id: &str, updated_event: &Event) -> Result<(), String>;
    async fn delete_event(&self, event_id: &str) -> Result<(), String>;

}

