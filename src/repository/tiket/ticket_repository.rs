use crate::model::tiket::ticket::Ticket;
use uuid::Uuid;

/// Repository trait for handling Ticket data persistence
pub trait TicketRepository {
    /// Saves a ticket to the repository
    /// Generates a UUID if one is not present
    fn save(&self, ticket: Ticket) -> Result<Ticket, String>;
    
    /// Finds a ticket by its ID
    fn find_by_id(&self, id: &Uuid) -> Result<Option<Ticket>, String>;
    
    /// Finds all tickets for a specific event
    fn find_by_event_id(&self, event_id: &Uuid) -> Result<Vec<Ticket>, String>;
    
    /// Updates an existing ticket
    fn update(&self, ticket: Ticket) -> Result<Ticket, String>;
    
    /// Deletes a ticket by ID
    fn delete(&self, id: &Uuid) -> Result<(), String>;
    
    /// Updates the quota for a specific ticket
    fn update_quota(&self, id: &Uuid, new_quota: u32) -> Result<Ticket, String>;
}

// Here you could add concrete implementations of the repository
// For example, a PostgreSQL implementation could be:

/*
pub struct PostgresTicketRepository {
    pool: PgPool,
}

impl PostgresTicketRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl TicketRepository for PostgresTicketRepository {
    // Implement all the required methods using PostgreSQL
}
*/

// Or a MongoDB implementation:

/*
pub struct MongoTicketRepository {
    collection: Collection<Document>,
}

impl TicketRepository for MongoTicketRepository {
    // Implement all the required methods using MongoDB
}
*/
