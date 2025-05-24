use chrono::Utc;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

use crate::model::transaction::{Balance, Transaction, TransactionStatus};
use crate::service::transaction::TransactionService;
use crate::controller::transaction::transaction_controller::{
    AddFundsRequest, ApiResponse, BalanceResponse, CreateTransactionRequest, ProcessPaymentRequest,
    WithdrawFundsRequest,
};

struct MockTransactionService {
    transactions: Mutex<HashMap<Uuid, Transaction>>,
    balances: Mutex<HashMap<Uuid, Balance>>,
}

impl MockTransactionService {
    fn new() -> Self {
        Self {
            transactions: Mutex::new(HashMap::new()),
            balances: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl TransactionService for MockTransactionService {
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
        let mut transactions = self.transactions.lock().unwrap();
        transactions.insert(transaction.id, transaction.clone());
        Ok(transaction)
    }

    async fn process_payment(
        &self,
        transaction_id: Uuid,
        external_reference: Option<String>,
    ) -> Result<Transaction, Box<dyn Error + Send + Sync + 'static>> {
        let mut transactions = self.transactions.lock().unwrap();
        if let Some(transaction) = transactions.get_mut(&transaction_id) {
            if transaction.is_finalized() {
                return Err("Transaction is already finalized".into());
            }
            transaction.status = TransactionStatus::Success;
            transaction.external_reference =
                external_reference.or_else(|| Some(format!("PG-REF-{}", Uuid::new_v4())));
            transaction.updated_at = Utc::now();
            Ok(transaction.clone())
        } else {
            Err("Transaction not found".into())
        }
    }

    async fn validate_payment(
        &self,
        transaction_id: Uuid,
    ) -> Result<bool, Box<dyn Error + Send + Sync + 'static>> {
        let transactions = self.transactions.lock().unwrap();
        if let Some(transaction) = transactions.get(&transaction_id) {
            Ok(transaction.status == TransactionStatus::Success)
        } else {
            Err("Transaction not found".into())
        }
    }

    async fn refund_transaction(
        &self,
        transaction_id: Uuid,
    ) -> Result<Transaction, Box<dyn Error + Send + Sync + 'static>> {
        let mut transactions = self.transactions.lock().unwrap();
        if let Some(transaction) = transactions.get_mut(&transaction_id) {
            if transaction.status != TransactionStatus::Success {
                return Err("Only successful transactions can be refunded".into());
            }
            transaction.status = TransactionStatus::Refunded;
            transaction.updated_at = Utc::now();
            Ok(transaction.clone())
        } else {
            Err("Transaction not found".into())
        }
    }

    async fn get_transaction(
        &self,
        transaction_id: Uuid,
    ) -> Result<Option<Transaction>, Box<dyn Error + Send + Sync + 'static>> {
        let transactions = self.transactions.lock().unwrap();
        Ok(transactions.get(&transaction_id).cloned())
    }

    async fn get_user_transactions(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync + 'static>> {
        let transactions = self.transactions.lock().unwrap();
        Ok(transactions
            .values()
            .filter(|t| t.user_id == user_id)
            .cloned()
            .collect())
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
        let mut balances = self.balances.lock().unwrap();
        let balance = balances
            .entry(user_id)
            .or_insert_with(|| Balance::new(user_id));
        let new_amount = balance.add_funds(amount).map_err(|e| e.to_string())?;
        Ok((processed_transaction, new_amount))
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

        {
            let mut balances_guard = self.balances.lock().unwrap();
            let balance = balances_guard
                .entry(user_id)
                .or_insert_with(|| Balance::new(user_id));
            if balance.amount < amount {
                return Err("Insufficient funds".into());
            }
        }

        let mut transaction = self
            .create_transaction(user_id, None, amount, description, "Balance".to_string())
            .await?;
        transaction.amount = -amount;

        transaction.status = TransactionStatus::Success;
        transaction.updated_at = Utc::now();

        {
            let mut transactions_lock = self.transactions.lock().unwrap();
            transactions_lock.insert(transaction.id, transaction.clone()); // Save the updated transaction
        }

        let new_balance_amount;
        {
            let mut balances_guard = self.balances.lock().unwrap();
            let balance_entry = balances_guard
                .entry(user_id)
                .or_insert_with(|| Balance::new(user_id));

            new_balance_amount = balance_entry
                .withdraw(amount)
                .map_err(|e| Box::<dyn Error + Send + Sync + 'static>::from(e.to_string()))?;
        }        Ok((transaction, new_balance_amount))
    }

    async fn get_user_balance(
        &self,
        user_id: Uuid,
    ) -> Result<Option<crate::model::transaction::Balance>, Box<dyn Error + Send + Sync + 'static>> {
        let balances = self.balances.lock().unwrap();
        Ok(balances.get(&user_id).cloned())
    }

    async fn delete_transaction(
        &self,
        transaction_id: Uuid,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        let mut transactions = self.transactions.lock().unwrap();
        if let Some(transaction) = transactions.get(&transaction_id) {
            if transaction.status != TransactionStatus::Pending {
                return Err("Cannot delete a processed transaction".into());
            }
            transactions.remove(&transaction_id);
            Ok(())
        } else {
            Err("Transaction not found".into())
        }
    }
}

