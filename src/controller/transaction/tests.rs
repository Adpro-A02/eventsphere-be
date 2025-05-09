#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;
    use uuid::Uuid;
    use warp::test::request;
    use warp::http::StatusCode;
    use chrono::Utc;
    use serde_json::{json, Value};
    use warp::{Filter, Reply, Rejection};

    use crate::model::transaction::{Transaction, TransactionStatus, Balance};
    use crate::service::transaction::TransactionService;
    use crate::controller::transaction::transaction_controller::{TransactionController, ApiResponse, ApiError};

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

    impl TransactionService for MockTransactionService {
        fn create_transaction(
            &self,
            user_id: Uuid,
            ticket_id: Option<Uuid>,
            amount: i64,
            description: String,
            payment_method: String,
        ) -> Result<Transaction, Box<dyn Error>> {
            if amount <= 0 {
                return Err("Transaction amount must be positive".into());
            }
            
            let transaction = Transaction::new(
                user_id,
                ticket_id,
                amount,
                description,
                payment_method,
            );
            
            let mut transactions = self.transactions.lock().unwrap();
            transactions.insert(transaction.id, transaction.clone());
            
            Ok(transaction)
        }

        fn process_payment(
            &self,
            transaction_id: Uuid,
            external_reference: Option<String>,
        ) -> Result<Transaction, Box<dyn Error>> {
            let mut transactions = self.transactions.lock().unwrap();
            
            if let Some(transaction) = transactions.get_mut(&transaction_id) {
                if transaction.is_finalized() {
                    return Err("Transaction is already finalized".into());
                }
                
                let status = TransactionStatus::Success;
                transaction.status = status;
                
                if let Some(ref_id) = external_reference {
                    transaction.external_reference = Some(ref_id);
                } else {
                    transaction.external_reference = Some(format!("PG-REF-{}", Uuid::new_v4()));
                }
                
                transaction.updated_at = Utc::now();
                return Ok(transaction.clone());
            }
            
            Err("Transaction not found".into())
        }

        fn validate_payment(&self, transaction_id: Uuid) -> Result<bool, Box<dyn Error>> {
            let transactions = self.transactions.lock().unwrap();
            
            if let Some(transaction) = transactions.get(&transaction_id) {
                return Ok(transaction.status == TransactionStatus::Success);
            }
            
            Err("Transaction not found".into())
        }

        fn refund_transaction(&self, transaction_id: Uuid) -> Result<Transaction, Box<dyn Error>> {
            let mut transactions = self.transactions.lock().unwrap();
            
            if let Some(transaction) = transactions.get_mut(&transaction_id) {
                if transaction.status != TransactionStatus::Success {
                    return Err("Only successful transactions can be refunded".into());
                }
                
                transaction.status = TransactionStatus::Refunded;
                transaction.updated_at = Utc::now();
                
                return Ok(transaction.clone());
            }
            
            Err("Transaction not found".into())
        }

        fn get_transaction(&self, transaction_id: Uuid) -> Result<Option<Transaction>, Box<dyn Error>> {
            let transactions = self.transactions.lock().unwrap();
            Ok(transactions.get(&transaction_id).cloned())
        }

        fn get_user_transactions(&self, user_id: Uuid) -> Result<Vec<Transaction>, Box<dyn Error>> {
            let transactions = self.transactions.lock().unwrap();
            
            let user_transactions = transactions
                .values()
                .filter(|t| t.user_id == user_id)
                .cloned()
                .collect();
                
            Ok(user_transactions)
        }

        fn add_funds_to_balance(
            &self,
            user_id: Uuid,
            amount: i64,
            payment_method: String,
        ) -> Result<(Transaction, i64), Box<dyn Error>> {
            if amount <= 0 {
                return Err("Amount must be positive".into());
            }
            
            let transaction = self.create_transaction(
                user_id,
                None,
                amount,
                "Add funds to balance".to_string(),
                payment_method,
            )?;
            
            let processed = self.process_payment(transaction.id, None)?;
            
            let mut balances = self.balances.lock().unwrap();
            let balance = balances.entry(user_id).or_insert_with(|| Balance::new(user_id));
            
            let new_amount = balance.add_funds(amount).map_err(|e| e.to_string())?;
            
            Ok((processed, new_amount))
        }

        fn withdraw_funds(
            &self,
            user_id: Uuid,
            amount: i64,
            description: String,
        ) -> Result<(Transaction, i64), Box<dyn Error>> {
            if amount <= 0 {
                return Err("Amount must be positive".into());
            }
            
            let mut balances = self.balances.lock().unwrap();
            let balance = balances.entry(user_id).or_insert_with(|| Balance::new(user_id));
            
            if balance.amount < amount {
                return Err("Insufficient funds".into());
            }
            
            let mut transaction = self.create_transaction(
                user_id,
                None,
                amount,
                description,
                "Balance".to_string(),
            )?;
            
            transaction.amount = -amount;
            let mut transactions = self.transactions.lock().unwrap();
            transactions.insert(transaction.id, transaction.clone());
            
            transaction.status = TransactionStatus::Success;
            transaction.updated_at = Utc::now();
            transactions.insert(transaction.id, transaction.clone());
            
            let new_amount = balance.withdraw(amount).map_err(|e| e.to_string())?;
            
            Ok((transaction, new_amount))
        }

        fn delete_transaction(&self, transaction_id: Uuid) -> Result<(), Box<dyn Error>> {
            let mut transactions = self.transactions.lock().unwrap();
            
            if let Some(transaction) = transactions.get(&transaction_id) {
                if transaction.status != TransactionStatus::Pending {
                    return Err("Cannot delete a processed transaction".into());
                }
                
                transactions.remove(&transaction_id);
                return Ok(());
            }
            
            Err("Transaction not found".into())
        }
    }

    async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
        if let Some(api_error) = err.find::<ApiError>() {
            let json = warp::reply::json(&ApiResponse::<()>::error(api_error.message.clone()));
            return Ok(warp::reply::with_status(json, StatusCode::BAD_REQUEST));
        }

        Err(err)
    }

    fn create_test_routes() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let service = Arc::new(MockTransactionService::new()) as Arc<dyn TransactionService + Send + Sync>;
        
        let service_clone = service.clone();
        let create_transaction = warp::path!("transactions")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || service_clone.clone()))
            .and_then(TransactionController::create_transaction_handler);

        let service_clone = service.clone();
        let process_payment = warp::path!("transactions" / Uuid / "process")
            .and(warp::put())
            .and(warp::body::json())
            .and(warp::any().map(move || service_clone.clone()))
            .and_then(TransactionController::process_payment_handler);

        let service_clone = service.clone();
        let validate_payment = warp::path!("transactions" / Uuid / "validate")
            .and(warp::get())
            .and(warp::any().map(move || service_clone.clone()))
            .and_then(TransactionController::validate_payment_handler);

        let service_clone = service.clone();
        let refund_transaction = warp::path!("transactions" / Uuid / "refund")
            .and(warp::put())
            .and(warp::any().map(move || service_clone.clone()))
            .and_then(TransactionController::refund_transaction_handler);

        let service_clone = service.clone();
        let get_transaction = warp::path!("transactions" / Uuid)
            .and(warp::get())
            .and(warp::any().map(move || service_clone.clone()))
            .and_then(TransactionController::get_transaction_handler);

        let service_clone = service.clone();
        let get_user_transactions = warp::path!("users" / Uuid / "transactions")
            .and(warp::get())
            .and(warp::any().map(move || service_clone.clone()))
            .and_then(TransactionController::get_user_transactions_handler);

        let service_clone = service.clone();
        let add_funds = warp::path!("balance" / "add")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || service_clone.clone()))
            .and_then(TransactionController::add_funds_handler);

        let service_clone = service.clone();
        let withdraw_funds = warp::path!("balance" / "withdraw")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || service_clone.clone()))
            .and_then(TransactionController::withdraw_funds_handler);

        let service_clone = service.clone();
        let delete_transaction = warp::path!("transactions" / Uuid)
            .and(warp::delete())
            .and(warp::any().map(move || service_clone.clone()))
            .and_then(TransactionController::delete_transaction_handler);

        create_transaction
            .or(process_payment)
            .or(validate_payment)
            .or(refund_transaction)
            .or(get_transaction)
            .or(get_user_transactions)
            .or(add_funds)
            .or(withdraw_funds)
            .or(delete_transaction)
            .recover(handle_rejection)
    }

    #[tokio::test]
    async fn test_create_transaction() {
        let routes = create_test_routes();

        let response = request()
            .method("POST")
            .path("/transactions")
            .json(&json!({
                "user_id": Uuid::new_v4(),
                "ticket_id": Uuid::new_v4(),
                "amount": 1000,
                "description": "Test transaction",
                "payment_method": "Credit Card"
            }))
            .reply(&routes)
            .await;

        assert_eq!(response.status(), StatusCode::OK);
        
        let body: Value = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(body["status"], "success");
        assert_eq!(body["data"]["amount"], 1000);
        assert_eq!(body["data"]["description"], "Test transaction");
        assert_eq!(body["data"]["payment_method"], "Credit Card");
        assert_eq!(body["data"]["status"], "Pending");
    }

    #[tokio::test]
    async fn test_create_transaction_invalid_amount() {
        let routes = create_test_routes();

        let response = request()
            .method("POST")
            .path("/transactions")
            .json(&json!({
                "user_id": Uuid::new_v4(),
                "ticket_id": Uuid::new_v4(),
                "amount": 0,
                "description": "Test transaction",
                "payment_method": "Credit Card"
            }))
            .reply(&routes)
            .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        
        let body: Value = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(body["status"], "error");
        assert_eq!(body["message"], "Transaction amount must be positive");
    }

    #[tokio::test]
    async fn test_process_payment() {
        let routes = create_test_routes();

        let create_response = request()
            .method("POST")
            .path("/transactions")
            .json(&json!({
                "user_id": Uuid::new_v4(),
                "ticket_id": Uuid::new_v4(),
                "amount": 1000,
                "description": "Test transaction",
                "payment_method": "Credit Card"
            }))
            .reply(&routes)
            .await;
            
        let create_body: Value = serde_json::from_slice(create_response.body()).unwrap();
        let transaction_id = create_body["data"]["id"].as_str().unwrap();

        let process_response = request()
            .method("PUT")
            .path(&format!("/transactions/{}/process", transaction_id))
            .json(&json!({
                "external_reference": "EXT-REF-123"
            }))
            .reply(&routes)
            .await;

        assert_eq!(process_response.status(), StatusCode::OK);
        
        let body: Value = serde_json::from_slice(process_response.body()).unwrap();
        assert_eq!(body["status"], "success");
        assert_eq!(body["data"]["status"], "Success");
        assert_eq!(body["data"]["external_reference"], "EXT-REF-123");
    }

    #[tokio::test]
    async fn test_validate_payment() {
        let routes = create_test_routes();

        let create_response = request()
            .method("POST")
            .path("/transactions")
            .json(&json!({
                "user_id": Uuid::new_v4(),
                "ticket_id": Uuid::new_v4(),
                "amount": 1000,
                "description": "Test transaction",
                "payment_method": "Credit Card"
            }))
            .reply(&routes)
            .await;
            
        let create_body: Value = serde_json::from_slice(create_response.body()).unwrap();
        let transaction_id = create_body["data"]["id"].as_str().unwrap();

        request()
            .method("PUT")
            .path(&format!("/transactions/{}/process", transaction_id))
            .json(&json!({}))
            .reply(&routes)
            .await;

        let validate_response = request()
            .method("GET")
            .path(&format!("/transactions/{}/validate", transaction_id))
            .reply(&routes)
            .await;

        assert_eq!(validate_response.status(), StatusCode::OK);
        
        let body: Value = serde_json::from_slice(validate_response.body()).unwrap();
        assert_eq!(body["status"], "success");
        assert_eq!(body["data"], true);
    }

    #[tokio::test]
    async fn test_refund_transaction() {
        let routes = create_test_routes();

        let create_response = request()
            .method("POST")
            .path("/transactions")
            .json(&json!({
                "user_id": Uuid::new_v4(),
                "ticket_id": Uuid::new_v4(),
                "amount": 1000,
                "description": "Test transaction",
                "payment_method": "Credit Card"
            }))
            .reply(&routes)
            .await;
            
        let create_body: Value = serde_json::from_slice(create_response.body()).unwrap();
        let transaction_id = create_body["data"]["id"].as_str().unwrap();

        request()
            .method("PUT")
            .path(&format!("/transactions/{}/process", transaction_id))
            .json(&json!({}))
            .reply(&routes)
            .await;

        let refund_response = request()
            .method("PUT")
            .path(&format!("/transactions/{}/refund", transaction_id))
            .reply(&routes)
            .await;

        assert_eq!(refund_response.status(), StatusCode::OK);
        
        let body: Value = serde_json::from_slice(refund_response.body()).unwrap();
        assert_eq!(body["status"], "success");
        assert_eq!(body["data"]["status"], "Refunded");
    }

    #[tokio::test]
    async fn test_get_transaction() {
        let routes = create_test_routes();

        let create_response = request()
            .method("POST")
            .path("/transactions")
            .json(&json!({
                "user_id": Uuid::new_v4(),
                "ticket_id": Uuid::new_v4(),
                "amount": 1000,
                "description": "Test transaction",
                "payment_method": "Credit Card"
            }))
            .reply(&routes)
            .await;
            
        let create_body: Value = serde_json::from_slice(create_response.body()).unwrap();
        let transaction_id = create_body["data"]["id"].as_str().unwrap();

        let get_response = request()
            .method("GET")
            .path(&format!("/transactions/{}", transaction_id))
            .reply(&routes)
            .await;

        assert_eq!(get_response.status(), StatusCode::OK);
        
        let body: Value = serde_json::from_slice(get_response.body()).unwrap();
        assert_eq!(body["status"], "success");
        assert_eq!(body["data"]["id"], transaction_id);
    }

    #[tokio::test]
    async fn test_get_user_transactions() {
        let routes = create_test_routes();

        let user_id = Uuid::new_v4();

        request()
            .method("POST")
            .path("/transactions")
            .json(&json!({
                "user_id": user_id,
                "amount": 1000,
                "description": "Transaction 1",
                "payment_method": "Credit Card"
            }))
            .reply(&routes)
            .await;

        request()
            .method("POST")
            .path("/transactions")
            .json(&json!({
                "user_id": user_id,
                "amount": 2000,
                "description": "Transaction 2",
                "payment_method": "Credit Card"
            }))
            .reply(&routes)
            .await;

        let get_response = request()
            .method("GET")
            .path(&format!("/users/{}/transactions", user_id))
            .reply(&routes)
            .await;

        assert_eq!(get_response.status(), StatusCode::OK);
        
        let body: Value = serde_json::from_slice(get_response.body()).unwrap();
        assert_eq!(body["status"], "success");
        assert!(body["data"].as_array().unwrap().len() == 2);
    }

    #[tokio::test]
    async fn test_add_funds() {
        let routes = create_test_routes();

        let user_id = Uuid::new_v4();
        let amount = 1000;

        let response = request()
            .method("POST")
            .path("/balance/add")
            .json(&json!({
                "user_id": user_id,
                "amount": amount,
                "payment_method": "Credit Card"
            }))
            .reply(&routes)
            .await;

        assert_eq!(response.status(), StatusCode::OK);
        
        let body: Value = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(body["status"], "success");
        assert_eq!(body["data"]["balance"], amount);
        assert_eq!(body["data"]["transaction"]["amount"], amount);
        assert_eq!(body["data"]["transaction"]["status"], "Success");
    }

    #[tokio::test]
    async fn test_withdraw_funds() {
        let routes = create_test_routes();

        let user_id = Uuid::new_v4();
        
        request()
            .method("POST")
            .path("/balance/add")
            .json(&json!({
                "user_id": user_id,
                "amount": 2000,
                "payment_method": "Credit Card"
            }))
            .reply(&routes)
            .await;

        let withdraw_response = request()
            .method("POST")
            .path("/balance/withdraw")
            .json(&json!({
                "user_id": user_id,
                "amount": 1000,
                "description": "Withdrawal test"
            }))
            .reply(&routes)
            .await;

        assert_eq!(withdraw_response.status(), StatusCode::OK);
        
        let body: Value = serde_json::from_slice(withdraw_response.body()).unwrap();
        assert_eq!(body["status"], "success");
        assert_eq!(body["data"]["balance"], 1000);
        assert_eq!(body["data"]["transaction"]["amount"], -1000);
        assert_eq!(body["data"]["transaction"]["status"], "Success");
    }

    #[tokio::test]
    async fn test_withdraw_funds_insufficient() {
        let routes = create_test_routes();

        let user_id = Uuid::new_v4();
        
        request()
            .method("POST")
            .path("/balance/add")
            .json(&json!({
                "user_id": user_id,
                "amount": 500,
                "payment_method": "Credit Card"
            }))
            .reply(&routes)
            .await;

        let withdraw_response = request()
            .method("POST")
            .path("/balance/withdraw")
            .json(&json!({
                "user_id": user_id,
                "amount": 1000,
                "description": "Withdrawal test"
            }))
            .reply(&routes)
            .await;

        assert_eq!(withdraw_response.status(), StatusCode::BAD_REQUEST);
        
        let body: Value = serde_json::from_slice(withdraw_response.body()).unwrap();
        assert_eq!(body["status"], "error");
        assert_eq!(body["message"], "Insufficient funds");
    }

    #[tokio::test]
    async fn test_delete_transaction() {
        let routes = create_test_routes();

        let create_response = request()
            .method("POST")
            .path("/transactions")
            .json(&json!({
                "user_id": Uuid::new_v4(),
                "amount": 1000,
                "description": "Test transaction",
                "payment_method": "Credit Card"
            }))
            .reply(&routes)
            .await;
            
        let create_body: Value = serde_json::from_slice(create_response.body()).unwrap();
        let transaction_id = create_body["data"]["id"].as_str().unwrap();

        let delete_response = request()
            .method("DELETE")
            .path(&format!("/transactions/{}", transaction_id))
            .reply(&routes)
            .await;

        assert_eq!(delete_response.status(), StatusCode::OK);
        
        let get_response = request()
            .method("GET")
            .path(&format!("/transactions/{}", transaction_id))
            .reply(&routes)
            .await;
            
        let get_body: Value = serde_json::from_slice(get_response.body()).unwrap();
        assert_eq!(get_body["data"], Value::Null);
    }
}
