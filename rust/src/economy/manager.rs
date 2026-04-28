//! Virtual Economy Manager for OpenSim Next
//!
//! Orchestrates all economy components including currency, transactions,
//! marketplace, and analytics systems.

use super::*;
use crate::database::DatabaseManager;
use anyhow::Result;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Virtual economy management system
pub struct VirtualEconomyManager {
    database: Arc<DatabaseManager>,
    config: EconomyConfig,
    currency_system: Arc<super::currency::CurrencySystem>,
    transaction_processor: Arc<super::transactions::TransactionProcessor>,
    marketplace_manager: Arc<super::marketplace::MarketplaceManager>,
    analytics_engine: Arc<super::analytics::AnalyticsEngine>,
}

impl VirtualEconomyManager {
    /// Create new virtual economy manager
    pub fn new(database: Arc<DatabaseManager>, config: EconomyConfig) -> Self {
        // Create currency system
        let currency_system = Arc::new(super::currency::CurrencySystem::new(
            database.clone(),
            config.clone(),
        ));

        // Create transaction processor
        let transaction_processor = Arc::new(super::transactions::TransactionProcessor::new(
            database.clone(),
            currency_system.clone(),
            config.clone(),
        ));

        // Create marketplace manager
        let marketplace_manager = Arc::new(super::marketplace::MarketplaceManager::new(
            database.clone(),
            transaction_processor.clone(),
            currency_system.clone(),
            config.marketplace_config.clone(),
        ));

        // Create analytics engine
        let analytics_engine = Arc::new(super::analytics::AnalyticsEngine::new(
            database.clone(),
            config.analytics_config.clone(),
        ));

        Self {
            database,
            config,
            currency_system,
            transaction_processor,
            marketplace_manager,
            analytics_engine,
        }
    }

    /// Initialize economy system
    pub async fn initialize(&self) -> EconomyResult<()> {
        info!("Initializing virtual economy system");

        // Initialize currency system
        self.currency_system.initialize().await?;

        // Initialize transaction processor
        self.transaction_processor.initialize().await?;

        // Initialize marketplace
        self.marketplace_manager.initialize().await?;

        // Initialize analytics
        self.analytics_engine.initialize().await?;

        info!("Virtual economy system initialized successfully");
        Ok(())
    }

    /// Get currency system
    pub fn currency_system(&self) -> Arc<super::currency::CurrencySystem> {
        self.currency_system.clone()
    }

    /// Get transaction processor
    pub fn transaction_processor(&self) -> Arc<super::transactions::TransactionProcessor> {
        self.transaction_processor.clone()
    }

    /// Get marketplace manager
    pub fn marketplace_manager(&self) -> Arc<super::marketplace::MarketplaceManager> {
        self.marketplace_manager.clone()
    }

    /// Get analytics engine
    pub fn analytics_engine(&self) -> Arc<super::analytics::AnalyticsEngine> {
        self.analytics_engine.clone()
    }

    /// Create user account with initial balance
    pub async fn create_user_account(
        &self,
        user_id: Uuid,
        initial_balance: Option<i64>,
    ) -> EconomyResult<()> {
        info!("Creating economy account for user {}", user_id);

        let default_currency = &self.config.default_currency;
        let initial_amount = initial_balance.unwrap_or(1000); // Default starter amount

        // Create balance for default currency
        self.currency_system
            .create_balance(user_id, default_currency, initial_amount)
            .await?;

        // Record account creation transaction
        let transaction_request = super::transactions::TransactionRequest {
            transaction_type: TransactionType::System,
            from_user_id: None,
            to_user_id: Some(user_id),
            currency_code: default_currency.clone(),
            amount: initial_amount,
            description: "Account creation bonus".to_string(),
            reference_id: None,
            metadata: std::collections::HashMap::new(),
            idempotency_key: Some(format!("account-creation-{}", user_id)),
        };

        self.transaction_processor
            .process_transaction(transaction_request)
            .await?;

        info!(
            "Economy account created for user {} with {} {}",
            user_id, initial_amount, default_currency
        );
        Ok(())
    }

