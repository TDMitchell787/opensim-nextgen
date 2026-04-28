//! Economic Analytics Engine for OpenSim Next
//!
//! Provides comprehensive economic analysis, reporting, and insights
//! for virtual world economy management.

use super::*;
use crate::database::{DatabaseManager, DatabasePoolRef};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

pub struct AnalyticsEngine {
    database: Arc<DatabaseManager>,
    config: AnalyticsConfig,
}

impl AnalyticsEngine {
    pub fn new(database: Arc<DatabaseManager>, config: AnalyticsConfig) -> Self {
        Self { database, config }
    }

    pub async fn initialize(&self) -> EconomyResult<()> {
        info!("Initializing economic analytics engine");

        if !self.config.enabled {
            info!("Analytics engine is disabled");
            return Ok(());
        }

        self.create_analytics_tables().await?;

        info!("Economic analytics engine initialized successfully");
        Ok(())
    }

    pub async fn generate_economy_metrics(&self) -> EconomyResult<EconomicMetrics> {
        debug!("Generating economic metrics");

        let now = chrono::Utc::now();
        let period_start = now - chrono::Duration::days(1);

        let metrics = EconomicMetrics {
            total_transactions: self.get_total_transactions(period_start, now).await?,
            total_volume: self.get_total_volume(period_start, now).await?,
            active_users: self.get_active_users(period_start, now).await?,
            marketplace_listings: self.get_marketplace_listings_count().await?,
            average_transaction_size: self.get_average_transaction_size(period_start, now).await?,
            transaction_velocity: self.get_transaction_velocity(period_start, now).await?,
            currency_circulation: self.get_currency_circulation().await?,
            top_categories: self.get_top_categories(period_start, now).await?,
            fraud_incidents: self.get_fraud_incidents(period_start, now).await?,
            period_start,
            period_end: now,
        };

        debug!("Economic metrics generated successfully");
        Ok(metrics)
    }

