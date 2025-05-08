use async_trait::async_trait;
use chrono::Utc;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::error::Error;
use std::sync::RwLock;
use uuid::Uuid;

use crate::model::transaction::{Transaction, TransactionStatus};

pub trait TransactionPersistenceStrategy {
    fn save(&self, transaction: &Transaction) -> Result<Transaction, Box<dyn Error>>;
    fn find_by_id(&self, id: Uuid) -> Result<Option<Transaction>, Box<dyn Error>>;
    fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Transaction>, Box<dyn Error>>;
    fn update_status(
        &self,
        id: Uuid,
        status: TransactionStatus,
    ) -> Result<Transaction, Box<dyn Error>>;
    fn delete(&self, id: Uuid) -> Result<(), Box<dyn Error>>;
}

pub struct InMemoryTransactionPersistence {
    transactions: RwLock<HashMap<Uuid, Transaction>>,
}

impl InMemoryTransactionPersistence {
    pub fn new() -> Self {
        Self {
            transactions: RwLock::new(HashMap::new()),
        }
    }
}

impl TransactionPersistenceStrategy for InMemoryTransactionPersistence {
    fn save(&self, transaction: &Transaction) -> Result<Transaction, Box<dyn Error>> {
        let mut transactions = self.transactions.write().unwrap();
        let transaction_clone = transaction.clone();
        transactions.insert(transaction.id, transaction_clone.clone());
        Ok(transaction_clone)
    }

    fn find_by_id(&self, id: Uuid) -> Result<Option<Transaction>, Box<dyn Error>> {
        let transactions = self.transactions.read().unwrap();
        Ok(transactions.get(&id).cloned())
    }

    fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Transaction>, Box<dyn Error>> {
        let transactions = self.transactions.read().unwrap();
        let user_transactions = transactions
            .values()
            .filter(|t| t.user_id == user_id)
            .cloned()
            .collect();
        Ok(user_transactions)
    }

    fn update_status(
        &self,
        id: Uuid,
        status: TransactionStatus,
    ) -> Result<Transaction, Box<dyn Error>> {
        let mut transactions = self.transactions.write().unwrap();

        if let Some(transaction) = transactions.get_mut(&id) {
            transaction.status = status;
            transaction.updated_at = Utc::now();
            Ok(transaction.clone())
        } else {
            Err("Transaction not found".into())
        }
    }

    fn delete(&self, id: Uuid) -> Result<(), Box<dyn Error>> {
        let mut transactions = self.transactions.write().unwrap();

        if transactions.remove(&id).is_some() {
            Ok(())
        } else {
            Err("Transaction not found".into())
        }
    }
}

pub trait TransactionRepository {
    fn save(&self, transaction: &Transaction) -> Result<Transaction, Box<dyn Error>>;
    fn find_by_id(&self, id: Uuid) -> Result<Option<Transaction>, Box<dyn Error>>;
    fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Transaction>, Box<dyn Error>>;
    fn update_status(
        &self,
        id: Uuid,
        status: TransactionStatus,
    ) -> Result<Transaction, Box<dyn Error>>;
    fn delete(&self, id: Uuid) -> Result<(), Box<dyn Error>>;
}

pub struct DbTransactionRepository<S: TransactionPersistenceStrategy> {
    strategy: S,
}

impl<S: TransactionPersistenceStrategy> DbTransactionRepository<S> {
    pub fn new(strategy: S) -> Self {
        DbTransactionRepository { strategy }
    }
}

impl<S: TransactionPersistenceStrategy> TransactionRepository for DbTransactionRepository<S> {
    fn save(&self, transaction: &Transaction) -> Result<Transaction, Box<dyn Error>> {
        self.strategy.save(transaction)
    }

    fn find_by_id(&self, id: Uuid) -> Result<Option<Transaction>, Box<dyn Error>> {
        self.strategy.find_by_id(id)
    }

    fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Transaction>, Box<dyn Error>> {
        self.strategy.find_by_user(user_id)
    }

    fn update_status(
        &self,
        id: Uuid,
        status: TransactionStatus,
    ) -> Result<Transaction, Box<dyn Error>> {
        self.strategy.update_status(id, status)
    }

