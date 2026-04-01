//! Virtual Economy System for OpenSim Next
//! 
//! This module provides comprehensive virtual economy management including:
//! - Multi-currency virtual currency system
//! - Secure transaction processing and validation
//! - Marketplace integration with automated payments
//! - Economic analytics and fraud detection

pub mod currency;
pub mod transactions;
pub mod marketplace;
pub mod analytics;
pub mod manager;

pub use currency::*;
pub use transactions::*;
pub use marketplace::*;
pub use analytics::*;
pub use manager::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Virtual economy manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyConfig {
    pub default_currency: String,
    pub supported_currencies: Vec<CurrencyDefinition>,
    pub transaction_limits: TransactionLimits,
    pub marketplace_config: MarketplaceConfig,
    pub analytics_config: AnalyticsConfig,
    pub fraud_detection: FraudDetectionConfig,
}

/// Currency definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyDefinition {
    pub currency_code: String,
    pub currency_name: String,
    pub currency_symbol: String,
    pub decimal_places: u8,
    pub exchange_rate_to_base: f64,
    pub enabled: bool,
    pub minimum_balance: i64,
    pub maximum_balance: i64,
}

/// Transaction limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLimits {
    pub max_transaction_amount: i64,
    pub daily_limit: i64,
    pub weekly_limit: i64,
    pub monthly_limit: i64,
    pub minimum_transaction: i64,
    pub rate_limit_per_minute: u32,
    pub require_verification_above: i64,
}

/// Marketplace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceConfig {
    pub enabled: bool,
    pub commission_rate: f64,
    pub listing_fee: i64,
    pub auto_delivery: bool,
    pub escrow_enabled: bool,
    pub dispute_resolution: bool,
    pub categories: Vec<MarketplaceCategory>,
}

/// Marketplace category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceCategory {
    pub category_id: Uuid,
    pub name: String,
    pub description: String,
    pub parent_category: Option<Uuid>,
    pub commission_rate: Option<f64>,
    pub allowed_item_types: Vec<ItemType>,
}

/// Item types for marketplace
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ItemType {
    Avatar,
    Clothing,
    Accessories,
    Furniture,
    Buildings,
    Vehicles,
    Animations,
    Gestures,
    Sounds,
    Textures,
    Scripts,
    Land,
    Services,
    Digital,
}

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    pub enabled: bool,
    pub retention_days: u32,
    pub real_time_tracking: bool,
    pub export_enabled: bool,
    pub dashboard_access: Vec<Uuid>, // User IDs with access
}

/// Fraud detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudDetectionConfig {
    pub enabled: bool,
    pub velocity_checks: bool,
    pub pattern_analysis: bool,
    pub blacklist_enabled: bool,
    pub suspicious_activity_threshold: f64,
    pub auto_freeze_enabled: bool,
    pub notification_enabled: bool,
}

/// Virtual currency balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyBalance {
    pub user_id: Uuid,
    pub currency_code: String,
    pub balance: i64, // Stored as smallest unit (e.g., cents)
    pub reserved: i64, // Amount reserved for pending transactions
    pub available: i64, // balance - reserved
    pub last_updated: DateTime<Utc>,
    pub version: i64, // For optimistic locking
}

/// Transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_id: Uuid,
    pub transaction_type: TransactionType,
    pub from_user_id: Option<Uuid>,
    pub to_user_id: Option<Uuid>,
    pub currency_code: String,
    pub amount: i64,
    pub fee: i64,
    pub description: String,
    pub reference_id: Option<Uuid>,
    pub status: TransactionStatus,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

/// Transaction types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,
    Purchase,
    Sale,
    Deposit,
    Withdrawal,
    Fee,
    Commission,
    Refund,
    Reward,
    Penalty,
    Exchange,
    Escrow,
    Release,
    System,
}

/// Transaction status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
    Reversed,
    Disputed,
    OnHold,
}

/// Marketplace listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceListing {
    pub listing_id: Uuid,
    pub seller_id: Uuid,
    pub item_id: Uuid,
    pub item_type: ItemType,
    pub category_id: Uuid,
    pub title: String,
    pub description: String,
    pub price: i64,
    pub currency_code: String,
    pub quantity_available: u32,
    pub quantity_sold: u32,
    pub images: Vec<String>,
    pub tags: Vec<String>,
    pub permissions: ItemPermissions,
    pub listing_status: ListingStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub featured: bool,
    pub auto_delivery: bool,
}

/// Item permissions for marketplace items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemPermissions {
    pub copy: bool,
    pub modify: bool,
    pub transfer: bool,
    pub resell: bool,
    pub give_away: bool,
}

/// Marketplace listing status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ListingStatus {
    Draft,
    Active,
    Sold,
    Expired,
    Suspended,
    Removed,
}

/// Purchase order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrder {
    pub order_id: Uuid,
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub listing_id: Uuid,
    pub item_id: Uuid,
    pub quantity: u32,
    pub unit_price: i64,
    pub total_price: i64,
    pub fees: i64,
    pub currency_code: String,
    pub order_status: OrderStatus,
    pub payment_method: PaymentMethod,
    pub delivery_method: DeliveryMethod,
    pub created_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

