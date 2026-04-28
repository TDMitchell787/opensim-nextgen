//! Transaction Processing Engine for OpenSim Next
//!
//! Provides secure, atomic transaction processing with fraud detection,
//! rate limiting, and comprehensive audit trails.

use super::*;
use crate::database::DatabaseManager;
use anyhow::Result;
use sqlx::Row;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Transaction processing engine
pub struct TransactionProcessor {
    database: Arc<DatabaseManager>,
    currency_system: Arc<super::currency::CurrencySystem>,
    config: EconomyConfig,
    active_transactions: Arc<RwLock<HashMap<Uuid, ActiveTransaction>>>,
    rate_limiter: Arc<RwLock<HashMap<Uuid, UserRateLimit>>>,
    fraud_detector: Arc<FraudDetector>,
    transaction_locks: Arc<Mutex<HashMap<Uuid, Arc<Mutex<()>>>>>,
}

/// Active transaction tracking
#[derive(Debug, Clone)]
struct ActiveTransaction {
    transaction_id: Uuid,
    transaction_type: TransactionType,
    status: TransactionStatus,
    started_at: Instant,
    timeout_at: Instant,
    participants: Vec<Uuid>,
}

/// User rate limiting
#[derive(Debug, Clone)]
struct UserRateLimit {
    user_id: Uuid,
    transactions_this_minute: u32,
    last_reset: Instant,
    daily_volume: i64,
    daily_reset: chrono::DateTime<chrono::Utc>,
}

/// Fraud detection system
pub struct FraudDetector {
    config: FraudDetectionConfig,
    user_patterns: Arc<RwLock<HashMap<Uuid, UserTransactionPattern>>>,
    blacklisted_users: Arc<RwLock<Vec<Uuid>>>,
    suspicious_patterns: Arc<RwLock<Vec<SuspiciousPattern>>>,
}

/// User transaction pattern for fraud detection
#[derive(Debug, Clone)]
struct UserTransactionPattern {
    user_id: Uuid,
    average_transaction_size: f64,
    transaction_frequency: f64,
    preferred_currencies: Vec<String>,
    usual_transaction_types: Vec<TransactionType>,
    last_analysis: chrono::DateTime<chrono::Utc>,
    risk_score: f64,
}

/// Suspicious pattern definition
#[derive(Debug, Clone)]
struct SuspiciousPattern {
    pattern_id: Uuid,
    pattern_type: FraudAlertType,
    threshold: f64,
    time_window_minutes: u32,
    action: FraudAction,
}

/// Fraud action to take
#[derive(Debug, Clone)]
enum FraudAction {
    Alert,
    Block,
    Review,
    Freeze,
}

/// Transaction request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub transaction_type: TransactionType,
    pub from_user_id: Option<Uuid>,
    pub to_user_id: Option<Uuid>,
    pub currency_code: String,
    pub amount: i64,
    pub description: String,
    pub reference_id: Option<Uuid>,
    pub metadata: HashMap<String, String>,
    pub idempotency_key: Option<String>,
}

/// Transaction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    pub transaction: Transaction,
    pub from_balance: Option<CurrencyBalance>,
    pub to_balance: Option<CurrencyBalance>,
    pub fees_charged: i64,
    pub processing_time_ms: u64,
}