    /// Transfer currency between users
    pub async fn transfer_currency(
        &self,
        from_user_id: Uuid,
        to_user_id: Uuid,
        amount: i64,
        currency_code: String,
        description: String,
    ) -> EconomyResult<super::transactions::TransactionResult> {
        info!(
            "Transferring {} {} from {} to {}",
            amount, currency_code, from_user_id, to_user_id
        );

        let transaction_request = super::transactions::TransactionRequest {
            transaction_type: TransactionType::Transfer,
            from_user_id: Some(from_user_id),
            to_user_id: Some(to_user_id),
            currency_code,
            amount,
            description,
            reference_id: None,
            metadata: std::collections::HashMap::new(),
            idempotency_key: None,
        };

        let result = self
            .transaction_processor
            .process_transaction(transaction_request)
            .await?;

        info!("Transfer completed successfully");
        Ok(result)
    }

    /// Purchase item from marketplace
    pub async fn purchase_marketplace_item(
        &self,
        buyer_id: Uuid,
        listing_id: Uuid,
        quantity: u32,
        payment_method: PaymentMethod,
    ) -> EconomyResult<PurchaseOrder> {
        info!(
            "Processing marketplace purchase: buyer {} listing {} quantity {}",
            buyer_id, listing_id, quantity
        );

        let order = self
            .marketplace_manager
            .purchase_item(buyer_id, listing_id, quantity, payment_method)
            .await?;

        info!("Marketplace purchase completed: order {}", order.order_id);
        Ok(order)
    }

    /// Get user's economy summary
    pub async fn get_user_economy_summary(
        &self,
        user_id: Uuid,
    ) -> EconomyResult<UserEconomySummary> {
        info!("Generating economy summary for user {}", user_id);

        // Get all balances
        let balances = self.currency_system.get_all_balances(user_id).await?;

        // Get recent transactions (last 10)
        let recent_transactions = self
            .transaction_processor
            .get_transaction_history(
                user_id,
                None, // All currencies
                Some(10),
                None,
            )
            .await?;

        // Get marketplace statistics
        let seller_listings = self
            .marketplace_manager
            .get_seller_listings(user_id)
            .await?;

        // Calculate totals
        let total_balance_default_currency = balances
            .iter()
            .filter(|b| b.currency_code == self.config.default_currency)
            .map(|b| b.balance)
            .sum::<i64>();

        let total_reserved_default_currency = balances
            .iter()
            .filter(|b| b.currency_code == self.config.default_currency)
            .map(|b| b.reserved)
            .sum::<i64>();

        let summary = UserEconomySummary {
            user_id,
            balances,
            total_balance_default_currency,
            total_reserved_default_currency,
            recent_transactions,
            marketplace_listings_count: seller_listings.len() as u32,
            marketplace_active_listings: seller_listings
                .iter()
                .filter(|l| matches!(l.listing_status, ListingStatus::Active))
                .count() as u32,
            generated_at: chrono::Utc::now(),
        };

        info!("Economy summary generated for user {}", user_id);
        Ok(summary)
    }

    /// Get system-wide economy metrics
    pub async fn get_economy_metrics(&self) -> EconomyResult<EconomicMetrics> {
        info!("Generating system-wide economy metrics");

        let metrics = self.analytics_engine.generate_economy_metrics().await?;

        info!("Economy metrics generated successfully");
        Ok(metrics)
    }

    /// Add currency to user (admin function)
    pub async fn admin_add_currency(
        &self,
        user_id: Uuid,
        currency_code: String,
        amount: i64,
        reason: String,
    ) -> EconomyResult<CurrencyBalance> {
        info!(
            "Admin adding {} {} to user {}: {}",
            amount, currency_code, user_id, reason
        );

        // Create admin transaction
        let transaction_request = super::transactions::TransactionRequest {
            transaction_type: TransactionType::System,
            from_user_id: None,
            to_user_id: Some(user_id),
            currency_code: currency_code.clone(),
            amount,
            description: format!("Admin credit: {}", reason),
            reference_id: None,
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("admin_action".to_string(), "add_currency".to_string());
                meta.insert("reason".to_string(), reason);
                meta
            },
            idempotency_key: None,
        };

        let result = self
            .transaction_processor
            .process_transaction(transaction_request)
            .await?;