/// Order status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    Created,
    PaymentPending,
    Paid,
    Processing,
    Delivered,
    Completed,
    Cancelled,
    Refunded,
    Disputed,
}

/// Payment method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentMethod {
    VirtualCurrency,
    Escrow,
    InstantTransfer,
    CreditCard,
    PayPal,
    Cryptocurrency,
}

/// Delivery method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryMethod {
    Automatic,
    Manual,
    Download,
    InWorld,
    Email,
}

/// Economic analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicMetrics {
    pub total_transactions: u64,
    pub total_volume: i64,
    pub active_users: u64,
    pub marketplace_listings: u64,
    pub average_transaction_size: f64,
    pub transaction_velocity: f64,
    pub currency_circulation: HashMap<String, i64>,
    pub top_categories: Vec<CategoryMetrics>,
    pub fraud_incidents: u64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

/// Category metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryMetrics {
    pub category_id: Uuid,
    pub category_name: String,
    pub transaction_count: u64,
    pub total_volume: i64,
    pub average_price: f64,
    pub top_sellers: Vec<Uuid>,
}

/// Fraud detection alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudAlert {
    pub alert_id: Uuid,
    pub user_id: Uuid,
    pub alert_type: FraudAlertType,
    pub severity: AlertSeverity,
    pub description: String,
    pub transaction_id: Option<Uuid>,
    pub risk_score: f64,
    pub created_at: DateTime<Utc>,
    pub investigated_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub status: AlertStatus,
}

/// Fraud alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FraudAlertType {
    VelocityAnomaly,
    PatternAnomaly,
    BlacklistMatch,
    SuspiciousActivity,
    LargeTransaction,
    AccountTakeover,
    Chargeback,
    DuplicateTransaction,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Alert status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertStatus {
    Open,
    Investigating,
    Resolved,
    FalsePositive,
    Escalated,
}

/// Economic error types
#[derive(Debug, thiserror::Error)]
pub enum EconomyError {
    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: i64, available: i64 },
    
    #[error("Invalid currency: {currency}")]
    InvalidCurrency { currency: String },
    
    #[error("Transaction limit exceeded: {limit}")]
    TransactionLimitExceeded { limit: i64 },
    
    #[error("User not found: {user_id}")]
    UserNotFound { user_id: Uuid },
    
    #[error("Marketplace listing not found: {listing_id}")]
    ListingNotFound { listing_id: Uuid },
    
    #[error("Transaction failed: {reason}")]
    TransactionFailed { reason: String },
    
    #[error("Fraud detected: {reason}")]
    FraudDetected { reason: String },
    
    #[error("Access denied: {reason}")]
    AccessDenied { reason: String },
    
    #[error("Economy system error: {message}")]
    SystemError { message: String },
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Database connection error: {0}")]
    ConnectionError(#[from] anyhow::Error),
}

/// Result type for economy operations
pub type EconomyResult<T> = Result<T, EconomyError>;

impl Default for EconomyConfig {
    fn default() -> Self {
        Self {
            default_currency: "L$".to_string(),
            supported_currencies: vec![
                CurrencyDefinition {
                    currency_code: "L$".to_string(),
                    currency_name: "Linden Dollars".to_string(),
                    currency_symbol: "L$".to_string(),
                    decimal_places: 2,
                    exchange_rate_to_base: 1.0,
                    enabled: true,
                    minimum_balance: 0,
                    maximum_balance: 999999999,
                }
            ],
            transaction_limits: TransactionLimits::default(),
            marketplace_config: MarketplaceConfig::default(),
            analytics_config: AnalyticsConfig::default(),
            fraud_detection: FraudDetectionConfig::default(),
        }
    }
}

impl Default for TransactionLimits {
    fn default() -> Self {
        Self {
            max_transaction_amount: 100000, // L$1000.00
            daily_limit: 500000, // L$5000.00
            weekly_limit: 2000000, // L$20000.00
            monthly_limit: 5000000, // L$50000.00
            minimum_transaction: 1, // L$0.01
            rate_limit_per_minute: 30,
            require_verification_above: 10000, // L$100.00
        }
    }
}

impl Default for MarketplaceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            commission_rate: 0.05, // 5%
            listing_fee: 10, // L$0.10
            auto_delivery: true,
            escrow_enabled: true,
            dispute_resolution: true,
            categories: Vec::new(),
        }
    }
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_days: 365,
            real_time_tracking: true,
            export_enabled: true,
            dashboard_access: Vec::new(),
        }
    }
}

impl Default for FraudDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            velocity_checks: true,
            pattern_analysis: true,
            blacklist_enabled: true,
            suspicious_activity_threshold: 0.8,
            auto_freeze_enabled: false,
            notification_enabled: true,
        }
    }
}

impl Default for ItemPermissions {
    fn default() -> Self {
        Self {
            copy: true,
            modify: true,
            transfer: true,
            resell: false,
            give_away: true,
        }
    }
}