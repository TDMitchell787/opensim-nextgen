//! Virtual Currency System for OpenSim Next
//!
//! Provides multi-currency virtual currency management with secure
//! balance tracking, currency conversion, and transaction processing.

use super::*;
use crate::database::DatabaseManager;
use anyhow::Result;
use sqlx::Row;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Virtual currency management system
pub struct CurrencySystem {
    database: Arc<DatabaseManager>,
    config: EconomyConfig,
    currency_definitions: Arc<RwLock<HashMap<String, CurrencyDefinition>>>,
    balance_cache: Arc<RwLock<HashMap<(Uuid, String), CurrencyBalance>>>,
    exchange_rates: Arc<RwLock<HashMap<String, f64>>>,
}

impl CurrencySystem {
    /// Create new currency system
    pub fn new(database: Arc<DatabaseManager>, config: EconomyConfig) -> Self {
        let currency_definitions = Arc::new(RwLock::new(
            config
                .supported_currencies
                .iter()
                .map(|c| (c.currency_code.clone(), c.clone()))
                .collect(),
        ));

        let exchange_rates = Arc::new(RwLock::new(
            config
                .supported_currencies
                .iter()
                .map(|c| (c.currency_code.clone(), c.exchange_rate_to_base))
                .collect(),
        ));

        Self {
            database,
            config,
            currency_definitions,
            balance_cache: Arc::new(RwLock::new(HashMap::new())),
            exchange_rates,
        }
    }

    /// Initialize currency system
    pub async fn initialize(&self) -> EconomyResult<()> {
        info!("Initializing virtual currency system");

        // Create database tables
        self.create_tables().await?;

        // Load existing currency definitions
        self.load_currency_definitions().await?;

        // Validate configuration
        self.validate_config().await?;

        info!("Virtual currency system initialized successfully");
        Ok(())
    }

    /// Get user balance for currency
    pub async fn get_balance(
        &self,
        user_id: Uuid,
        currency_code: &str,
    ) -> EconomyResult<CurrencyBalance> {
        debug!(
            "Getting balance for user {} currency {}",
            user_id, currency_code
        );

        // Check cache first
        {
            let cache = self.balance_cache.read().await;
            if let Some(balance) = cache.get(&(user_id, currency_code.to_string())) {
                return Ok(balance.clone());
            }
        }

        // Load from database
        let balance = self.load_balance_from_db(user_id, currency_code).await?;

        // Update cache
        {
            let mut cache = self.balance_cache.write().await;
            cache.insert((user_id, currency_code.to_string()), balance.clone());
        }

        debug!("Balance retrieved: {} {}", balance.available, currency_code);
        Ok(balance)
    }