    fn delete(&self, id: Uuid) -> Result<(), Box<dyn Error>> {
        self.strategy.delete(id)
    }
}

pub struct PostgresTransactionPersistence {
    pool: PgPool,
}

impl PostgresTransactionPersistence {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
pub trait AsyncTransactionPersistenceStrategy {
    async fn save(&self, transaction: &Transaction) -> Result<Transaction, Box<dyn Error>>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Transaction>, Box<dyn Error>>;
    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Transaction>, Box<dyn Error>>;
    async fn update_status(
        &self,
        id: Uuid,
        status: TransactionStatus,
    ) -> Result<Transaction, Box<dyn Error>>;
    async fn delete(&self, id: Uuid) -> Result<(), Box<dyn Error>>;
}

#[async_trait]
impl AsyncTransactionPersistenceStrategy for PostgresTransactionPersistence {
    async fn save(&self, transaction: &Transaction) -> Result<Transaction, Box<dyn Error>> {
        let query = "INSERT INTO transactions (id, user_id, ticket_id, amount, description, payment_method, external_reference, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *";
        let row = sqlx::query(query)
            .bind(transaction.id)
            .bind(transaction.user_id)
            .bind(transaction.ticket_id)
            .bind(transaction.amount)
            .bind(&transaction.description)
            .bind(&transaction.payment_method)
            .bind(&transaction.external_reference)
            .bind(transaction.status.to_string())
            .bind(transaction.created_at)
            .bind(transaction.updated_at)
            .fetch_one(&self.pool)
            .await?;
            
        let saved_transaction = Transaction {
            id: row.get("id"),
            user_id: row.get("user_id"),
            ticket_id: row.get("ticket_id"),
            amount: row.get("amount"),
            description: row.get("description"),
            payment_method: row.get("payment_method"),
            external_reference: row.get("external_reference"),
            status: TransactionStatus::from_string(row.get("status")),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        Ok(saved_transaction)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Transaction>, Box<dyn Error>> {
        let query = "SELECT * FROM transactions WHERE id = $1";
        let row = sqlx::query(query).bind(id).fetch_optional(&self.pool).await?;
        if let Some(row) = row {
            let transaction = Transaction {
                id: row.get("id"),
                user_id: row.get("user_id"),
                ticket_id: row.get("ticket_id"),
                amount: row.get("amount"),
                description: row.get("description"),
                payment_method: row.get("payment_method"),
                external_reference: row.get("external_reference"),
                status: TransactionStatus::from_string(row.get("status")),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(transaction))
        } else {
            Ok(None)
        }
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Transaction>, Box<dyn Error>> {
        let query = "SELECT * FROM transactions WHERE user_id = $1";
        let rows = sqlx::query(query).bind(user_id).fetch_all(&self.pool).await?;
        
        let transactions = rows
            .iter()
            .map(|row| Transaction {
                id: row.get("id"),
                user_id: row.get("user_id"),
                ticket_id: row.get("ticket_id"),
                amount: row.get("amount"),
                description: row.get("description"),
                payment_method: row.get("payment_method"),
                external_reference: row.get("external_reference"),
                status: TransactionStatus::from_string(row.get("status")),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();
            
        Ok(transactions)
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: TransactionStatus,
    ) -> Result<Transaction, Box<dyn Error>> {
        let query = "UPDATE transactions SET status = $1 WHERE id = $2 RETURNING *";
        
        let row = sqlx::query(query)
            .bind(status.to_string())
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
            
        match row {
            Some(row) => {
                let transaction = Transaction {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    ticket_id: row.get("ticket_id"),
                    amount: row.get("amount"),
                    description: row.get("description"),
                    payment_method: row.get("payment_method"),
                    external_reference: row.get("external_reference"),
                    status: TransactionStatus::from_string(row.get("status")),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };
                Ok(transaction)
            },
            None => Err("Transaction not found".into()),
        }
    }

    async fn delete(&self, id: Uuid) -> Result<(), Box<dyn Error>> {
        let query = "DELETE FROM transactions WHERE id = $1";
        
        let result = sqlx::query(query)
            .bind(id)
            .execute(&self.pool)
            .await?;
            
        if result.rows_affected() > 0 {
            Ok(())
        } else {
            Err("Transaction not found".into())
        }
    }
}