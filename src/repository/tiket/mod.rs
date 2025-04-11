use crate::model::tiket::ticket::Ticket;
use uuid::Uuid;

/// Defines the interface for Ticket repository operations
pub trait TicketRepository {
    fn save(&self, ticket: Ticket) -> Result<Ticket, String>;
    fn find_by_id(&self, id: &Uuid) -> Result<Option<Ticket>, String>;
    fn find_by_event_id(&self, event_id: &Uuid) -> Result<Vec<Ticket>, String>;
    fn update(&self, ticket: Ticket) -> Result<Ticket, String>;
    fn delete(&self, id: &Uuid) -> Result<(), String>;
    fn update_quota(&self, id: &Uuid, new_quota: u32) -> Result<Ticket, String>;
}

pub mod ticket_repository;
pub mod tests;

pub use ticket_repository::TicketRepository;

#[cfg(test)]
pub mod tests;