    /// Get all balances for user
    pub async fn get_all_balances(&self, user_id: Uuid) -> EconomyResult<Vec<CurrencyBalance>> {
        debug!("Getting all balances for user {}", user_id);

        let currencies = {
            let defs = self.currency_definitions.read().await;
            defs.keys().cloned().collect::<Vec<_>>()
        };

        let mut balances = Vec::new();
        for currency_code in currencies {
            match self.get_balance(user_id, &currency_code).await {
                Ok(balance) => {
                    if balance.balance > 0 || balance.reserved > 0 {
                        balances.push(balance);
                    }
                }
                Err(EconomyError::UserNotFound { .. }) => {
                    // User doesn't have this currency, skip
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        debug!("Retrieved {} balances for user {}", balances.len(), user_id);
        Ok(balances)
    }

    /// Create or update user balance
    pub async fn create_balance(
        &self,
        user_id: Uuid,
        currency_code: &str,
        initial_amount: i64,
    ) -> EconomyResult<CurrencyBalance> {
        info!(
            "Creating balance for user {} currency {} amount {}",
            user_id, currency_code, initial_amount
        );

        // Validate currency
        self.validate_currency(currency_code).await?;

        // Validate amount
        let currency_def = self.get_currency_definition(currency_code).await?;
        if initial_amount < currency_def.minimum_balance {
            return Err(EconomyError::TransactionFailed {
                reason: format!(
                    "Initial amount {} below minimum {}",
                    initial_amount, currency_def.minimum_balance
                ),
            });
        }

        if initial_amount > currency_def.maximum_balance {
            return Err(EconomyError::TransactionFailed {
                reason: format!(
                    "Initial amount {} above maximum {}",
                    initial_amount, currency_def.maximum_balance
                ),
            });
        }

        let balance = CurrencyBalance {
            user_id,
            currency_code: currency_code.to_string(),
            balance: initial_amount,
            reserved: 0,
            available: initial_amount,
            last_updated: chrono::Utc::now(),
            version: 1,
        };

        // Save to database
        self.save_balance_to_db(&balance).await?;

        // Update cache
        {
            let mut cache = self.balance_cache.write().await;
            cache.insert((user_id, currency_code.to_string()), balance.clone());
        }

        info!("Balance created successfully for user {}", user_id);
        Ok(balance)
    }

    /// Add currency to user balance
    pub async fn add_currency(
        &self,
        user_id: Uuid,
        currency_code: &str,
        amount: i64,
        description: String,
    ) -> EconomyResult<CurrencyBalance> {
        info!(
            "Adding {} {} to user {} ({})",
            amount, currency_code, user_id, description
        );

        if amount <= 0 {
            return Err(EconomyError::TransactionFailed {
                reason: "Amount must be positive".to_string(),
            });
        }

        // Get current balance
        let mut current_balance = match self.get_balance(user_id, currency_code).await {
            Ok(balance) => balance,
            Err(EconomyError::UserNotFound { .. }) => {
                // Create new balance
                self.create_balance(user_id, currency_code, 0).await?
            }
            Err(e) => return Err(e),
        };

        // Validate new balance won't exceed limits
        let currency_def = self.get_currency_definition(currency_code).await?;
        let new_balance = current_balance.balance + amount;
        if new_balance > currency_def.maximum_balance {
            return Err(EconomyError::TransactionFailed {
                reason: format!(
                    "New balance {} would exceed maximum {}",
                    new_balance, currency_def.maximum_balance
                ),
            });
        }

        // Update balance
        current_balance.balance = new_balance;
        current_balance.available = new_balance - current_balance.reserved;
        current_balance.last_updated = chrono::Utc::now();
        current_balance.version += 1;

        // Save to database with optimistic locking
        self.update_balance_with_version(&current_balance).await?;

        // Update cache
        {
            let mut cache = self.balance_cache.write().await;
            cache.insert(
                (user_id, currency_code.to_string()),
                current_balance.clone(),
            );
        }

        info!(
            "Successfully added {} {} to user {}",
            amount, currency_code, user_id
        );
        Ok(current_balance)
    }

    /// Subtract currency from user balance
    pub async fn subtract_currency(
        &self,
        user_id: Uuid,
        currency_code: &str,
        amount: i64,
        description: String,
    ) -> EconomyResult<CurrencyBalance> {
        info!(
            "Subtracting {} {} from user {} ({})",
            amount, currency_code, user_id, description
        );

        if amount <= 0 {
            return Err(EconomyError::TransactionFailed {
                reason: "Amount must be positive".to_string(),
            });
        }

        // Get current balance
        let mut current_balance = self.get_balance(user_id, currency_code).await?;

        // Check sufficient funds
        if current_balance.available < amount {
            return Err(EconomyError::InsufficientFunds {
                required: amount,
                available: current_balance.available,
            });
        }

        // Update balance
        current_balance.balance -= amount;
        current_balance.available = current_balance.balance - current_balance.reserved;
        current_balance.last_updated = chrono::Utc::now();
        current_balance.version += 1;

        // Save to database with optimistic locking
        self.update_balance_with_version(&current_balance).await?;

        // Update cache
        {
            let mut cache = self.balance_cache.write().await;
            cache.insert(
                (user_id, currency_code.to_string()),
                current_balance.clone(),
            );
        }

        info!(
            "Successfully subtracted {} {} from user {}",
            amount, currency_code, user_id
        );
        Ok(current_balance)
    }

    /// Reserve currency for pending transaction
    pub async fn reserve_currency(
        &self,
        user_id: Uuid,
        currency_code: &str,
        amount: i64,
    ) -> EconomyResult<CurrencyBalance> {
        debug!(
            "Reserving {} {} for user {}",
            amount, currency_code, user_id
        );

        if amount <= 0 {
            return Err(EconomyError::TransactionFailed {
                reason: "Amount must be positive".to_string(),
            });
        }

        // Get current balance
        let mut current_balance = self.get_balance(user_id, currency_code).await?;

        // Check sufficient available funds
        if current_balance.available < amount {
            return Err(EconomyError::InsufficientFunds {
                required: amount,
                available: current_balance.available,
            });
        }

        // Update reservation
        current_balance.reserved += amount;
        current_balance.available = current_balance.balance - current_balance.reserved;
        current_balance.last_updated = chrono::Utc::now();
        current_balance.version += 1;

        // Save to database
        self.update_balance_with_version(&current_balance).await?;

        // Update cache
        {
            let mut cache = self.balance_cache.write().await;
            cache.insert(
                (user_id, currency_code.to_string()),
                current_balance.clone(),
            );
        }

        debug!(
            "Successfully reserved {} {} for user {}",
            amount, currency_code, user_id
        );
        Ok(current_balance)
    }

    /// Release reserved currency
    pub async fn release_reservation(
        &self,
        user_id: Uuid,
        currency_code: &str,
        amount: i64,
    ) -> EconomyResult<CurrencyBalance> {
        debug!(
            "Releasing {} {} reservation for user {}",
            amount, currency_code, user_id
        );

        if amount <= 0 {
            return Err(EconomyError::TransactionFailed {
                reason: "Amount must be positive".to_string(),
            });
        }

        // Get current balance
        let mut current_balance = self.get_balance(user_id, currency_code).await?;

        // Validate reservation amount
        if current_balance.reserved < amount {
            return Err(EconomyError::TransactionFailed {
                reason: format!(
                    "Cannot release {} - only {} reserved",
                    amount, current_balance.reserved
                ),
            });
        }

        // Update reservation
        current_balance.reserved -= amount;
        current_balance.available = current_balance.balance - current_balance.reserved;
        current_balance.last_updated = chrono::Utc::now();
        current_balance.version += 1;

        // Save to database
        self.update_balance_with_version(&current_balance).await?;

        // Update cache
        {
            let mut cache = self.balance_cache.write().await;
            cache.insert(
                (user_id, currency_code.to_string()),
                current_balance.clone(),
            );
        }

        debug!(
            "Successfully released {} {} reservation for user {}",
            amount, currency_code, user_id
        );
        Ok(current_balance)
    }

    /// Convert currency between types
    pub async fn convert_currency(
        &self,
        user_id: Uuid,
        from_currency: &str,
        to_currency: &str,
        amount: i64,
    ) -> EconomyResult<(CurrencyBalance, CurrencyBalance)> {
        info!(
            "Converting {} {} to {} for user {}",
            amount, from_currency, to_currency, user_id
        );

        // Validate currencies
        self.validate_currency(from_currency).await?;
        self.validate_currency(to_currency).await?;

        if from_currency == to_currency {
            return Err(EconomyError::TransactionFailed {
                reason: "Cannot convert currency to itself".to_string(),
            });
        }

        // Get exchange rates
        let exchange_rates = self.exchange_rates.read().await;
        let from_rate = exchange_rates.get(from_currency).copied().unwrap_or(1.0);
        let to_rate = exchange_rates.get(to_currency).copied().unwrap_or(1.0);

        // Calculate conversion
        let base_amount = (amount as f64) * from_rate;
        let converted_amount = (base_amount / to_rate) as i64;

        // Apply conversion fee (0.5%)
        let fee = (converted_amount as f64 * 0.005) as i64;
        let final_amount = converted_amount - fee;

        if final_amount <= 0 {
            return Err(EconomyError::TransactionFailed {
                reason: "Converted amount too small after fees".to_string(),
            });
        }

        // Perform the conversion
        let from_balance = self
            .subtract_currency(
                user_id,
                from_currency,
                amount,
                format!("Currency conversion to {}", to_currency),
            )
            .await?;
        let to_balance = self
            .add_currency(
                user_id,
                to_currency,
                final_amount,
                format!("Currency conversion from {}", from_currency),
            )
            .await?;

        info!(
            "Successfully converted {} {} to {} {} for user {}",
            amount, from_currency, final_amount, to_currency, user_id
        );
        Ok((from_balance, to_balance))
    }

    /// Get currency definition
    pub async fn get_currency_definition(
        &self,
        currency_code: &str,
    ) -> EconomyResult<CurrencyDefinition> {
        let definitions = self.currency_definitions.read().await;
        definitions
            .get(currency_code)
            .cloned()
            .ok_or_else(|| EconomyError::InvalidCurrency {
                currency: currency_code.to_string(),
            })
    }

    /// Add new currency definition
    pub async fn add_currency_definition(
        &self,
        definition: CurrencyDefinition,
    ) -> EconomyResult<()> {
        info!("Adding currency definition: {}", definition.currency_code);

        // Validate definition
        if definition.currency_code.is_empty() {
            return Err(EconomyError::TransactionFailed {
                reason: "Currency code cannot be empty".to_string(),
            });
        }

        if definition.decimal_places > 8 {
            return Err(EconomyError::TransactionFailed {
                reason: "Currency cannot have more than 8 decimal places".to_string(),
            });
        }

        // Save to database
        self.save_currency_definition(&definition).await?;

        // Update in-memory cache
        {
            let mut definitions = self.currency_definitions.write().await;
            definitions.insert(definition.currency_code.clone(), definition.clone());
        }

        {
            let mut rates = self.exchange_rates.write().await;
            rates.insert(
                definition.currency_code.clone(),
                definition.exchange_rate_to_base,
            );
        }

        info!(
            "Currency definition added successfully: {}",
            definition.currency_code
        );
        Ok(())
    }

    /// Update exchange rate
    pub async fn update_exchange_rate(
        &self,
        currency_code: &str,
        new_rate: f64,
    ) -> EconomyResult<()> {
        info!("Updating exchange rate for {}: {}", currency_code, new_rate);

        // Validate currency exists
        self.validate_currency(currency_code).await?;

        if new_rate <= 0.0 {
            return Err(EconomyError::TransactionFailed {
                reason: "Exchange rate must be positive".to_string(),
            });
        }

        // Update database
        self.update_currency_exchange_rate(currency_code, new_rate)
            .await?;

        // Update cache
        {
            let mut rates = self.exchange_rates.write().await;
            rates.insert(currency_code.to_string(), new_rate);
        }

        {
            let mut definitions = self.currency_definitions.write().await;
            if let Some(def) = definitions.get_mut(currency_code) {
                def.exchange_rate_to_base = new_rate;
            }
        }

        info!("Exchange rate updated successfully for {}", currency_code);
        Ok(())
    }

    /// Get currency statistics
    pub async fn get_currency_statistics(
        &self,
        currency_code: &str,
    ) -> EconomyResult<CurrencyStatistics> {
        self.validate_currency(currency_code).await?;

        let stats = CurrencyStatistics {
            currency_code: currency_code.to_string(),
            total_supply: self.get_total_supply(currency_code).await?,
            circulating_supply: self.get_circulating_supply(currency_code).await?,
            total_accounts: self.get_account_count(currency_code).await?,
            average_balance: self.get_average_balance(currency_code).await?,
            transaction_volume_24h: self.get_transaction_volume_24h(currency_code).await?,
            largest_balance: self.get_largest_balance(currency_code).await?,
            generated_at: chrono::Utc::now(),
        };

        Ok(stats)
    }

    // Private helper methods

    async fn validate_currency(&self, currency_code: &str) -> EconomyResult<()> {
        let definitions = self.currency_definitions.read().await;
        if !definitions.contains_key(currency_code) {
            return Err(EconomyError::InvalidCurrency {
                currency: currency_code.to_string(),
            });
        }

        let definition = &definitions[currency_code];
        if !definition.enabled {
            return Err(EconomyError::TransactionFailed {
                reason: format!("Currency {} is disabled", currency_code),
            });
        }

        Ok(())
    }

    async fn validate_config(&self) -> EconomyResult<()> {
        if self.config.supported_currencies.is_empty() {
            return Err(EconomyError::SystemError {
                message: "No currencies configured".to_string(),
            });
        }

        if !self
            .config
            .supported_currencies
            .iter()
            .any(|c| c.currency_code == self.config.default_currency)
        {
            return Err(EconomyError::SystemError {
                message: "Default currency not in supported currencies".to_string(),
            });
        }

        Ok(())
    }

    // Database operations (simplified implementations)

    async fn create_tables(&self) -> EconomyResult<()> {
        info!("Economy tables managed by migration 042_economy_tables.sql — skipping runtime creation");
        Ok(())
    }

    async fn load_currency_definitions(&self) -> EconomyResult<()> {
        info!("Loading currency definitions from database");

        let rows = sqlx::query(
            "SELECT currency_code, currency_name, currency_symbol, decimal_places, exchange_rate_to_base, enabled, minimum_balance, maximum_balance FROM currency_definitions WHERE enabled = true"
        )
        .fetch_all(self.database.legacy_pool()?)
        .await;

        let rows = match rows {
            Ok(r) => r,
            Err(e) => {
                warn!(
                    "No currency_definitions in DB yet ({}), using config defaults",
                    e
                );
                return Ok(());
            }
        };

        if rows.is_empty() {
            info!("No currency definitions in DB, using config defaults");
            return Ok(());
        }

        let mut definitions = self.currency_definitions.write().await;
        let mut exchange_rates = self.exchange_rates.write().await;

        for row in rows {
            let currency_code: String = row.try_get("currency_code")?;
            let enabled: bool = row.try_get("enabled")?;

            let definition = CurrencyDefinition {
                currency_code: currency_code.clone(),
                currency_name: row.try_get("currency_name")?,
                currency_symbol: row.try_get("currency_symbol")?,
                decimal_places: row.try_get::<i32, _>("decimal_places")? as u8,
                exchange_rate_to_base: row.try_get("exchange_rate_to_base")?,
                enabled,
                minimum_balance: row.try_get::<i64, _>("minimum_balance")?,
                maximum_balance: row.try_get::<i64, _>("maximum_balance")?,
            };

            exchange_rates.insert(currency_code.clone(), definition.exchange_rate_to_base);
            definitions.insert(currency_code, definition);
        }

        info!(
            "Loaded {} currency definitions from database",
            definitions.len()
        );
        Ok(())
    }

    async fn load_balance_from_db(
        &self,
        user_id: Uuid,
        currency_code: &str,
    ) -> EconomyResult<CurrencyBalance> {
        debug!(
            "Loading balance from database: {} {}",
            user_id, currency_code
        );

        let row = sqlx::query(
            "SELECT balance, reserved, available, updated_at, version FROM currency_balances WHERE user_id = $1 AND currency_code = $2"
        )
        .bind(user_id)
        .bind(currency_code)
        .fetch_optional(self.database.legacy_pool()?)
        .await?;

        if let Some(row) = row {
            let last_updated: chrono::DateTime<chrono::Utc> = row
                .try_get("updated_at")
                .unwrap_or_else(|_| chrono::Utc::now());

            Ok(CurrencyBalance {
                user_id,
                currency_code: currency_code.to_string(),
                balance: row.try_get::<i64, _>("balance")?,
                reserved: row.try_get::<i64, _>("reserved")?,
                available: row.try_get::<i64, _>("available")?,
                last_updated,
                version: row.try_get::<i32, _>("version")? as i64,
            })
        } else {
            let default_balance = CurrencyBalance {
                user_id,
                currency_code: currency_code.to_string(),
                balance: 0,
                reserved: 0,
                available: 0,
                last_updated: chrono::Utc::now(),
                version: 1,
            };

            self.save_balance_to_db(&default_balance).await?;
            Ok(default_balance)
        }
    }

    async fn save_balance_to_db(&self, balance: &CurrencyBalance) -> EconomyResult<()> {
        debug!(
            "Saving balance to database: {} {} = {}",
            balance.user_id, balance.currency_code, balance.balance
        );

        sqlx::query(
            r#"
            INSERT INTO currency_balances (user_id, currency_code, balance, reserved, available, updated_at, version)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (user_id, currency_code) DO UPDATE SET
                balance = EXCLUDED.balance,
                reserved = EXCLUDED.reserved,
                available = EXCLUDED.available,
                updated_at = EXCLUDED.updated_at,
                version = EXCLUDED.version
            "#
        )
        .bind(balance.user_id)
        .bind(&balance.currency_code)
        .bind(balance.balance)
        .bind(balance.reserved)
        .bind(balance.available)
        .bind(balance.last_updated)
        .bind(balance.version as i32)
        .execute(self.database.legacy_pool()?)
        .await?;

        debug!("Balance saved successfully");
        Ok(())
    }

    async fn update_balance_with_version(&self, balance: &CurrencyBalance) -> EconomyResult<()> {
        debug!(
            "Updating balance with version check: {} {} v{}",
            balance.user_id, balance.currency_code, balance.version
        );

        let old_version = balance.version - 1;
        let result = sqlx::query(
            r#"
            UPDATE currency_balances
            SET balance = $1, reserved = $2, available = $3, updated_at = $4, version = $5
            WHERE user_id = $6 AND currency_code = $7 AND version = $8
            "#,
        )
        .bind(balance.balance)
        .bind(balance.reserved)
        .bind(balance.available)
        .bind(balance.last_updated)
        .bind(balance.version as i32)
        .bind(balance.user_id)
        .bind(&balance.currency_code)
        .bind(old_version as i32)
        .execute(self.database.legacy_pool()?)
        .await?;

        if result.rows_affected() == 0 {
            return Err(EconomyError::TransactionFailed {
                reason: "Concurrent modification detected - balance version mismatch".to_string(),
            });
        }

        debug!("Balance updated successfully with optimistic locking");
        Ok(())
    }

    async fn save_currency_definition(&self, definition: &CurrencyDefinition) -> EconomyResult<()> {
        debug!("Saving currency definition: {}", definition.currency_code);

        sqlx::query(
            r#"
            INSERT INTO currency_definitions (currency_code, currency_name, currency_symbol, decimal_places, exchange_rate_to_base, enabled, minimum_balance, maximum_balance)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (currency_code) DO UPDATE SET
                currency_name = EXCLUDED.currency_name,
                currency_symbol = EXCLUDED.currency_symbol,
                decimal_places = EXCLUDED.decimal_places,
                exchange_rate_to_base = EXCLUDED.exchange_rate_to_base,
                enabled = EXCLUDED.enabled,
                minimum_balance = EXCLUDED.minimum_balance,
                maximum_balance = EXCLUDED.maximum_balance
            "#
        )
        .bind(&definition.currency_code)
        .bind(&definition.currency_name)
        .bind(&definition.currency_symbol)
        .bind(definition.decimal_places as i32)
        .bind(definition.exchange_rate_to_base)
        .bind(definition.enabled)
        .bind(definition.minimum_balance)
        .bind(definition.maximum_balance)
        .execute(self.database.legacy_pool()?)
        .await?;

        debug!("Currency definition saved successfully");
        Ok(())
    }

    async fn update_currency_exchange_rate(
        &self,
        currency_code: &str,
        rate: f64,
    ) -> EconomyResult<()> {
        debug!("Updating exchange rate for {}: {}", currency_code, rate);

        let result = sqlx::query(
            "UPDATE currency_definitions SET exchange_rate_to_base = $1 WHERE currency_code = $2",
        )
        .bind(rate)
        .bind(currency_code)
        .execute(self.database.legacy_pool()?)
        .await?;

        if result.rows_affected() == 0 {
            return Err(EconomyError::InvalidCurrency {
                currency: currency_code.to_string(),
            });
        }

        debug!("Exchange rate updated successfully");
        Ok(())
    }

    async fn get_total_supply(&self, currency_code: &str) -> EconomyResult<i64> {
        debug!("Calculating total supply for currency: {}", currency_code);

        let row = sqlx::query(
            "SELECT COALESCE(SUM(balance), 0) as total FROM currency_balances WHERE currency_code = $1"
        )
        .bind(currency_code)
        .fetch_one(self.database.legacy_pool()?)
        .await?;

        let total: i64 = row.try_get("total").unwrap_or(0);
        debug!("Total supply for {}: {}", currency_code, total);
        Ok(total)
    }

    async fn get_circulating_supply(&self, currency_code: &str) -> EconomyResult<i64> {
        debug!(
            "Calculating circulating supply for currency: {}",
            currency_code
        );

        let row = sqlx::query(
            "SELECT COALESCE(SUM(available), 0) as circulating FROM currency_balances WHERE currency_code = $1"
        )
        .bind(currency_code)
        .fetch_one(self.database.legacy_pool()?)
        .await?;

        let circulating: i64 = row.try_get("circulating").unwrap_or(0);
        debug!("Circulating supply for {}: {}", currency_code, circulating);
        Ok(circulating)
    }

    async fn get_account_count(&self, currency_code: &str) -> EconomyResult<u64> {
        debug!("Counting accounts for currency: {}", currency_code);

        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM currency_balances WHERE currency_code = $1 AND (balance > 0 OR reserved > 0)"
        )
        .bind(currency_code)
        .fetch_one(self.database.legacy_pool()?)
        .await?;

        let count: i64 = row.try_get("count").unwrap_or(0);
        debug!("Account count for {}: {}", currency_code, count);
        Ok(count as u64)
    }

    async fn get_average_balance(&self, currency_code: &str) -> EconomyResult<f64> {
        debug!(
            "Calculating average balance for currency: {}",
            currency_code
        );

        let row = sqlx::query(
            "SELECT COALESCE(AVG(balance::double precision), 0.0) as average FROM currency_balances WHERE currency_code = $1 AND balance > 0"
        )
        .bind(currency_code)
        .fetch_one(self.database.legacy_pool()?)
        .await?;

        let average: f64 = row.try_get("average").unwrap_or(0.0);
        debug!("Average balance for {}: {:.2}", currency_code, average);
        Ok(average)
    }

    async fn get_transaction_volume_24h(&self, currency_code: &str) -> EconomyResult<i64> {
        debug!(
            "Calculating 24h transaction volume for currency: {}",
            currency_code
        );

        let twenty_four_hours_ago = chrono::Utc::now() - chrono::Duration::hours(24);

        let row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(amount), 0) as volume
            FROM economy_transactions
            WHERE currency_code = $1
              AND created_at >= $2
              AND status = 'Completed'
            "#,
        )
        .bind(currency_code)
        .bind(twenty_four_hours_ago)
        .fetch_optional(self.database.legacy_pool()?)
        .await?;

        let volume = if let Some(row) = row {
            row.try_get::<i64, _>("volume").unwrap_or(0)
        } else {
            0
        };

        debug!("24h volume for {}: {}", currency_code, volume);
        Ok(volume)
    }

