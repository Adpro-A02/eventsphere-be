use rocket::{routes, Route, State};
use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::serde::Deserialize;
use uuid::Uuid;
use std::sync::Arc;

use crate::service::transaction::TransactionService;
use crate::model::transaction::Transaction;
use crate::common::response::ApiResponse;

/// Collection of transaction-related routes
pub fn routes() -> Vec<Route> {
    routes![
        create_transaction, 
        get_transaction,
        get_user_transactions,
        process_payment,
        refund_transaction,
        validate_payment,
        add_funds,
        withdraw_funds,
        delete_transaction
    ]
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateTransactionRequest {
    pub user_id: String,
    pub ticket_id: Option<String>,
    pub amount: i64,
    pub description: String,
    pub payment_method: String,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ProcessPaymentRequest {
    pub external_reference: Option<String>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AddFundsRequest {
    pub amount: i64,
    pub payment_method: String,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct WithdrawFundsRequest {
    pub amount: i64,
    pub description: String,
}

/// Structure for balance operation responses
#[derive(serde::Serialize)]
struct BalanceResponse {
    transaction: Transaction,
    balance: i64,
}

/// Create a new transaction
#[post("/transactions", format = "json", data = "<request>")]
async fn create_transaction(
    service: &State<Arc<dyn TransactionService + Send + Sync>>,
    request: Json<CreateTransactionRequest>,
) -> Result<Json<ApiResponse<Transaction>>, Status> {
    let user_id = match Uuid::parse_str(&request.user_id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest),
    };
    
    let ticket_id = match request.ticket_id {
        Some(ref id_str) => {
            match Uuid::parse_str(id_str) {
                Ok(id) => Some(id),
                Err(_) => return Err(Status::BadRequest),
            }
        },
        None => None,
    };
    
    match service.create_transaction(
        user_id,
        ticket_id,
        request.amount,
        request.description.clone(),
        request.payment_method.clone()
    ) {
        Ok(transaction) => Ok(ApiResponse::created("Transaction created successfully", transaction)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// Get transaction by ID
#[get("/transactions/<id>")]
async fn get_transaction(
    service: &State<Arc<dyn TransactionService + Send + Sync>>,
    id: &str,
) -> Result<Json<ApiResponse<Option<Transaction>>>, Status> {
    let transaction_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest),
    };
    
    match service.get_transaction(transaction_id) {
        Ok(transaction) => Ok(ApiResponse::success("Transaction retrieved", transaction)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// Get all transactions for a user
#[get("/users/<id>/transactions")]
async fn get_user_transactions(
    service: &State<Arc<dyn TransactionService + Send + Sync>>,
    id: &str,
) -> Result<Json<ApiResponse<Vec<Transaction>>>, Status> {
    let user_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest),
    };
    
    match service.get_user_transactions(user_id) {
        Ok(transactions) => Ok(ApiResponse::success("User transactions retrieved", transactions)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// Process a payment for a transaction
#[put("/transactions/<id>/process", format = "json", data = "<request>")]
async fn process_payment(
    service: &State<Arc<dyn TransactionService + Send + Sync>>,
    id: &str,
    request: Json<ProcessPaymentRequest>,
) -> Result<Json<ApiResponse<Transaction>>, Status> {
    let transaction_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest),
    };
    
    match service.process_payment(transaction_id, request.external_reference.clone()) {
        Ok(transaction) => Ok(ApiResponse::success("Payment processed successfully", transaction)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// Refund a transaction
#[put("/transactions/<id>/refund")]
async fn refund_transaction(
    service: &State<Arc<dyn TransactionService + Send + Sync>>,
    id: &str,
) -> Result<Json<ApiResponse<Transaction>>, Status> {
    let transaction_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest),
    };
    
    match service.refund_transaction(transaction_id) {
        Ok(transaction) => Ok(ApiResponse::success("Transaction refunded successfully", transaction)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// Validate a payment
#[get("/transactions/<id>/validate")]
async fn validate_payment(
    service: &State<Arc<dyn TransactionService + Send + Sync>>,
    id: &str,
) -> Result<Json<ApiResponse<bool>>, Status> {
    let transaction_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest),
    };
    
    match service.validate_payment(transaction_id) {
        Ok(valid) => Ok(ApiResponse::success("Payment validation completed", valid)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// Add funds to user balance
#[post("/users/<id>/balance/add", format = "json", data = "<request>")]
async fn add_funds(
    service: &State<Arc<dyn TransactionService + Send + Sync>>,
    id: &str,
    request: Json<AddFundsRequest>,
) -> Result<Json<ApiResponse<BalanceResponse>>, Status> {
    let user_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest),
    };
    
    match service.add_funds_to_balance(user_id, request.amount, request.payment_method.clone()) {
        Ok((transaction, balance)) => {
            let response = BalanceResponse { transaction, balance };
            Ok(ApiResponse::success("Funds added successfully", response))
        },
        Err(_) => Err(Status::InternalServerError),
    }
}

/// Withdraw funds from user balance
#[post("/users/<id>/balance/withdraw", format = "json", data = "<request>")]
async fn withdraw_funds(
    service: &State<Arc<dyn TransactionService + Send + Sync>>,
    id: &str,
    request: Json<WithdrawFundsRequest>,
) -> Result<Json<ApiResponse<BalanceResponse>>, Status> {
    let user_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest),
    };
    
    match service.withdraw_funds(user_id, request.amount, request.description.clone()) {
        Ok((transaction, balance)) => {
            let response = BalanceResponse { transaction, balance };
            Ok(ApiResponse::success("Funds withdrawn successfully", response))
        },
        Err(e) if e.to_string().contains("Insufficient funds") => Err(Status::BadRequest),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// Delete a transaction
#[delete("/transactions/<id>")]
async fn delete_transaction(
    service: &State<Arc<dyn TransactionService + Send + Sync>>,
    id: &str,
) -> Result<Json<ApiResponse<()>>, Status> {
    let transaction_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest),
    };
    
    match service.delete_transaction(transaction_id) {
        Ok(_) => Ok(ApiResponse::success("Transaction deleted successfully", ())),
        Err(_) => Err(Status::InternalServerError),
    }
}