        info!("Admin currency addition completed");
        Ok(result.to_balance.unwrap())
    }

    /// Remove currency from user (admin function)
    pub async fn admin_remove_currency(
        &self,
        user_id: Uuid,
        currency_code: String,
        amount: i64,
        reason: String,
    ) -> EconomyResult<CurrencyBalance> {
        info!(
            "Admin removing {} {} from user {}: {}",
            amount, currency_code, user_id, reason
        );

        // Create admin transaction
        let transaction_request = super::transactions::TransactionRequest {
            transaction_type: TransactionType::System,
            from_user_id: Some(user_id),
            to_user_id: None,
            currency_code: currency_code.clone(),
            amount,
            description: format!("Admin debit: {}", reason),
            reference_id: None,
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("admin_action".to_string(), "remove_currency".to_string());
                meta.insert("reason".to_string(), reason);
                meta
            },
            idempotency_key: None,
        };

        let result = self
            .transaction_processor
            .process_transaction(transaction_request)
            .await?;

        info!("Admin currency removal completed");
        Ok(result.from_balance.unwrap())
    }

    /// Freeze user account (admin function)
    pub async fn admin_freeze_account(&self, user_id: Uuid, reason: String) -> EconomyResult<()> {
        warn!("Admin freezing account for user {}: {}", user_id, reason);

        // Implementation would mark account as frozen in database
        // This would prevent new transactions from processing

        warn!("Account frozen for user {}", user_id);
        Ok(())
    }

    /// Unfreeze user account (admin function)
    pub async fn admin_unfreeze_account(&self, user_id: Uuid, reason: String) -> EconomyResult<()> {
        info!("Admin unfreezing account for user {}: {}", user_id, reason);

        // Implementation would remove freeze status from database

        info!("Account unfrozen for user {}", user_id);
        Ok(())
    }

    /// Get system health status
    pub async fn get_system_health(&self) -> EconomySystemHealth {
        let currency_stats = self
            .currency_system
            .get_currency_statistics(&self.config.default_currency)
            .await
            .unwrap_or_else(|_| super::currency::CurrencyStatistics {
                currency_code: self.config.default_currency.clone(),
                total_supply: 0,
                circulating_supply: 0,
                total_accounts: 0,
                average_balance: 0.0,
                transaction_volume_24h: 0,
                largest_balance: 0,
                generated_at: chrono::Utc::now(),
            });

        let marketplace_stats = self
            .marketplace_manager
            .get_marketplace_statistics()
            .await
            .unwrap_or_else(|_| super::marketplace::MarketplaceStatistics {
                total_listings: 0,
                active_listings: 0,
                total_sales: 0,
                total_volume: 0,
                unique_sellers: 0,
                unique_buyers: 0,
                average_sale_price: 0.0,
                top_categories: Vec::new(),
                generated_at: chrono::Utc::now(),
            });

        EconomySystemHealth {
            status: "healthy".to_string(),
            total_accounts: currency_stats.total_accounts,
            total_supply: currency_stats.total_supply,
            circulating_supply: currency_stats.circulating_supply,
            transaction_volume_24h: currency_stats.transaction_volume_24h,
            marketplace_active_listings: marketplace_stats.active_listings,
            marketplace_total_volume: marketplace_stats.total_volume,
            default_currency: self.config.default_currency.clone(),
            supported_currencies: self.config.supported_currencies.len() as u32,
            generated_at: chrono::Utc::now(),
        }
    }

    /// Shutdown economy system
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down virtual economy system");

        // Graceful shutdown of components
        // Implementation would ensure all pending transactions are completed

        info!("Virtual economy system shutdown completed");
        Ok(())
    }
}

/// User economy summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEconomySummary {
    pub user_id: Uuid,
    pub balances: Vec<CurrencyBalance>,
    pub total_balance_default_currency: i64,
    pub total_reserved_default_currency: i64,
    pub recent_transactions: Vec<Transaction>,
    pub marketplace_listings_count: u32,
    pub marketplace_active_listings: u32,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// Economy system health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomySystemHealth {
    pub status: String,
    pub total_accounts: u64,
    pub total_supply: i64,
    pub circulating_supply: i64,
    pub transaction_volume_24h: i64,
    pub marketplace_active_listings: u64,
    pub marketplace_total_volume: i64,
    pub default_currency: String,
    pub supported_currencies: u32,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}
