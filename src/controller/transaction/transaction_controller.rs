use std::sync::Arc;
use uuid::Uuid;
use warp::{Filter, Rejection, Reply};
use serde::{Deserialize, Serialize};

use crate::model::transaction::Transaction;
use crate::service::transaction::TransactionService;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            status: "success".to_string(),
            message: None,
            data: Some(data),
        }
    }

    pub fn success_no_content() -> Self {
        Self {
            status: "success".to_string(),
            message: None,
            data: None,
        }
    }
    
    pub fn error(message: String) -> Self {
        Self {
            status: "error".to_string(),
            message: Some(message),
            data: None,
        }
    }
}

// Custom error type for rejections
#[derive(Debug)]
pub struct ApiError {
    pub message: String,
}

impl ApiError {
    pub fn from_error<E: std::fmt::Display>(err: E) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

impl warp::reject::Reject for ApiError {}

#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub user_id: Uuid,
    pub ticket_id: Option<Uuid>,
    pub amount: i64,
    pub description: String,
    pub payment_method: String,
}

#[derive(Debug, Deserialize)]
pub struct ProcessPaymentRequest {
    pub external_reference: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddFundsRequest {
    pub user_id: Uuid,
    pub amount: i64,
    pub payment_method: String,
}

#[derive(Debug, Deserialize)]
pub struct WithdrawFundsRequest {
    pub user_id: Uuid,
    pub amount: i64,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub transaction: Transaction,
    pub balance: i64,
}

pub struct TransactionController {
    transaction_service: Arc<dyn TransactionService + Send + Sync>,
}

impl TransactionController {
    pub fn new(transaction_service: Arc<dyn TransactionService + Send + Sync>) -> Self {
        Self { transaction_service }
    }

    pub fn routes<'a>(&'a self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone + 'a {
        let service = Arc::clone(&self.transaction_service);

        let create_transaction = warp::path!("transactions")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_service(service.clone()))
            .and_then(Self::create_transaction_handler);

        let process_payment = warp::path!("transactions" / Uuid / "process")
            .and(warp::put())
            .and(warp::body::json())
            .and(with_service(service.clone()))
            .and_then(Self::process_payment_handler);

        let validate_payment = warp::path!("transactions" / Uuid / "validate")
            .and(warp::get())
            .and(with_service(service.clone()))
            .and_then(Self::validate_payment_handler);

        let refund_transaction = warp::path!("transactions" / Uuid / "refund")
            .and(warp::put())
            .and(with_service(service.clone()))
            .and_then(Self::refund_transaction_handler);

        let get_transaction = warp::path!("transactions" / Uuid)
            .and(warp::get())
            .and(with_service(service.clone()))
            .and_then(Self::get_transaction_handler);

        let get_user_transactions = warp::path!("users" / Uuid / "transactions")
            .and(warp::get())
            .and(with_service(service.clone()))
            .and_then(Self::get_user_transactions_handler);

        let add_funds = warp::path!("balance" / "add")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_service(service.clone()))
            .and_then(Self::add_funds_handler);

        let withdraw_funds = warp::path!("balance" / "withdraw")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_service(service.clone()))
            .and_then(Self::withdraw_funds_handler);

        let delete_transaction = warp::path!("transactions" / Uuid)
            .and(warp::delete())
            .and(with_service(service.clone()))
            .and_then(Self::delete_transaction_handler);

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

    // Helper method to handle the common pattern of processing results and creating API responses
    fn handle_result<T: Serialize>(result: Result<T, Box<dyn std::error::Error>>) -> Result<impl Reply, Rejection> {
        match result {
            Ok(data) => Ok(warp::reply::json(&ApiResponse::success(data))),
            Err(e) => Err(warp::reject::custom(ApiError::from_error(e))),
        }
    }

    pub async fn create_transaction_handler(
        req: CreateTransactionRequest,
        service: Arc<dyn TransactionService + Send + Sync>,
    ) -> Result<impl Reply, Rejection> {
        let result = service.create_transaction(
            req.user_id,
            req.ticket_id,
            req.amount,
            req.description,
            req.payment_method,
        );
        
        Self::handle_result(result)
    }

    pub async fn process_payment_handler(
        transaction_id: Uuid,
        req: ProcessPaymentRequest,
        service: Arc<dyn TransactionService + Send + Sync>,
    ) -> Result<impl Reply, Rejection> {
        let result = service.process_payment(transaction_id, req.external_reference);
        Self::handle_result(result)
    }

    pub async fn validate_payment_handler(
        transaction_id: Uuid,
        service: Arc<dyn TransactionService + Send + Sync>,
    ) -> Result<impl Reply, Rejection> {
        let result = service.validate_payment(transaction_id);
        Self::handle_result(result)
    }

    pub async fn refund_transaction_handler(
        transaction_id: Uuid,
        service: Arc<dyn TransactionService + Send + Sync>,
    ) -> Result<impl Reply, Rejection> {
        let result = service.refund_transaction(transaction_id);
        Self::handle_result(result)
    }

    pub async fn get_transaction_handler(
        transaction_id: Uuid,
        service: Arc<dyn TransactionService + Send + Sync>,
    ) -> Result<impl Reply, Rejection> {
        let result = service.get_transaction(transaction_id);
        Self::handle_result(result)
    }

    pub async fn get_user_transactions_handler(
        user_id: Uuid,
        service: Arc<dyn TransactionService + Send + Sync>,
    ) -> Result<impl Reply, Rejection> {
        let result = service.get_user_transactions(user_id);
        Self::handle_result(result)
    }

    pub async fn add_funds_handler(
        req: AddFundsRequest,
        service: Arc<dyn TransactionService + Send + Sync>,
    ) -> Result<impl Reply, Rejection> {
        let result = service.add_funds_to_balance(req.user_id, req.amount, req.payment_method);
        
        match result {
            Ok((transaction, balance)) => {
                let response = BalanceResponse {
                    transaction,
                    balance,
                };
                Ok(warp::reply::json(&ApiResponse::success(response)))
            }
            Err(e) => Err(warp::reject::custom(ApiError::from_error(e))),
        }
    }

    pub async fn withdraw_funds_handler(
        req: WithdrawFundsRequest,
        service: Arc<dyn TransactionService + Send + Sync>,
    ) -> Result<impl Reply, Rejection> {
        let result = service.withdraw_funds(req.user_id, req.amount, req.description);
        
        match result {
            Ok((transaction, balance)) => {
                let response = BalanceResponse {
                    transaction,
                    balance,
                };
                Ok(warp::reply::json(&ApiResponse::success(response)))
            }
            Err(e) => Err(warp::reject::custom(ApiError::from_error(e))),
        }
    }

    pub async fn delete_transaction_handler(
        transaction_id: Uuid,
        service: Arc<dyn TransactionService + Send + Sync>,
    ) -> Result<impl Reply, Rejection> {
        let result = service.delete_transaction(transaction_id);
        
        match result {
            Ok(_) => Ok(warp::reply::json(&ApiResponse::<()>::success_no_content())),
            Err(e) => Err(warp::reject::custom(ApiError::from_error(e))),
        }
    }
}

fn with_service(
    service: Arc<dyn TransactionService + Send + Sync>,
) -> impl Filter<Extract = (Arc<dyn TransactionService + Send + Sync>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || service.clone())
}
