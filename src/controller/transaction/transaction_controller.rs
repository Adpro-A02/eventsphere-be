use rocket::http::uri::fmt::{FromUriParam, Part, UriDisplay};
use rocket::request::FromParam;
use rocket::{Route, State, delete, get, http::Status, post, put, routes, serde::json::Json};
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use crate::model::transaction::{Transaction, Balance};
use crate::service::transaction::transaction_service::TransactionService;

pub struct UuidParam(pub Uuid);

impl<'a> FromParam<'a> for UuidParam {
    type Error = uuid::Error;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        Uuid::from_str(param).map(UuidParam)
    }
}

impl From<UuidParam> for Uuid {
    fn from(param: UuidParam) -> Self {
        param.0
    }
}

impl<P: Part> UriDisplay<P> for UuidParam {
    fn fmt(&self, f: &mut rocket::http::uri::fmt::Formatter<'_, P>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

impl<'a, P: Part> FromUriParam<P, &'a Uuid> for UuidParam {
    type Target = UuidParam;
    fn from_uri_param(param: &'a Uuid) -> Self::Target {
        UuidParam(*param)
    }
}

impl<P: Part> FromUriParam<P, Uuid> for UuidParam {
    type Target = UuidParam;
    fn from_uri_param(param: Uuid) -> Self::Target {
        UuidParam(param)
    }
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub success: bool,
    pub status_code: u16,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn success(message: &str, data: T) -> Json<Self> {
        Json(Self {
            success: true,
            status_code: 200,
            message: message.to_string(),
            data: Some(data),
        })
    }

    pub fn success_no_data(message: &str, status_code: u16) -> Json<Self> {
        Json(Self {
            success: true,
            status_code,
            message: message.to_string(),
            data: None,
        })
    }

    pub fn error(status_code: u16, message: &str) -> Json<Self> {
        Json(Self {
            success: false,
            status_code,
            message: message.to_string(),
            data: None,
        })
    }
}

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

pub fn transaction_routes() -> Vec<Route> {
    routes![
        create_transaction_handler,
        process_payment_handler,
        validate_payment_handler,
        refund_transaction_handler,
        get_transaction_handler,
        get_user_transactions_handler,
        get_user_balance_handler,
        add_funds_handler,
        withdraw_funds_handler,
        delete_transaction_handler
    ]
}

#[post("/transactions", data = "<req>")]
pub async fn create_transaction_handler(
    token: crate::middleware::auth::JwtToken,
    req: Json<CreateTransactionRequest>,
    service: &State<Arc<dyn TransactionService + Send + Sync>>,
) -> Result<Json<ApiResponse<Transaction>>, Status> {
    // Verify the authenticated user matches the user_id in the request or is admin
    let token_user_id = match uuid::Uuid::parse_str(&token.user_id) {
        Ok(id) => id,
        Err(_) => return Err(Status::Unauthorized),
    };
    
    if token_user_id != req.user_id && !token.is_admin() {
        return Err(Status::Forbidden);
    }

    match service
        .create_transaction(
            req.user_id,
            req.ticket_id,
            req.amount,
            req.description.clone(),
            req.payment_method.clone(),
        )
        .await
    {
        Ok(transaction) => Ok(ApiResponse::success(
            "Transaction created successfully",
            transaction,
        )),
        Err(e) => {
            eprintln!("Failed to create transaction: {:?}", e);
            Ok(ApiResponse::error(
                500,
                &format!("Failed to create transaction: {}", e),
            ))
        }
    }
}

#[put("/transactions/<transaction_id>/process", data = "<req>")]
pub async fn process_payment_handler(
    token: crate::middleware::auth::JwtToken,
    transaction_id: UuidParam,
    req: Json<ProcessPaymentRequest>,
    service: &State<Arc<dyn TransactionService + Send + Sync>>,
) -> Result<Json<ApiResponse<Transaction>>, Status> {
    // Check if the transaction belongs to the authenticated user or user is admin
    let token_user_id = match uuid::Uuid::parse_str(&token.user_id) {
        Ok(id) => id,
        Err(_) => return Err(Status::Unauthorized),
    };

    // First get the transaction to verify ownership
    let transaction = match service.get_transaction(transaction_id.0).await {
        Ok(Some(t)) => t,
        Ok(None) => return Ok(ApiResponse::error(404, "Transaction not found")),
        Err(e) => return Ok(ApiResponse::error(500, &format!("Failed to get transaction: {}", e))),
    };

    if transaction.user_id != token_user_id && !token.is_admin() {
        return Err(Status::Forbidden);
    }

    match service
        .process_payment(transaction_id.0, req.external_reference.clone())
        .await
    {
        Ok(transaction) => Ok(ApiResponse::success(
            "Payment processed successfully",
            transaction,
        )),
        Err(e) => {
            eprintln!("Failed to process payment: {:?}", e);
            Ok(ApiResponse::error(
                500,
                &format!("Failed to process payment: {}", e),
            ))
        }
    }
}

#[get("/transactions/<transaction_id>/validate")]
pub async fn validate_payment_handler(
    token: crate::middleware::auth::JwtToken,
    transaction_id: UuidParam,
    service: &State<Arc<dyn TransactionService + Send + Sync>>,
) -> Result<Json<ApiResponse<bool>>, Status> {
    // Check if the transaction belongs to the authenticated user or user is admin
    let token_user_id = match uuid::Uuid::parse_str(&token.user_id) {
        Ok(id) => id,
        Err(_) => return Err(Status::Unauthorized),
    };

    // First get the transaction to verify ownership
    let transaction = match service.get_transaction(transaction_id.0).await {
        Ok(Some(t)) => t,
        Ok(None) => return Ok(ApiResponse::error(404, "Transaction not found")),
        Err(e) => return Ok(ApiResponse::error(500, &format!("Failed to get transaction: {}", e))),
    };

    if transaction.user_id != token_user_id && !token.is_admin() {
        return Err(Status::Forbidden);
    }

    match service.validate_payment(transaction_id.0).await {
        Ok(is_valid) => Ok(ApiResponse::success(
            "Payment validation completed",
            is_valid,
        )),
        Err(e) => {
            eprintln!("Failed to validate payment: {:?}", e);
            Ok(ApiResponse::error(
                500,
                &format!("Failed to validate payment: {}", e),
            ))
        }
    }
}

#[put("/transactions/<transaction_id>/refund")]
pub async fn refund_transaction_handler(
    token: crate::middleware::auth::JwtToken,
    transaction_id: UuidParam,
    service: &State<Arc<dyn TransactionService + Send + Sync>>,
) -> Result<Json<ApiResponse<Transaction>>, Status> {
    // Check if the transaction belongs to the authenticated user or user is admin
    let token_user_id = match uuid::Uuid::parse_str(&token.user_id) {
        Ok(id) => id,
        Err(_) => return Err(Status::Unauthorized),
    };

    // First get the transaction to verify ownership
    let transaction = match service.get_transaction(transaction_id.0).await {
        Ok(Some(t)) => t,
        Ok(None) => return Ok(ApiResponse::error(404, "Transaction not found")),
        Err(e) => return Ok(ApiResponse::error(500, &format!("Failed to get transaction: {}", e))),
    };

    if transaction.user_id != token_user_id && !token.is_admin() {
        return Err(Status::Forbidden);
    }

    match service.refund_transaction(transaction_id.0).await {
        Ok(transaction) => Ok(ApiResponse::success(
            "Transaction refunded successfully",
            transaction,
        )),
        Err(e) => {
            eprintln!("Failed to refund transaction: {:?}", e);
            Ok(ApiResponse::error(
                500,
                &format!("Failed to refund transaction: {}", e),
            ))
        }
    }
}

#[get("/transactions/<transaction_id>")]
pub async fn get_transaction_handler(
    token: crate::middleware::auth::JwtToken,
    transaction_id: UuidParam,
    service: &State<Arc<dyn TransactionService + Send + Sync>>,
) -> Result<Json<ApiResponse<Transaction>>, Status> {
    let token_user_id = match uuid::Uuid::parse_str(&token.user_id) {
        Ok(id) => id,
        Err(_) => return Err(Status::Unauthorized),
    };

    match service.get_transaction(transaction_id.0).await {
        Ok(Some(transaction)) => {
            // Verify the transaction belongs to the authenticated user or user is admin
            if transaction.user_id != token_user_id && !token.is_admin() {
                return Err(Status::Forbidden);
            }
            Ok(ApiResponse::success("Transaction found", transaction))
        },
        Ok(None) => Ok(ApiResponse::error(404, "Transaction not found")),
        Err(e) => {
            eprintln!("Failed to get transaction: {:?}", e);
            Ok(ApiResponse::error(
                500,
                &format!("Failed to get transaction: {}", e),
            ))
        }
    }
}

#[get("/users/<user_id>/transactions")]
pub async fn get_user_transactions_handler(
    token: crate::middleware::auth::JwtToken,
    user_id: UuidParam,
    service: &State<Arc<dyn TransactionService + Send + Sync>>,
) -> Result<Json<ApiResponse<Vec<Transaction>>>, Status> {
    let token_user_id = match uuid::Uuid::parse_str(&token.user_id) {
        Ok(id) => id,
        Err(_) => return Err(Status::Unauthorized),
    };

    // Verify the requested user_id matches the authenticated user or user is admin
    if user_id.0 != token_user_id && !token.is_admin() {
        return Err(Status::Forbidden);
    }

    match service.get_user_transactions(user_id.0).await {
        Ok(transactions) => Ok(ApiResponse::success(
            "User transactions found",
            transactions,
        )),
        Err(e) => {
            eprintln!("Failed to get user transactions: {:?}", e);
            Ok(ApiResponse::error(
                500,
                &format!("Failed to get user transactions: {}", e),
            ))
        }    }
}

#[get("/users/<user_id>/balance")]
pub async fn get_user_balance_handler(
    token: crate::middleware::auth::JwtToken,
    user_id: UuidParam,
    service: &State<Arc<dyn TransactionService + Send + Sync>>,
) -> Result<Json<ApiResponse<Balance>>, Status> {
    let token_user_id = match uuid::Uuid::parse_str(&token.user_id) {
        Ok(id) => id,
        Err(_) => return Err(Status::Unauthorized),
    };

    // Verify the requested user_id matches the authenticated user or user is admin
    if user_id.0 != token_user_id && !token.is_admin() {
        return Err(Status::Forbidden);
    }

    match service.get_user_balance(user_id.0).await {
        Ok(Some(balance)) => Ok(ApiResponse::success(
            "User balance found",
            balance,
        )),
        Ok(None) => Ok(ApiResponse::error(404, "User balance not found")),
        Err(e) => {
            eprintln!("Failed to get user balance: {:?}", e);
            Ok(ApiResponse::error(
                500,
                &format!("Failed to get user balance: {}", e),
            ))
        }
    }
}

#[post("/balance/add", data = "<req>")]
pub async fn add_funds_handler(
    token: crate::middleware::auth::JwtToken,
    req: Json<AddFundsRequest>,
    service: &State<Arc<dyn TransactionService + Send + Sync>>,
) -> Result<Json<ApiResponse<BalanceResponse>>, Status> {
    // Verify the authenticated user matches the user_id in the request or is admin
    let token_user_id = match uuid::Uuid::parse_str(&token.user_id) {
        Ok(id) => id,
        Err(_) => return Err(Status::Unauthorized),
    };
    
    if token_user_id != req.user_id && !token.is_admin() {
        return Err(Status::Forbidden);
    }

    match service
        .add_funds_to_balance(req.user_id, req.amount, req.payment_method.clone())
        .await
    {
        Ok((transaction, balance)) => {
            let response = BalanceResponse {
                transaction,
                balance,
            };
            Ok(ApiResponse::success("Funds added successfully", response))
        }
        Err(e) => {
            eprintln!("Failed to add funds: {:?}", e);
            Ok(ApiResponse::error(
                500,
                &format!("Failed to add funds: {}", e),
            ))
        }
    }
}

#[post("/balance/withdraw", data = "<req>")]
pub async fn withdraw_funds_handler(
    token: crate::middleware::auth::JwtToken,
    req: Json<WithdrawFundsRequest>,
    service: &State<Arc<dyn TransactionService + Send + Sync>>,
) -> Result<Json<ApiResponse<BalanceResponse>>, Status> {
    // Verify the authenticated user matches the user_id in the request or is admin
    let token_user_id = match uuid::Uuid::parse_str(&token.user_id) {
        Ok(id) => id,
        Err(_) => return Err(Status::Unauthorized),
    };
    
    if token_user_id != req.user_id && !token.is_admin() {
        return Err(Status::Forbidden);
    }

    match service
        .withdraw_funds(req.user_id, req.amount, req.description.clone())
        .await
    {
        Ok((transaction, balance)) => {
            let response = BalanceResponse {
                transaction,
                balance,
            };
            Ok(ApiResponse::success(
                "Funds withdrawn successfully",
                response,
            ))
        }
        Err(e) => {
            eprintln!("Failed to withdraw funds: {:?}", e);
            Ok(ApiResponse::error(
                500,
                &format!("Failed to withdraw funds: {}", e),
            ))
        }
    }
}

#[delete("/transactions/<transaction_id>")]
pub async fn delete_transaction_handler(
    token: crate::middleware::auth::JwtToken,
    transaction_id: UuidParam,
    service: &State<Arc<dyn TransactionService + Send + Sync>>,
) -> Result<Json<ApiResponse<()>>, Status> {
    // Check if the transaction belongs to the authenticated user or user is admin
    let token_user_id = match uuid::Uuid::parse_str(&token.user_id) {
        Ok(id) => id,
        Err(_) => return Err(Status::Unauthorized),
    };

    // First get the transaction to verify ownership
    let transaction = match service.get_transaction(transaction_id.0).await {
        Ok(Some(t)) => t,
        Ok(None) => return Ok(ApiResponse::error(404, "Transaction not found")),
        Err(e) => return Ok(ApiResponse::error(500, &format!("Failed to get transaction: {}", e))),
    };

    if transaction.user_id != token_user_id && !token.is_admin() {
        return Err(Status::Forbidden);
    }

    match service.delete_transaction(transaction_id.0).await {
        Ok(_) => Ok(ApiResponse::success("Transaction deleted successfully", ())),
        Err(e) => {
            eprintln!("Failed to delete transaction: {:?}", e);
            Ok(ApiResponse::error(
                500,
                &format!("Failed to delete transaction: {}", e),
            ))
        }
    }
}
