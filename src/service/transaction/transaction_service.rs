use async_trait::async_trait;
use chrono::Utc;
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;

use crate::model::transaction::{Transaction, TransactionStatus};
use crate::repository::transaction::transaction_repo::TransactionRepository;
use crate::service::transaction::balance_service::BalanceService;
use crate::service::transaction::payment_service::PaymentService;

#[async_trait]
pub trait TransactionService {
    async fn create_transaction(
        &self,
        user_id: Uuid,
        ticket_id: Option<Uuid>,
        amount: i64,
        description: String,
        payment_method: String,
    ) -> Result<Transaction, Box<dyn Error + Send + Sync + 'static>>;

    async fn process_payment(
        &self,
        transaction_id: Uuid,
        external_reference: Option<String>,
    ) -> Result<Transaction, Box<dyn Error + Send + Sync + 'static>>;

    async fn validate_payment(
        &self,
        transaction_id: Uuid,
    ) -> Result<bool, Box<dyn Error + Send + Sync + 'static>>;
    async fn refund_transaction(
        &self,
        transaction_id: Uuid,
    ) -> Result<Transaction, Box<dyn Error + Send + Sync + 'static>>;
    async fn get_transaction(
        &self,
        transaction_id: Uuid,
    ) -> Result<Option<Transaction>, Box<dyn Error + Send + Sync + 'static>>;
    async fn get_user_transactions(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync + 'static>>;

    async fn add_funds_to_balance(
        &self,
        user_id: Uuid,
        amount: i64,
        payment_method: String,
    ) -> Result<(Transaction, i64), Box<dyn Error + Send + Sync + 'static>>;    async fn withdraw_funds(
        &self,
        user_id: Uuid,
        amount: i64,
        description: String,
    ) -> Result<(Transaction, i64), Box<dyn Error + Send + Sync + 'static>>;

    async fn get_user_balance(
        &self,
        user_id: Uuid,
    ) -> Result<Option<crate::model::transaction::Balance>, Box<dyn Error + Send + Sync + 'static>>;

    async fn delete_transaction(
        &self,
        transaction_id: Uuid,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>>;
}

pub struct DefaultTransactionService {
    transaction_repository: Arc<dyn TransactionRepository + Send + Sync>,
    balance_service: Arc<dyn BalanceService + Send + Sync>,
    payment_service: Arc<dyn PaymentService + Send + Sync>,
}

impl DefaultTransactionService {
    pub fn new(
        transaction_repository: Arc<dyn TransactionRepository + Send + Sync>,
        balance_service: Arc<dyn BalanceService + Send + Sync>,
        payment_service: Arc<dyn PaymentService + Send + Sync>,
    ) -> Self {
        Self {
            transaction_repository,
            balance_service,
            payment_service,
        }
    }
}

#[async_trait]
impl TransactionService for DefaultTransactionService {
    async fn create_transaction(
        &self,
        user_id: Uuid,
        ticket_id: Option<Uuid>,
        amount: i64,
        description: String,
        payment_method: String,
    ) -> Result<Transaction, Box<dyn Error + Send + Sync + 'static>> {
        if amount <= 0 {
            return Err("Transaction amount must be positive".into());
        }

        let transaction = Transaction::new(user_id, ticket_id, amount, description, payment_method);

        self.transaction_repository.save(&transaction).await
    }

    async fn process_payment(
        &self,
        transaction_id: Uuid,
        external_reference: Option<String>,
    ) -> Result<Transaction, Box<dyn Error + Send + Sync + 'static>> {
        let transaction = match self
            .transaction_repository
            .find_by_id(transaction_id)
            .await?
        {
            Some(t) => t,
            None => return Err("Transaction not found".into()),
        };

        if transaction.is_finalized() {
            return Err("Transaction is already finalized".into());
        }

        if let Some(ref_id) = external_reference {
            let mut updated = self
                .transaction_repository
                .update_status(transaction_id, TransactionStatus::Success)
                .await?;
            updated.external_reference = Some(ref_id);
            return self.transaction_repository.save(&updated).await;
        }

        let (success, reference) = self.payment_service.process_payment(&transaction).await?;