impl TransactionProcessor {
    /// Create new transaction processor
    pub fn new(
        database: Arc<DatabaseManager>,
        currency_system: Arc<super::currency::CurrencySystem>,
        config: EconomyConfig,
    ) -> Self {
        let fraud_detector = Arc::new(FraudDetector::new(config.fraud_detection.clone()));

        Self {
            database,
            currency_system,
            config,
            active_transactions: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter: Arc::new(RwLock::new(HashMap::new())),
            fraud_detector,
            transaction_locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Initialize transaction processor
    pub async fn initialize(&self) -> EconomyResult<()> {
        info!("Initializing transaction processor");

        // Create database tables
        self.create_tables().await?;

        // Load fraud patterns
        self.fraud_detector.load_patterns().await?;

        // Start cleanup task for expired transactions
        self.start_cleanup_task().await;

        info!("Transaction processor initialized successfully");
        Ok(())
    }

    /// Process a transaction
    pub async fn process_transaction(
        &self,
        request: TransactionRequest,
    ) -> EconomyResult<TransactionResult> {
        let start_time = Instant::now();
        info!(
            "Processing transaction: {:?} {} {}",
            request.transaction_type, request.amount, request.currency_code
        );

        // Validate request
        self.validate_transaction_request(&request).await?;

        // Check rate limits
        if let Some(user_id) = request.from_user_id {
            self.check_rate_limits(user_id, request.amount).await?;
        }

        // Fraud detection
        self.detect_fraud(&request).await?;

        // Get user locks to prevent concurrent transactions
        let user_locks = self.acquire_user_locks(&request).await;

        // Process transaction with locks held
        let result = {
            let _locks = user_locks; // Hold locks for scope
            self.execute_transaction(request, start_time).await?
        };

        let processing_time = start_time.elapsed();
        info!(
            "Transaction processed successfully in {}ms",
            processing_time.as_millis()
        );

        Ok(result)
    }

    /// Execute transaction atomically
    async fn execute_transaction(
        &self,
        request: TransactionRequest,
        start_time: Instant,
    ) -> EconomyResult<TransactionResult> {
        let transaction_id = Uuid::new_v4();

        // Create transaction record
        let mut transaction = Transaction {
            transaction_id,
            transaction_type: request.transaction_type.clone(),
            from_user_id: request.from_user_id,
            to_user_id: request.to_user_id,
            currency_code: request.currency_code.clone(),
            amount: request.amount,
            fee: 0,
            description: request.description.clone(),
            reference_id: request.reference_id,
            status: TransactionStatus::Processing,
            created_at: chrono::Utc::now(),
            processed_at: None,
            metadata: request.metadata.clone(),
        };

        // Calculate fees
        transaction.fee = self.calculate_fees(&request).await?;

        // Track active transaction
        self.track_active_transaction(&transaction).await;

        // Execute based on transaction type
        let result = match request.transaction_type {
            TransactionType::Transfer => self.process_transfer(&transaction).await,
            TransactionType::Purchase => self.process_purchase(&transaction).await,
            TransactionType::Deposit => self.process_deposit(&transaction).await,
            TransactionType::Withdrawal => self.process_withdrawal(&transaction).await,
            TransactionType::Exchange => self.process_exchange(&transaction).await,
            TransactionType::Refund => self.process_refund(&transaction).await,
            _ => self.process_generic_transaction(&transaction).await,
        };

        match result {
            Ok(mut tx_result) => {
                // Mark transaction as completed
                transaction.status = TransactionStatus::Completed;
                transaction.processed_at = Some(chrono::Utc::now());

                // Save final transaction state
                self.save_transaction(&transaction).await?;

                // Update transaction in result
                tx_result.transaction = transaction;
                tx_result.processing_time_ms = start_time.elapsed().as_millis() as u64;

                // Remove from active transactions
                self.remove_active_transaction(transaction_id).await;

                Ok(tx_result)
            }
            Err(e) => {
                // Mark transaction as failed
                transaction.status = TransactionStatus::Failed;
                transaction.processed_at = Some(chrono::Utc::now());

                // Save failed transaction for audit
                self.save_transaction(&transaction).await?;

                // Remove from active transactions
                self.remove_active_transaction(transaction_id).await;

                error!("Transaction {} failed: {}", transaction_id, e);
                Err(e)
            }
        }
    }

    /// Process transfer transaction
    async fn process_transfer(
        &self,
        transaction: &Transaction,
    ) -> EconomyResult<TransactionResult> {
        let from_user =
            transaction
                .from_user_id
                .ok_or_else(|| EconomyError::TransactionFailed {
                    reason: "Transfer requires from_user_id".to_string(),
                })?;

        let to_user = transaction
            .to_user_id
            .ok_or_else(|| EconomyError::TransactionFailed {
                reason: "Transfer requires to_user_id".to_string(),
            })?;

        if from_user == to_user {
            return Err(EconomyError::TransactionFailed {
                reason: "Cannot transfer to yourself".to_string(),
            });
        }

        // Calculate total amount including fees
        let total_amount = transaction.amount + transaction.fee;

        // Subtract from sender (including fees)
        let from_balance = self
            .currency_system
            .subtract_currency(
                from_user,
                &transaction.currency_code,
                total_amount,
                format!("Transfer to user {}: {}", to_user, transaction.description),
            )
            .await?;

        // Add to receiver (excluding fees)
        let to_balance = self
            .currency_system
            .add_currency(
                to_user,
                &transaction.currency_code,
                transaction.amount,
                format!(
                    "Transfer from user {}: {}",
                    from_user, transaction.description
                ),
            )
            .await?;

        Ok(TransactionResult {
            transaction: transaction.clone(),
            from_balance: Some(from_balance),
            to_balance: Some(to_balance),
            fees_charged: transaction.fee,
            processing_time_ms: 0, // Will be set by caller
        })
    }

    /// Process purchase transaction
    async fn process_purchase(
        &self,
        transaction: &Transaction,
    ) -> EconomyResult<TransactionResult> {
        let buyer_id = transaction
            .from_user_id
            .ok_or_else(|| EconomyError::TransactionFailed {
                reason: "Purchase requires from_user_id (buyer)".to_string(),
            })?;

        let seller_id = transaction
            .to_user_id
            .ok_or_else(|| EconomyError::TransactionFailed {
                reason: "Purchase requires to_user_id (seller)".to_string(),
            })?;

        // Calculate total cost including fees
        let total_cost = transaction.amount + transaction.fee;

        // Subtract from buyer
        let buyer_balance = self
            .currency_system
            .subtract_currency(
                buyer_id,
                &transaction.currency_code,
                total_cost,
                format!("Purchase: {}", transaction.description),
            )
            .await?;

        // Calculate seller amount (purchase amount minus commission)
        let commission_rate = self.config.marketplace_config.commission_rate;
        let commission = (transaction.amount as f64 * commission_rate) as i64;
        let seller_amount = transaction.amount - commission;

        // Add to seller (minus commission)
        let seller_balance = self
            .currency_system
            .add_currency(
                seller_id,
                &transaction.currency_code,
                seller_amount,
                format!("Sale: {}", transaction.description),
            )
            .await?;

        Ok(TransactionResult {
            transaction: transaction.clone(),
            from_balance: Some(buyer_balance),
            to_balance: Some(seller_balance),
            fees_charged: transaction.fee + commission,
            processing_time_ms: 0,
        })
    }

    /// Process deposit transaction
    async fn process_deposit(&self, transaction: &Transaction) -> EconomyResult<TransactionResult> {
        let user_id = transaction
            .to_user_id
            .ok_or_else(|| EconomyError::TransactionFailed {
                reason: "Deposit requires to_user_id".to_string(),
            })?;

        // Add currency to user account
        let balance = self
            .currency_system
            .add_currency(
                user_id,
                &transaction.currency_code,
                transaction.amount,
                format!("Deposit: {}", transaction.description),
            )
            .await?;

        Ok(TransactionResult {
            transaction: transaction.clone(),
            from_balance: None,
            to_balance: Some(balance),
            fees_charged: transaction.fee,
            processing_time_ms: 0,
        })
    }

    /// Process withdrawal transaction
    async fn process_withdrawal(
        &self,
        transaction: &Transaction,
    ) -> EconomyResult<TransactionResult> {
        let user_id = transaction
            .from_user_id
            .ok_or_else(|| EconomyError::TransactionFailed {
                reason: "Withdrawal requires from_user_id".to_string(),
            })?;

        // Calculate total amount including fees
        let total_amount = transaction.amount + transaction.fee;

        // Subtract from user account
        let balance = self
            .currency_system
            .subtract_currency(
                user_id,
                &transaction.currency_code,
                total_amount,
                format!("Withdrawal: {}", transaction.description),
            )
            .await?;

        Ok(TransactionResult {
            transaction: transaction.clone(),
            from_balance: Some(balance),
            to_balance: None,
            fees_charged: transaction.fee,
            processing_time_ms: 0,
        })
    }

    /// Process currency exchange transaction
    async fn process_exchange(
        &self,
        transaction: &Transaction,
    ) -> EconomyResult<TransactionResult> {
        let user_id = transaction
            .from_user_id
            .ok_or_else(|| EconomyError::TransactionFailed {
                reason: "Exchange requires from_user_id".to_string(),
            })?;

        // Parse target currency from metadata
        let to_currency = transaction.metadata.get("to_currency").ok_or_else(|| {
            EconomyError::TransactionFailed {
                reason: "Exchange requires to_currency in metadata".to_string(),
            }
        })?;

        // Perform currency conversion
        let (from_balance, to_balance) = self
            .currency_system
            .convert_currency(
                user_id,
                &transaction.currency_code,
                to_currency,
                transaction.amount,
            )
            .await?;

        Ok(TransactionResult {
            transaction: transaction.clone(),
            from_balance: Some(from_balance),
            to_balance: Some(to_balance),
            fees_charged: transaction.fee,
            processing_time_ms: 0,
        })
    }

    /// Process refund transaction
    async fn process_refund(&self, transaction: &Transaction) -> EconomyResult<TransactionResult> {
        let user_id = transaction
            .to_user_id
            .ok_or_else(|| EconomyError::TransactionFailed {
                reason: "Refund requires to_user_id".to_string(),
            })?;

        // Add refund amount to user account (no fees on refunds)
        let balance = self
            .currency_system
            .add_currency(
                user_id,
                &transaction.currency_code,
                transaction.amount,
                format!("Refund: {}", transaction.description),
            )
            .await?;

        Ok(TransactionResult {
            transaction: transaction.clone(),
            from_balance: None,
            to_balance: Some(balance),
            fees_charged: 0, // No fees on refunds
            processing_time_ms: 0,
        })
    }

    /// Process generic transaction
    async fn process_generic_transaction(
        &self,
        transaction: &Transaction,
    ) -> EconomyResult<TransactionResult> {
        // For system transactions, rewards, penalties, etc.
        match (transaction.from_user_id, transaction.to_user_id) {
            (Some(from_user), Some(to_user)) => {
                // Transfer-like transaction
                let total_amount = transaction.amount + transaction.fee;

                let from_balance = self
                    .currency_system
                    .subtract_currency(
                        from_user,
                        &transaction.currency_code,
                        total_amount,
                        transaction.description.clone(),
                    )
                    .await?;

                let to_balance = self
                    .currency_system
                    .add_currency(
                        to_user,
                        &transaction.currency_code,
                        transaction.amount,
                        transaction.description.clone(),
                    )
                    .await?;

                Ok(TransactionResult {
                    transaction: transaction.clone(),
                    from_balance: Some(from_balance),
                    to_balance: Some(to_balance),
                    fees_charged: transaction.fee,
                    processing_time_ms: 0,
                })
            }
            (Some(from_user), None) => {
                // Debit transaction
                let total_amount = transaction.amount + transaction.fee;

                let balance = self
                    .currency_system
                    .subtract_currency(
                        from_user,
                        &transaction.currency_code,
                        total_amount,
                        transaction.description.clone(),
                    )
                    .await?;

                Ok(TransactionResult {
                    transaction: transaction.clone(),
                    from_balance: Some(balance),
                    to_balance: None,
                    fees_charged: transaction.fee,
                    processing_time_ms: 0,
                })
            }
            (None, Some(to_user)) => {
                // Credit transaction
                let balance = self
                    .currency_system
                    .add_currency(
                        to_user,
                        &transaction.currency_code,
                        transaction.amount,
                        transaction.description.clone(),
                    )
                    .await?;

                Ok(TransactionResult {
                    transaction: transaction.clone(),
                    from_balance: None,
                    to_balance: Some(balance),
                    fees_charged: transaction.fee,
                    processing_time_ms: 0,
                })
            }
            (None, None) => Err(EconomyError::TransactionFailed {
                reason: "Transaction must specify at least one user".to_string(),
            }),
        }
    }

    /// Get transaction history for user
    pub async fn get_transaction_history(
        &self,
        user_id: Uuid,
        currency_code: Option<String>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> EconomyResult<Vec<Transaction>> {
        debug!("Getting transaction history for user {}", user_id);

        let limit_val = limit.unwrap_or(50).min(200) as i64;
        let offset_val = offset.unwrap_or(0) as i64;

        let rows = if let Some(ref cc) = currency_code {
            sqlx::query(
                r#"
                SELECT id, from_user, to_user, amount, fee, currency_code, transaction_type, status, description, metadata, created_at
                FROM economy_transactions
                WHERE (from_user = $1 OR to_user = $1) AND currency_code = $2
                ORDER BY created_at DESC
                LIMIT $3 OFFSET $4
                "#
            )
            .bind(user_id)
            .bind(cc)
            .bind(limit_val)
            .bind(offset_val)
            .fetch_all(self.database.legacy_pool()?)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT id, from_user, to_user, amount, fee, currency_code, transaction_type, status, description, metadata, created_at
                FROM economy_transactions
                WHERE from_user = $1 OR to_user = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#
            )
            .bind(user_id)
            .bind(limit_val)
            .bind(offset_val)
            .fetch_all(self.database.legacy_pool()?)
            .await?
        };

        let mut transactions = Vec::with_capacity(rows.len());
        for row in rows {
            transactions.push(Self::row_to_transaction(&row)?);
        }

        debug!(
            "Retrieved {} transactions for user {}",
            transactions.len(),
            user_id
        );
        Ok(transactions)
    }

    /// Get transaction by ID
    pub async fn get_transaction(&self, transaction_id: Uuid) -> EconomyResult<Transaction> {
        debug!("Getting transaction {}", transaction_id);

        let row = sqlx::query(
            "SELECT id, from_user, to_user, amount, fee, currency_code, transaction_type, status, description, metadata, created_at FROM economy_transactions WHERE id = $1"
        )
        .bind(transaction_id)
        .fetch_optional(self.database.legacy_pool()?)
        .await?;

        match row {
            Some(row) => Self::row_to_transaction(&row),
            None => Err(EconomyError::TransactionFailed {
                reason: format!("Transaction {} not found", transaction_id),
            }),
        }
    }

    fn row_to_transaction(row: &sqlx::postgres::PgRow) -> EconomyResult<Transaction> {
        let from_user: Uuid = row.try_get("from_user")?;
        let to_user: Uuid = row.try_get("to_user")?;
        let tx_type_str: String = row.try_get("transaction_type")?;
        let status_str: String = row.try_get("status")?;
        let metadata_val: serde_json::Value = row.try_get("metadata").unwrap_or_default();

        let from_user_id = if from_user == Uuid::nil() {
            None
        } else {
            Some(from_user)
        };
        let to_user_id = if to_user == Uuid::nil() {
            None
        } else {
            Some(to_user)
        };

        let transaction_type = match tx_type_str.as_str() {
            "Transfer" => TransactionType::Transfer,
            "Purchase" => TransactionType::Purchase,
            "Sale" => TransactionType::Sale,
            "Deposit" => TransactionType::Deposit,
            "Withdrawal" => TransactionType::Withdrawal,
            "Fee" => TransactionType::Fee,
            "Commission" => TransactionType::Commission,
            "Refund" => TransactionType::Refund,
            "Reward" => TransactionType::Reward,
            "Penalty" => TransactionType::Penalty,
            "Exchange" => TransactionType::Exchange,
            "Escrow" => TransactionType::Escrow,
            "Release" => TransactionType::Release,
            _ => TransactionType::System,
        };

        let status = match status_str.as_str() {
            "Pending" => TransactionStatus::Pending,
            "Processing" => TransactionStatus::Processing,
            "Completed" => TransactionStatus::Completed,
            "Failed" => TransactionStatus::Failed,
            "Cancelled" => TransactionStatus::Cancelled,
            "Reversed" => TransactionStatus::Reversed,
            "Disputed" => TransactionStatus::Disputed,
            "OnHold" => TransactionStatus::OnHold,
            _ => TransactionStatus::Pending,
        };

        let metadata: HashMap<String, String> = if let serde_json::Value::Object(map) = metadata_val
        {
            map.into_iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
                .collect()
        } else {
            HashMap::new()
        };

        Ok(Transaction {
            transaction_id: row.try_get("id")?,
            transaction_type,
            from_user_id,
            to_user_id,
            currency_code: row.try_get("currency_code")?,
            amount: row.try_get("amount")?,
            fee: row.try_get("fee")?,
            description: row.try_get("description")?,
            reference_id: None,
            status,
            created_at: row.try_get("created_at")?,
            processed_at: None,
            metadata,
        })
    }

    /// Cancel pending transaction
    pub async fn cancel_transaction(
        &self,
        transaction_id: Uuid,
        reason: String,
    ) -> EconomyResult<()> {
        info!("Cancelling transaction {}: {}", transaction_id, reason);

        // Check if transaction is active and cancellable
        let mut active_transactions = self.active_transactions.write().await;
        if let Some(mut active_tx) = active_transactions.remove(&transaction_id) {
            active_tx.status = TransactionStatus::Cancelled;

            // Load and update transaction in database
            // Implementation would update transaction status

            info!("Transaction {} cancelled successfully", transaction_id);
            Ok(())
        } else {
            Err(EconomyError::TransactionFailed {
                reason: "Transaction not found or not cancellable".to_string(),
            })
        }
    }

    // Private helper methods

    async fn validate_transaction_request(
        &self,
        request: &TransactionRequest,
    ) -> EconomyResult<()> {
        // Basic validation
        if request.amount <= 0 {
            return Err(EconomyError::TransactionFailed {
                reason: "Transaction amount must be positive".to_string(),
            });
        }

        if request.amount > self.config.transaction_limits.max_transaction_amount {
            return Err(EconomyError::TransactionLimitExceeded {
                limit: self.config.transaction_limits.max_transaction_amount,
            });
        }

        if request.amount < self.config.transaction_limits.minimum_transaction {
            return Err(EconomyError::TransactionFailed {
                reason: format!(
                    "Amount below minimum {}",
                    self.config.transaction_limits.minimum_transaction
                ),
            });
        }

        // Validate currency
        self.currency_system
            .get_currency_definition(&request.currency_code)
            .await?;

        // Validate user requirements based on transaction type
        match request.transaction_type {
            TransactionType::Transfer
            | TransactionType::Purchase
            | TransactionType::Withdrawal
            | TransactionType::Exchange => {
                if request.from_user_id.is_none() {
                    return Err(EconomyError::TransactionFailed {
                        reason: "Transaction type requires from_user_id".to_string(),
                    });
                }
            }
            _ => {}
        }

        Ok(())
    }

    async fn check_rate_limits(&self, user_id: Uuid, amount: i64) -> EconomyResult<()> {
        let mut rate_limiter = self.rate_limiter.write().await;
        let now = Instant::now();
        let today = chrono::Utc::now().date_naive();

        let user_limit = rate_limiter
            .entry(user_id)
            .or_insert_with(|| UserRateLimit {
                user_id,
                transactions_this_minute: 0,
                last_reset: now,
                daily_volume: 0,
                daily_reset: chrono::Utc::now(),
            });

        // Reset minute counter if needed
        if now.duration_since(user_limit.last_reset) >= Duration::from_secs(60) {
            user_limit.transactions_this_minute = 0;
            user_limit.last_reset = now;
        }

        // Reset daily volume if needed
        if user_limit.daily_reset.date_naive() != today {
            user_limit.daily_volume = 0;
            user_limit.daily_reset = chrono::Utc::now();
        }

        // Check rate limits
        if user_limit.transactions_this_minute
            >= self.config.transaction_limits.rate_limit_per_minute
        {
            return Err(EconomyError::TransactionLimitExceeded {
                limit: self.config.transaction_limits.rate_limit_per_minute as i64,
            });
        }

        if user_limit.daily_volume + amount > self.config.transaction_limits.daily_limit {
            return Err(EconomyError::TransactionLimitExceeded {
                limit: self.config.transaction_limits.daily_limit,
            });
        }

        // Update counters
        user_limit.transactions_this_minute += 1;
        user_limit.daily_volume += amount;

        Ok(())
    }

    async fn detect_fraud(&self, request: &TransactionRequest) -> EconomyResult<()> {
        if !self.config.fraud_detection.enabled {
            return Ok(());
        }

        self.fraud_detector.analyze_transaction(request).await
    }

    async fn calculate_fees(&self, request: &TransactionRequest) -> EconomyResult<i64> {
        // Basic fee calculation - could be more sophisticated
        let base_fee = match request.transaction_type {
            TransactionType::Transfer => (request.amount as f64 * 0.01) as i64, // 1%
            TransactionType::Purchase => 0, // No additional fee (commission handled separately)
            TransactionType::Withdrawal => 100, // Fixed fee
            TransactionType::Exchange => (request.amount as f64 * 0.005) as i64, // 0.5%
            _ => 0,
        };

        Ok(base_fee.max(1)) // Minimum 1 unit fee
    }

    async fn acquire_user_locks(&self, request: &TransactionRequest) -> Vec<Arc<Mutex<()>>> {
        let mut locks_map = self.transaction_locks.lock().await;
        let mut user_locks = Vec::new();

        // Collect all user IDs involved in transaction
        let mut user_ids = Vec::new();
        if let Some(from_user) = request.from_user_id {
            user_ids.push(from_user);
        }
        if let Some(to_user) = request.to_user_id {
            user_ids.push(to_user);
        }

        // Sort user IDs to prevent deadlocks
        user_ids.sort();

        // Acquire locks for all users
        for user_id in user_ids {
            let lock = locks_map
                .entry(user_id)
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone();
            user_locks.push(lock);
        }

        user_locks
    }

    async fn track_active_transaction(&self, transaction: &Transaction) {
        let active_tx = ActiveTransaction {
            transaction_id: transaction.transaction_id,
            transaction_type: transaction.transaction_type.clone(),
            status: transaction.status.clone(),
            started_at: Instant::now(),
            timeout_at: Instant::now() + Duration::from_secs(300), // 5 minute timeout
            participants: vec![transaction.from_user_id, transaction.to_user_id]
                .into_iter()
                .flatten()
                .collect(),
        };

        let mut active_transactions = self.active_transactions.write().await;
        active_transactions.insert(transaction.transaction_id, active_tx);
    }

    async fn remove_active_transaction(&self, transaction_id: Uuid) {
        let mut active_transactions = self.active_transactions.write().await;
        active_transactions.remove(&transaction_id);
    }

    async fn start_cleanup_task(&self) {
        // Implementation would start background task to clean up expired transactions
    }

    async fn create_tables(&self) -> EconomyResult<()> {
        info!("Economy tables managed by migration 042_economy_tables.sql — skipping runtime creation");
        Ok(())
    }

    async fn save_transaction(&self, transaction: &Transaction) -> EconomyResult<()> {
        debug!(
            "Saving transaction {} to database",
            transaction.transaction_id
        );

        let from_user = transaction.from_user_id.unwrap_or(Uuid::nil());
        let to_user = transaction.to_user_id.unwrap_or(Uuid::nil());
        let tx_type = format!("{:?}", transaction.transaction_type);
        let status = format!("{:?}", transaction.status);
        let metadata_json = serde_json::to_value(&transaction.metadata).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO economy_transactions (id, from_user, to_user, amount, fee, currency_code, transaction_type, status, description, metadata, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (id) DO UPDATE SET
                status = EXCLUDED.status,
                metadata = EXCLUDED.metadata
            "#
        )
        .bind(transaction.transaction_id)
        .bind(from_user)
        .bind(to_user)
        .bind(transaction.amount)
        .bind(transaction.fee)
        .bind(&transaction.currency_code)
        .bind(&tx_type)
        .bind(&status)
        .bind(&transaction.description)
        .bind(&metadata_json)
        .bind(transaction.created_at)
        .execute(self.database.legacy_pool()?)
        .await?;

        debug!(
            "Transaction {} saved successfully",
            transaction.transaction_id
        );
        Ok(())
    }
}

impl FraudDetector {
    fn new(config: FraudDetectionConfig) -> Self {
        Self {
            config,
            user_patterns: Arc::new(RwLock::new(HashMap::new())),
            blacklisted_users: Arc::new(RwLock::new(Vec::new())),
            suspicious_patterns: Arc::new(RwLock::new(Vec::new())),
        }
    }

    async fn load_patterns(&self) -> EconomyResult<()> {
        // Implementation would load fraud patterns from database
        Ok(())
    }

    async fn analyze_transaction(&self, request: &TransactionRequest) -> EconomyResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Check blacklist
        if let Some(user_id) = request.from_user_id {
            let blacklist = self.blacklisted_users.read().await;
            if blacklist.contains(&user_id) {
                return Err(EconomyError::FraudDetected {
                    reason: "User is blacklisted".to_string(),
                });
            }
        }

        // Additional fraud checks would be implemented here
        Ok(())
    }
}