    async fn create_analytics_tables(&self) -> EconomyResult<()> {
        let pool = match self.database.get_pool() {
            Ok(p) => p,
            Err(e) => {
                warn!("Could not get database pool for analytics: {}", e);
                return Ok(());
            }
        };

        match pool {
            DatabasePoolRef::PostgreSQL(pg_pool) => {
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS economy_transactions (
                        transaction_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                        transaction_type VARCHAR(50) NOT NULL,
                        from_user_id UUID,
                        to_user_id UUID,
                        currency_code VARCHAR(10) NOT NULL DEFAULT 'L$',
                        amount BIGINT NOT NULL,
                        fee BIGINT DEFAULT 0,
                        description TEXT,
                        reference_id UUID,
                        status VARCHAR(20) NOT NULL DEFAULT 'Pending',
                        created_at TIMESTAMPTZ DEFAULT NOW(),
                        processed_at TIMESTAMPTZ,
                        metadata JSONB DEFAULT '{}'
                    )",
                )
                .execute(pg_pool)
                .await?;

                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS currency_balances (
                        id SERIAL PRIMARY KEY,
                        user_id UUID NOT NULL,
                        currency_code VARCHAR(10) NOT NULL DEFAULT 'L$',
                        balance BIGINT NOT NULL DEFAULT 0,
                        reserved BIGINT DEFAULT 0,
                        last_updated TIMESTAMPTZ DEFAULT NOW(),
                        version BIGINT DEFAULT 1,
                        UNIQUE(user_id, currency_code)
                    )",
                )
                .execute(pg_pool)
                .await?;

                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS marketplace_listings (
                        listing_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                        seller_id UUID NOT NULL,
                        item_id UUID NOT NULL,
                        item_type VARCHAR(50),
                        category_id UUID,
                        title VARCHAR(255) NOT NULL,
                        description TEXT,
                        price BIGINT NOT NULL,
                        currency_code VARCHAR(10) DEFAULT 'L$',
                        quantity_available INT DEFAULT 1,
                        quantity_sold INT DEFAULT 0,
                        listing_status VARCHAR(20) DEFAULT 'Active',
                        created_at TIMESTAMPTZ DEFAULT NOW(),
                        updated_at TIMESTAMPTZ DEFAULT NOW(),
                        expires_at TIMESTAMPTZ,
                        featured BOOLEAN DEFAULT FALSE
                    )",
                )
                .execute(pg_pool)
                .await?;

                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS fraud_alerts (
                        alert_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                        user_id UUID NOT NULL,
                        alert_type VARCHAR(50) NOT NULL,
                        severity VARCHAR(20) NOT NULL,
                        description TEXT,
                        transaction_id UUID,
                        risk_score DOUBLE PRECISION DEFAULT 0.0,
                        created_at TIMESTAMPTZ DEFAULT NOW(),
                        investigated_at TIMESTAMPTZ,
                        resolved_at TIMESTAMPTZ,
                        status VARCHAR(20) DEFAULT 'Open'
                    )",
                )
                .execute(pg_pool)
                .await?;

                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS marketplace_categories (
                        category_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                        name VARCHAR(100) NOT NULL,
                        description TEXT,
                        parent_category UUID,
                        commission_rate DOUBLE PRECISION
                    )",
                )
                .execute(pg_pool)
                .await?;

                sqlx::query("CREATE INDEX IF NOT EXISTS idx_transactions_created ON economy_transactions(created_at)")
                    .execute(pg_pool)
                    .await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_transactions_status ON economy_transactions(status)")
                    .execute(pg_pool)
                    .await?;
                sqlx::query("CREATE INDEX IF NOT EXISTS idx_listings_status ON marketplace_listings(listing_status)")
                    .execute(pg_pool)
                    .await?;
                sqlx::query(
                    "CREATE INDEX IF NOT EXISTS idx_fraud_created ON fraud_alerts(created_at)",
                )
                .execute(pg_pool)
                .await?;

                info!("Created PostgreSQL analytics tables");
            }
            DatabasePoolRef::MySQL(mysql_pool) => {
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS economy_transactions (
                        transaction_id VARCHAR(36) PRIMARY KEY,
                        transaction_type VARCHAR(50) NOT NULL,
                        from_user_id VARCHAR(36),
                        to_user_id VARCHAR(36),
                        currency_code VARCHAR(10) NOT NULL DEFAULT 'L$',
                        amount BIGINT NOT NULL,
                        fee BIGINT DEFAULT 0,
                        description TEXT,
                        reference_id VARCHAR(36),
                        status VARCHAR(20) NOT NULL DEFAULT 'Pending',
                        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                        processed_at DATETIME,
                        metadata JSON
                    )",
                )
                .execute(mysql_pool)
                .await?;

                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS currency_balances (
                        id INT AUTO_INCREMENT PRIMARY KEY,
                        user_id VARCHAR(36) NOT NULL,
                        currency_code VARCHAR(10) NOT NULL DEFAULT 'L$',
                        balance BIGINT NOT NULL DEFAULT 0,
                        reserved BIGINT DEFAULT 0,
                        last_updated DATETIME DEFAULT CURRENT_TIMESTAMP,
                        version BIGINT DEFAULT 1,
                        UNIQUE KEY unique_user_currency (user_id, currency_code)
                    )",
                )
                .execute(mysql_pool)
                .await?;

                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS marketplace_listings (
                        listing_id VARCHAR(36) PRIMARY KEY,
                        seller_id VARCHAR(36) NOT NULL,
                        item_id VARCHAR(36) NOT NULL,
                        item_type VARCHAR(50),
                        category_id VARCHAR(36),
                        title VARCHAR(255) NOT NULL,
                        description TEXT,
                        price BIGINT NOT NULL,
                        currency_code VARCHAR(10) DEFAULT 'L$',
                        quantity_available INT DEFAULT 1,
                        quantity_sold INT DEFAULT 0,
                        listing_status VARCHAR(20) DEFAULT 'Active',
                        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                        updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                        expires_at DATETIME,
                        featured BOOLEAN DEFAULT FALSE
                    )",
                )
                .execute(mysql_pool)
                .await?;

                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS fraud_alerts (
                        alert_id VARCHAR(36) PRIMARY KEY,
                        user_id VARCHAR(36) NOT NULL,
                        alert_type VARCHAR(50) NOT NULL,
                        severity VARCHAR(20) NOT NULL,
                        description TEXT,
                        transaction_id VARCHAR(36),
                        risk_score DOUBLE DEFAULT 0.0,
                        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                        investigated_at DATETIME,
                        resolved_at DATETIME,
                        status VARCHAR(20) DEFAULT 'Open'
                    )",
                )
                .execute(mysql_pool)
                .await?;

                info!("Created MySQL analytics tables");
            }
        }

        Ok(())
    }

    async fn get_total_transactions(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> EconomyResult<u64> {
        let pool = match self.database.get_pool() {
            Ok(p) => p,
            Err(_) => return Ok(0),
        };

        match pool {
            DatabasePoolRef::PostgreSQL(pg_pool) => {
                let row: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM economy_transactions
                     WHERE created_at >= $1 AND created_at <= $2 AND status = 'Completed'",
                )
                .bind(start)
                .bind(end)
                .fetch_one(pg_pool)
                .await
                .unwrap_or((0,));
                Ok(row.0 as u64)
            }
            DatabasePoolRef::MySQL(mysql_pool) => {
                let row: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM economy_transactions
                     WHERE created_at >= ? AND created_at <= ? AND status = 'Completed'",
                )
                .bind(start)
                .bind(end)
                .fetch_one(mysql_pool)
                .await
                .unwrap_or((0,));
                Ok(row.0 as u64)
            }
        }
    }

    async fn get_total_volume(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> EconomyResult<i64> {
        let pool = match self.database.get_pool() {
            Ok(p) => p,
            Err(_) => return Ok(0),
        };

        match pool {
            DatabasePoolRef::PostgreSQL(pg_pool) => {
                let row: (Option<i64>,) = sqlx::query_as(
                    "SELECT COALESCE(SUM(amount), 0) FROM economy_transactions
                     WHERE created_at >= $1 AND created_at <= $2 AND status = 'Completed'",
                )
                .bind(start)
                .bind(end)
                .fetch_one(pg_pool)
                .await
                .unwrap_or((Some(0),));
                Ok(row.0.unwrap_or(0))
            }
            DatabasePoolRef::MySQL(mysql_pool) => {
                let row: (Option<i64>,) = sqlx::query_as(
                    "SELECT COALESCE(SUM(amount), 0) FROM economy_transactions
                     WHERE created_at >= ? AND created_at <= ? AND status = 'Completed'",
                )
                .bind(start)
                .bind(end)
                .fetch_one(mysql_pool)
                .await
                .unwrap_or((Some(0),));
                Ok(row.0.unwrap_or(0))
            }
        }
    }

    async fn get_active_users(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> EconomyResult<u64> {
        let pool = match self.database.get_pool() {
            Ok(p) => p,
            Err(_) => return Ok(0),
        };

        match pool {
            DatabasePoolRef::PostgreSQL(pg_pool) => {
                let row: (i64,) = sqlx::query_as(
                    "SELECT COUNT(DISTINCT user_id) FROM (
                        SELECT from_user_id AS user_id FROM economy_transactions
                        WHERE created_at >= $1 AND created_at <= $2 AND from_user_id IS NOT NULL
                        UNION
                        SELECT to_user_id AS user_id FROM economy_transactions
                        WHERE created_at >= $1 AND created_at <= $2 AND to_user_id IS NOT NULL
                    ) AS users",
                )
                .bind(start)
                .bind(end)
                .fetch_one(pg_pool)
                .await
                .unwrap_or((0,));
                Ok(row.0 as u64)
            }
            DatabasePoolRef::MySQL(mysql_pool) => {
                let row: (i64,) = sqlx::query_as(
                    "SELECT COUNT(DISTINCT user_id) FROM (
                        SELECT from_user_id AS user_id FROM economy_transactions
                        WHERE created_at >= ? AND created_at <= ? AND from_user_id IS NOT NULL
                        UNION
                        SELECT to_user_id AS user_id FROM economy_transactions
                        WHERE created_at >= ? AND created_at <= ? AND to_user_id IS NOT NULL
                    ) AS users",
                )
                .bind(start)
                .bind(end)
                .bind(start)
                .bind(end)
                .fetch_one(mysql_pool)
                .await
                .unwrap_or((0,));
                Ok(row.0 as u64)
            }
        }
    }

    async fn get_marketplace_listings_count(&self) -> EconomyResult<u64> {
        let pool = match self.database.get_pool() {
            Ok(p) => p,
            Err(_) => return Ok(0),
        };

        match pool {
            DatabasePoolRef::PostgreSQL(pg_pool) => {
                let row: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM marketplace_listings WHERE listing_status = 'Active'",
                )
                .fetch_one(pg_pool)
                .await
                .unwrap_or((0,));
                Ok(row.0 as u64)
            }
            DatabasePoolRef::MySQL(mysql_pool) => {
                let row: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM marketplace_listings WHERE listing_status = 'Active'",
                )
                .fetch_one(mysql_pool)
                .await
                .unwrap_or((0,));
                Ok(row.0 as u64)
            }
        }
    }

    async fn get_average_transaction_size(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> EconomyResult<f64> {
        let total_transactions = self.get_total_transactions(start, end).await?;
        if total_transactions == 0 {
            return Ok(0.0);
        }

        let total_volume = self.get_total_volume(start, end).await?;
        Ok(total_volume as f64 / total_transactions as f64)
    }

    async fn get_transaction_velocity(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> EconomyResult<f64> {
        let total_transactions = self.get_total_transactions(start, end).await?;
        let duration_hours = (end - start).num_hours() as f64;

        if duration_hours <= 0.0 {
            return Ok(0.0);
        }

        Ok(total_transactions as f64 / duration_hours)
    }

    async fn get_currency_circulation(&self) -> EconomyResult<HashMap<String, i64>> {
        let pool = match self.database.get_pool() {
            Ok(p) => p,
            Err(_) => return Ok(HashMap::new()),
        };

        let mut circulation = HashMap::new();

        match pool {
            DatabasePoolRef::PostgreSQL(pg_pool) => {
                let rows: Vec<(String, i64)> = sqlx::query_as(
                    "SELECT currency_code, COALESCE(SUM(balance), 0) as total
                     FROM currency_balances GROUP BY currency_code",
                )
                .fetch_all(pg_pool)
                .await
                .unwrap_or_default();

                for (code, total) in rows {
                    circulation.insert(code, total);
                }
            }
            DatabasePoolRef::MySQL(mysql_pool) => {
                let rows: Vec<(String, i64)> = sqlx::query_as(
                    "SELECT currency_code, COALESCE(SUM(balance), 0) as total
                     FROM currency_balances GROUP BY currency_code",
                )
                .fetch_all(mysql_pool)
                .await
                .unwrap_or_default();

                for (code, total) in rows {
                    circulation.insert(code, total);
                }
            }
        }

        if circulation.is_empty() {
            circulation.insert("L$".to_string(), 0);
        }

        Ok(circulation)
    }

    async fn get_top_categories(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> EconomyResult<Vec<CategoryMetrics>> {
        let pool = match self.database.get_pool() {
            Ok(p) => p,
            Err(_) => return Ok(Vec::new()),
        };

        let mut categories = Vec::new();

        match pool {
            DatabasePoolRef::PostgreSQL(pg_pool) => {
                let rows: Vec<(Uuid, String, i64, i64)> = sqlx::query_as(
                    "SELECT ml.category_id, COALESCE(mc.name, 'Uncategorized') as name,
                            COUNT(*) as tx_count, COALESCE(SUM(et.amount), 0) as volume
                     FROM economy_transactions et
                     JOIN marketplace_listings ml ON et.reference_id = ml.listing_id
                     LEFT JOIN marketplace_categories mc ON ml.category_id = mc.category_id
                     WHERE et.created_at >= $1 AND et.created_at <= $2
                           AND et.status = 'Completed' AND et.transaction_type = 'Purchase'
                     GROUP BY ml.category_id, mc.name
                     ORDER BY volume DESC
                     LIMIT 10",
                )
                .bind(start)
                .bind(end)
                .fetch_all(pg_pool)
                .await
                .unwrap_or_default();

                for (cat_id, name, tx_count, volume) in rows {
                    categories.push(CategoryMetrics {
                        category_id: cat_id,
                        category_name: name,
                        transaction_count: tx_count as u64,
                        total_volume: volume,
                        average_price: if tx_count > 0 {
                            volume as f64 / tx_count as f64
                        } else {
                            0.0
                        },
                        top_sellers: Vec::new(),
                    });
                }
            }
            DatabasePoolRef::MySQL(mysql_pool) => {
                let rows: Vec<(String, String, i64, i64)> = sqlx::query_as(
                    "SELECT ml.category_id, COALESCE(mc.name, 'Uncategorized') as name,
                            COUNT(*) as tx_count, COALESCE(SUM(et.amount), 0) as volume
                     FROM economy_transactions et
                     JOIN marketplace_listings ml ON et.reference_id = ml.listing_id
                     LEFT JOIN marketplace_categories mc ON ml.category_id = mc.category_id
                     WHERE et.created_at >= ? AND et.created_at <= ?
                           AND et.status = 'Completed' AND et.transaction_type = 'Purchase'
                     GROUP BY ml.category_id, mc.name
                     ORDER BY volume DESC
                     LIMIT 10",
                )
                .bind(start)
                .bind(end)
                .fetch_all(mysql_pool)
                .await
                .unwrap_or_default();

                for (cat_id_str, name, tx_count, volume) in rows {
                    let cat_id = Uuid::parse_str(&cat_id_str).unwrap_or(Uuid::nil());
                    categories.push(CategoryMetrics {
                        category_id: cat_id,
                        category_name: name,
                        transaction_count: tx_count as u64,
                        total_volume: volume,
                        average_price: if tx_count > 0 {
                            volume as f64 / tx_count as f64
                        } else {
                            0.0
                        },
                        top_sellers: Vec::new(),
                    });
                }
            }
        }

        Ok(categories)
    }

    async fn get_fraud_incidents(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> EconomyResult<u64> {
        let pool = match self.database.get_pool() {
            Ok(p) => p,
            Err(_) => return Ok(0),
        };

        match pool {
            DatabasePoolRef::PostgreSQL(pg_pool) => {
                let row: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM fraud_alerts
                     WHERE created_at >= $1 AND created_at <= $2",
                )
                .bind(start)
                .bind(end)
                .fetch_one(pg_pool)
                .await
                .unwrap_or((0,));
                Ok(row.0 as u64)
            }
            DatabasePoolRef::MySQL(mysql_pool) => {
                let row: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM fraud_alerts
                     WHERE created_at >= ? AND created_at <= ?",
                )
                .bind(start)
                .bind(end)
                .fetch_one(mysql_pool)
                .await
                .unwrap_or((0,));
                Ok(row.0 as u64)
            }
        }
    }

    pub async fn record_transaction(&self, transaction: &Transaction) -> EconomyResult<()> {
        let pool = match self.database.get_pool() {
            Ok(p) => p,
            Err(e) => {
                warn!("Could not record transaction: {}", e);
                return Ok(());
            }
        };

        match pool {
            DatabasePoolRef::PostgreSQL(pg_pool) => {
                sqlx::query(
                    "INSERT INTO economy_transactions
                     (transaction_id, transaction_type, from_user_id, to_user_id, currency_code,
                      amount, fee, description, reference_id, status, created_at, processed_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
                )
                .bind(transaction.transaction_id)
                .bind(format!("{:?}", transaction.transaction_type))
                .bind(transaction.from_user_id)
                .bind(transaction.to_user_id)
                .bind(&transaction.currency_code)
                .bind(transaction.amount)
                .bind(transaction.fee)
                .bind(&transaction.description)
                .bind(transaction.reference_id)
                .bind(format!("{:?}", transaction.status))
                .bind(transaction.created_at)
                .bind(transaction.processed_at)
                .execute(pg_pool)
                .await?;
            }
            DatabasePoolRef::MySQL(mysql_pool) => {
                sqlx::query(
                    "INSERT INTO economy_transactions
                     (transaction_id, transaction_type, from_user_id, to_user_id, currency_code,
                      amount, fee, description, reference_id, status, created_at, processed_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(transaction.transaction_id.to_string())
                .bind(format!("{:?}", transaction.transaction_type))
                .bind(transaction.from_user_id.map(|u| u.to_string()))
                .bind(transaction.to_user_id.map(|u| u.to_string()))
                .bind(&transaction.currency_code)
                .bind(transaction.amount)
                .bind(transaction.fee)
                .bind(&transaction.description)
                .bind(transaction.reference_id.map(|u| u.to_string()))
                .bind(format!("{:?}", transaction.status))
                .bind(transaction.created_at)
                .bind(transaction.processed_at)
                .execute(mysql_pool)
                .await?;
            }
        }

        debug!("Recorded transaction: {}", transaction.transaction_id);
        Ok(())
    }

    pub async fn record_fraud_alert(&self, alert: &FraudAlert) -> EconomyResult<()> {
        let pool = match self.database.get_pool() {
            Ok(p) => p,
            Err(e) => {
                warn!("Could not record fraud alert: {}", e);
                return Ok(());
            }
        };

        match pool {
            DatabasePoolRef::PostgreSQL(pg_pool) => {
                sqlx::query(
                    "INSERT INTO fraud_alerts
                     (alert_id, user_id, alert_type, severity, description, transaction_id,
                      risk_score, created_at, status)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                )
                .bind(alert.alert_id)
                .bind(alert.user_id)
                .bind(format!("{:?}", alert.alert_type))
                .bind(format!("{:?}", alert.severity))
                .bind(&alert.description)
                .bind(alert.transaction_id)
                .bind(alert.risk_score)
                .bind(alert.created_at)
                .bind(format!("{:?}", alert.status))
                .execute(pg_pool)
                .await?;
            }
            DatabasePoolRef::MySQL(mysql_pool) => {
                sqlx::query(
                    "INSERT INTO fraud_alerts
                     (alert_id, user_id, alert_type, severity, description, transaction_id,
                      risk_score, created_at, status)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(alert.alert_id.to_string())
                .bind(alert.user_id.to_string())
                .bind(format!("{:?}", alert.alert_type))
                .bind(format!("{:?}", alert.severity))
                .bind(&alert.description)
                .bind(alert.transaction_id.map(|u| u.to_string()))
                .bind(alert.risk_score)
                .bind(alert.created_at)
                .bind(format!("{:?}", alert.status))
                .execute(mysql_pool)
                .await?;
            }
        }

        info!(
            "Recorded fraud alert: {} for user {}",
            alert.alert_id, alert.user_id
        );
        Ok(())
    }
}
