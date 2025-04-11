use std::sync::Arc;
use crate::repository::tiket::TicketRepository;
use crate::config::DatabaseConfig;

pub enum DatabaseType {
    InMemory,
    Postgres,
    MongoDB,
}

pub struct RepositoryFactory;

impl RepositoryFactory {
    pub fn create_ticket_repository(db_type: DatabaseType, config: &DatabaseConfig) -> Arc<dyn TicketRepository + Send + Sync> {
        match db_type {
            DatabaseType::InMemory => {
                Arc::new(InMemoryTicketRepository::new())
            },
            DatabaseType::Postgres => {
                let pool = config.create_postgres_pool()
                    .expect("Failed to create Postgres connection pool");
                Arc::new(PostgresTicketRepository::new(pool))
            },
            DatabaseType::MongoDB => {
                let client = config.create_mongodb_client()
                    .expect("Failed to create MongoDB client");
                let db = client.database(&config.database_name);
                Arc::new(MongoTicketRepository::new(db.collection("tickets")))
            }
        }
    }
}

// You would need to implement these repository types
struct InMemoryTicketRepository {
    // Implementation from your tests
}

struct PostgresTicketRepository {
    // Postgres implementation
}

struct MongoTicketRepository {
    // MongoDB implementation
}

// Implementations for each repository type...