        let status = if success {
            TransactionStatus::Success
        } else {
            TransactionStatus::Failed
        };

        let mut updated_transaction = self
            .transaction_repository
            .update_status(transaction_id, status)
            .await?;
        updated_transaction.external_reference = reference;
        updated_transaction.updated_at = Utc::now();

        self.transaction_repository.save(&updated_transaction).await
    }

    async fn validate_payment(
        &self,
        transaction_id: Uuid,
    ) -> Result<bool, Box<dyn Error + Send + Sync + 'static>> {
        let transaction = match self
            .transaction_repository
            .find_by_id(transaction_id)
            .await?
        {
            Some(t) => t,
            None => return Err("Transaction not found".into()),
        };

        Ok(transaction.status == TransactionStatus::Success)
    }

    async fn refund_transaction(
        &self,
        transaction_id: Uuid,
    ) -> Result<Transaction, Box<dyn Error + Send + Sync + 'static>> {
        let mut transaction = match self
            .transaction_repository
            .find_by_id(transaction_id)
            .await?
        {
            Some(t) => t,
            None => return Err("Transaction not found".into()),
        };

        transaction
            .refund()
            .map_err(|e| -> Box<dyn Error + Send + Sync + 'static> { e.into() })?;

        self.transaction_repository
            .update_status(transaction_id, TransactionStatus::Refunded)
            .await
    }

    async fn get_transaction(
        &self,
        transaction_id: Uuid,
    ) -> Result<Option<Transaction>, Box<dyn Error + Send + Sync + 'static>> {
        self.transaction_repository.find_by_id(transaction_id).await
    }

    async fn get_user_transactions(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync + 'static>> {
        self.transaction_repository.find_by_user(user_id).await
    }

    async fn add_funds_to_balance(
        &self,
        user_id: Uuid,
        amount: i64,
        payment_method: String,
    ) -> Result<(Transaction, i64), Box<dyn Error + Send + Sync + 'static>> {
        if amount <= 0 {
            return Err("Amount must be positive".into());
        }

        let transaction = self
            .create_transaction(
                user_id,
                None,
                amount,
                "Add funds to balance".to_string(),
                payment_method,
            )
            .await?;

        let processed_transaction = self.process_payment(transaction.id, None).await?;

        if processed_transaction.status != TransactionStatus::Success {
            return Err("Payment processing failed".into());
        }

        let new_balance = self.balance_service.add_funds(user_id, amount).await?;

        Ok((processed_transaction, new_balance))
    }

    async fn withdraw_funds(
        &self,
        user_id: Uuid,
        amount: i64,
        description: String,
    ) -> Result<(Transaction, i64), Box<dyn Error + Send + Sync + 'static>> {
        if amount <= 0 {
            return Err("Amount must be positive".into());
        }

        let balance = self.balance_service.get_or_create_balance(user_id).await?;
        if balance.amount < amount {
            return Err("Insufficient funds".into());
        }

        let transaction = self
            .create_transaction(user_id, None, amount, description, "Balance".to_string())
            .await?;

        let mut processed_transaction = self
            .transaction_repository
            .update_status(transaction.id, TransactionStatus::Success)
            .await?;

        processed_transaction.amount = -amount;
        let processed_transaction = self
            .transaction_repository
            .save(&processed_transaction)
            .await?;

        let new_balance = self.balance_service.withdraw_funds(user_id, amount).await?;        Ok((processed_transaction, new_balance))
    }

    async fn get_user_balance(
        &self,
        user_id: Uuid,
    ) -> Result<Option<crate::model::transaction::Balance>, Box<dyn Error + Send + Sync + 'static>> {
        self.balance_service.get_user_balance(user_id).await
    }

    async fn delete_transaction(
        &self,
        transaction_id: Uuid,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        let transaction = match self
            .transaction_repository
            .find_by_id(transaction_id)
            .await?
        {
            Some(t) => t,
            None => return Err("Transaction not found".into()),
        };

        if transaction.status != TransactionStatus::Pending {
            return Err("Cannot delete a processed transaction".into());
        }

        self.transaction_repository.delete(transaction_id).await
    }
}