    async fn get_largest_balance(&self, currency_code: &str) -> EconomyResult<i64> {
        debug!("Finding largest balance for currency: {}", currency_code);

        let row = sqlx::query(
            "SELECT COALESCE(MAX(balance), 0) as largest FROM currency_balances WHERE currency_code = $1"
        )
        .bind(currency_code)
        .fetch_one(self.database.legacy_pool()?)
        .await?;

        let largest: i64 = row.try_get("largest").unwrap_or(0);
        debug!("Largest balance for {}: {}", currency_code, largest);
        Ok(largest)
    }

    pub async fn get_holder_count(&self, currency_code: &str) -> EconomyResult<u64> {
        debug!("Counting currency holders for: {}", currency_code);
        self.get_account_count(currency_code).await
    }

    pub async fn get_24h_volume(&self, currency_code: &str) -> EconomyResult<i64> {
        debug!("Getting 24h volume for: {}", currency_code);
        self.get_transaction_volume_24h(currency_code).await
    }
}

/// Currency statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyStatistics {
    pub currency_code: String,
    pub total_supply: i64,
    pub circulating_supply: i64,
    pub total_accounts: u64,
    pub average_balance: f64,
    pub transaction_volume_24h: i64,
    pub largest_balance: i64,
    pub generated_at: DateTime<Utc>,
}