fn build_error_reply(
    message: String,
    default_status: StatusCode,
) -> Result<warp::reply::WithStatus<warp::reply::Json>, Rejection> {
    let status_code = match message.as_str() {
        "Transaction amount must be positive" => StatusCode::BAD_REQUEST,
        "Amount must be positive" => StatusCode::BAD_REQUEST,
        "Insufficient funds" => StatusCode::BAD_REQUEST,
        "Transaction is already finalized" => StatusCode::BAD_REQUEST,
        "Only successful transactions can be refunded" => StatusCode::BAD_REQUEST,
        "Cannot delete a processed transaction" => StatusCode::BAD_REQUEST,
        "Transaction not found" => StatusCode::NOT_FOUND,
        _ => default_status,
    };
    let response = ApiResponse {
        success: false,
        status_code: status_code.as_u16(),
        message,
        data: None::<()>,
    };
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        status_code,
    ))
}

async fn create_transaction_handler_for_test(
    req: CreateTransactionRequest,
    service: Arc<MockTransactionService>,
) -> Result<impl Reply, Rejection> {
    match service
        .create_transaction(
            req.user_id,
            req.ticket_id,
            req.amount,
            req.description,
            req.payment_method,
        )
        .await
    {
        Ok(transaction) => {
            let response = ApiResponse {
                success: true,
                status_code: StatusCode::OK.as_u16(),
                message: "Transaction created".to_string(),
                data: Some(transaction),
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::OK,
            ))
        }
        Err(e) => build_error_reply(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn process_payment_handler_for_test(
    transaction_id: Uuid,
    req: ProcessPaymentRequest,
    service: Arc<MockTransactionService>,
) -> Result<impl Reply, Rejection> {
    match service
        .process_payment(transaction_id, req.external_reference)
        .await
    {
        Ok(transaction) => {
            let response = ApiResponse {
                success: true,
                status_code: StatusCode::OK.as_u16(),
                message: "Payment processed".to_string(),
                data: Some(transaction),
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::OK,
            ))
        }
        Err(e) => build_error_reply(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn validate_payment_handler_for_test(
    transaction_id: Uuid,
    service: Arc<MockTransactionService>,
) -> Result<impl Reply, Rejection> {
    match service.validate_payment(transaction_id).await {
        Ok(is_valid) => {
            let response = ApiResponse {
                success: true,
                status_code: StatusCode::OK.as_u16(),
                message: "Payment validated".to_string(),
                data: Some(is_valid),
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::OK,
            ))
        }
        Err(e) => build_error_reply(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn refund_transaction_handler_for_test(
    transaction_id: Uuid,
    service: Arc<MockTransactionService>,
) -> Result<impl Reply, Rejection> {
    match service.refund_transaction(transaction_id).await {
        Ok(transaction) => {
            let response = ApiResponse {
                success: true,
                status_code: StatusCode::OK.as_u16(),
                message: "Transaction refunded".to_string(),
                data: Some(transaction),
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::OK,
            ))
        }
        Err(e) => build_error_reply(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_transaction_handler_for_test(
    transaction_id: Uuid,
    service: Arc<MockTransactionService>,
) -> Result<impl Reply, Rejection> {
    match service.get_transaction(transaction_id).await {
        Ok(Some(transaction)) => {
            let response = ApiResponse {
                success: true,
                status_code: StatusCode::OK.as_u16(),
                message: "Transaction found".to_string(),
                data: Some(transaction),
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::OK,
            ))
        }
        Ok(None) => {
            let response = ApiResponse {
                success: false,
                status_code: StatusCode::NOT_FOUND.as_u16(),
                message: "Transaction not found".to_string(),
                data: None::<Transaction>,
            }; // Ensure correct type for None
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::NOT_FOUND,
            ))
        }
        Err(e) => build_error_reply(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_user_transactions_handler_for_test(
    user_id: Uuid,
    service: Arc<MockTransactionService>,
) -> Result<impl Reply, Rejection> {
    match service.get_user_transactions(user_id).await {
        Ok(transactions) => {
            let response = ApiResponse {
                success: true,
                status_code: StatusCode::OK.as_u16(),
                message: "User transactions found".to_string(),
                data: Some(transactions),
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::OK,
            ))
        }
        Err(e) => build_error_reply(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn add_funds_handler_for_test(
    req: AddFundsRequest,
    service: Arc<MockTransactionService>,
) -> Result<impl Reply, Rejection> {
    match service
        .add_funds_to_balance(req.user_id, req.amount, req.payment_method)
        .await
    {
        Ok((transaction, balance)) => {
            let data = BalanceResponse {
                transaction,
                balance,
            };
            let response = ApiResponse {
                success: true,
                status_code: StatusCode::OK.as_u16(),
                message: "Funds added".to_string(),
                data: Some(data),
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::OK,
            ))
        }
        Err(e) => build_error_reply(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn withdraw_funds_handler_for_test(
    req: WithdrawFundsRequest,
    service: Arc<MockTransactionService>,
) -> Result<impl Reply, Rejection> {
    match service
        .withdraw_funds(req.user_id, req.amount, req.description)
        .await
    {
        Ok((transaction, balance)) => {
            let data = BalanceResponse {
                transaction,
                balance,
            };
            let response = ApiResponse {
                success: true,
                status_code: StatusCode::OK.as_u16(),
                message: "Funds withdrawn".to_string(),
                data: Some(data),
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::OK,
            ))
        }
        Err(e) => build_error_reply(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_transaction_handler_for_test(
    transaction_id: Uuid,
    service: Arc<MockTransactionService>,
) -> Result<impl Reply, Rejection> {
    match service.delete_transaction(transaction_id).await {
        Ok(()) => {
            let response = ApiResponse {
                success: true,
                status_code: StatusCode::OK.as_u16(),
                message: "Transaction deleted".to_string(),
                data: Some(()),
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::OK,
            ))
        }
        Err(e) => build_error_reply(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
    }
}

fn create_test_routes() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let service = Arc::new(MockTransactionService::new());

    let with_service = warp::any().map(move || Arc::clone(&service));

    let create_transaction = warp::post()
        .and(warp::path("transactions"))
        .and(warp::body::json())
        .and(with_service.clone())
        .and_then(create_transaction_handler_for_test);

    let process_payment = warp::post()
        .and(warp::path!("transactions" / Uuid / "process"))
        .and(warp::body::json())
        .and(with_service.clone())
        .and_then(process_payment_handler_for_test);

    let validate_payment = warp::get()
        .and(warp::path!("transactions" / Uuid / "validate"))
        .and(with_service.clone())
        .and_then(validate_payment_handler_for_test);

    let refund_transaction = warp::post()
        .and(warp::path!("transactions" / Uuid / "refund"))
        .and(with_service.clone())
        .and_then(refund_transaction_handler_for_test);

    let get_transaction = warp::get()
        .and(warp::path!("transactions" / Uuid))
        .and(with_service.clone())
        .and_then(get_transaction_handler_for_test);

    let get_user_transactions = warp::get()
        .and(warp::path!("users" / Uuid / "transactions"))
        .and(with_service.clone())
        .and_then(get_user_transactions_handler_for_test);

    let add_funds = warp::post()
        .and(warp::path!("balance" / "add"))
        .and(warp::body::json())
        .and(with_service.clone())
        .and_then(add_funds_handler_for_test);

    let withdraw_funds = warp::post()
        .and(warp::path!("balance" / "withdraw"))
        .and(warp::body::json())
        .and(with_service.clone())
        .and_then(withdraw_funds_handler_for_test);

    let delete_transaction = warp::delete()
        .and(warp::path!("transactions" / Uuid))
        .and(with_service.clone())
        .and_then(delete_transaction_handler_for_test);

    create_transaction
        .or(process_payment)
        .or(validate_payment)
        .or(refund_transaction)
        .or(get_transaction)
        .or(get_user_transactions)
        .or(add_funds)
        .or(withdraw_funds)
        .or(delete_transaction)
}
