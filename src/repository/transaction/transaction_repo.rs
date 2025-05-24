use async_trait::async_trait;
use chrono::Utc;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::error::Error;
use std::sync::RwLock;
use uuid::Uuid;

use crate::model::transaction::{Transaction, TransactionStatus};

#[async_trait]
pub trait TransactionPersistenceStrategy {
    async fn save(
        &self,
        transaction: &Transaction,
    ) -> Result<Transaction, Box<dyn Error + Send + Sync>>;
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<Transaction>, Box<dyn Error + Send + Sync>>;
    async fn find_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>>;
    async fn update_status(
        &self,
        id: Uuid,
        status: TransactionStatus,
    ) -> Result<Transaction, Box<dyn Error + Send + Sync>>;
    async fn delete(&self, id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>>;
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

#[async_trait]
impl TransactionPersistenceStrategy for InMemoryTransactionPersistence {
    async fn save(
        &self,
        transaction: &Transaction,
    ) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        let mut transactions = self.transactions.write().unwrap();
        let transaction_clone = transaction.clone();
        transactions.insert(transaction.id, transaction_clone.clone());
        Ok(transaction_clone)
    }
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions = self.transactions.read().unwrap();
        Ok(transactions.get(&id).cloned())
    }

    async fn find_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions = self.transactions.read().unwrap();
        let user_transactions = transactions
            .values()
            .filter(|t| t.user_id == user_id)
            .cloned()
            .collect();
        Ok(user_transactions)
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: TransactionStatus,
    ) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        let mut transactions = self.transactions.write().unwrap();

        if let Some(transaction) = transactions.get_mut(&id) {
            transaction.status = status;
            transaction.updated_at = Utc::now();
            Ok(transaction.clone())
        } else {
            Err("Transaction not found".into())
        }
    }

    async fn delete(&self, id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut transactions = self.transactions.write().unwrap();

        if transactions.remove(&id).is_some() {
            Ok(())
        } else {
            Err("Transaction not found".into())
        }
    }
}

#[async_trait]
pub trait TransactionRepository {
    async fn save(
        &self,
        transaction: &Transaction,
    ) -> Result<Transaction, Box<dyn Error + Send + Sync>>;
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<Transaction>, Box<dyn Error + Send + Sync>>;
    async fn find_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>>;
    async fn update_status(
        &self,
        id: Uuid,
        status: TransactionStatus,
    ) -> Result<Transaction, Box<dyn Error + Send + Sync>>;
    async fn delete(&self, id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>>;
}

pub struct DbTransactionRepository<S: TransactionPersistenceStrategy> {
    strategy: S,
}

impl<S: TransactionPersistenceStrategy> DbTransactionRepository<S> {
    pub fn new(strategy: S) -> Self {
        DbTransactionRepository { strategy }
    }
}

#[async_trait]
impl<S: TransactionPersistenceStrategy + Send + Sync> TransactionRepository
    for DbTransactionRepository<S>
{
    async fn save(
        &self,
        transaction: &Transaction,
    ) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        self.strategy.save(transaction).await
    }

    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<Transaction>, Box<dyn Error + Send + Sync>> {
        self.strategy.find_by_id(id).await
    }

    async fn find_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        self.strategy.find_by_user(user_id).await
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: TransactionStatus,
    ) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        self.strategy.update_status(id, status).await
    }

    async fn delete(&self, id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.strategy.delete(id).await
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
impl TransactionPersistenceStrategy for PostgresTransactionPersistence {
    async fn save(
        &self,
        transaction: &Transaction,
    ) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        let query = "INSERT INTO transactions (id, user_id, ticket_id, amount, description, payment_method, external_reference, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8::transaction_status, $9, $10) RETURNING *";
        let row = sqlx::query(query)
            .bind(transaction.id)
            .bind(transaction.user_id)
            .bind(transaction.ticket_id)
            .bind(transaction.amount)            .bind(&transaction.description)
            .bind(&transaction.payment_method)
            .bind(&transaction.external_reference)
            .bind(transaction.status.to_string().to_lowercase())
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

    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<Transaction>, Box<dyn Error + Send + Sync>> {
        let query = "SELECT * FROM transactions WHERE id = $1";
        let row = sqlx::query(query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
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
    async fn find_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let query = "SELECT * FROM transactions WHERE user_id = $1";
        let rows = sqlx::query(query)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;

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
    }    async fn update_status(
        &self,
        id: Uuid,
        status: TransactionStatus,
    ) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        let query = "UPDATE transactions SET status = $1::transaction_status WHERE id = $2 RETURNING *";

        let row = sqlx::query(query)
            .bind(status.to_string().to_lowercase())
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
            }
            None => Err("Transaction not found".into()),
        }
    }
    async fn delete(&self, id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>> {
        let query = "DELETE FROM transactions WHERE id = $1";

        let result = sqlx::query(query).bind(id).execute(&self.pool).await?;

        if result.rows_affected() > 0 {
            Ok(())
        } else {
            Err("Transaction not found".into())
        }
    }
}
