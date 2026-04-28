//! Content Marketplace Integration for OpenSim Next
//!
//! Provides marketplace functionality for content buying, selling, and management
//! with ownership verification and automated delivery systems.

use super::{ContentError, ContentMetadata, ContentPermissions, ContentPrice, ContentResult};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Marketplace listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceListing {
    pub listing_id: Uuid,
    pub content_id: Uuid,
    pub seller_id: Uuid,
    pub title: String,
    pub description: String,
    pub price: ContentPrice,
    pub category: String,
    pub tags: Vec<String>,
    pub listing_status: ListingStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub featured: bool,
    pub sales_count: u32,
    pub rating: f32,
    pub review_count: u32,
    pub demo_available: bool,
    pub auto_delivery: bool,
}

/// Marketplace listing status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ListingStatus {
    Active,
    Inactive,
    Pending,
    Suspended,
    Deleted,
}

/// Marketplace transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceTransaction {
    pub transaction_id: Uuid,
    pub listing_id: Uuid,
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub content_id: Uuid,
    pub amount: f64,
    pub currency: String,
    pub transaction_status: TransactionStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub payment_method: PaymentMethod,
    pub delivery_status: DeliveryStatus,
    pub refund_eligible: bool,
}

/// Transaction status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Refunded,
    Disputed,
}

/// Payment methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentMethod {
    VirtualCurrency(String),
    CreditCard,
    PayPal,
    Cryptocurrency(String),
    BankTransfer,
}

/// Delivery status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeliveryStatus {
    Pending,
    Delivered,
    Failed,
    Cancelled,
}

/// Marketplace review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceReview {
    pub review_id: Uuid,
    pub listing_id: Uuid,
    pub reviewer_id: Uuid,
    pub rating: u8, // 1-5 stars
    pub title: Option<String>,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
    pub verified_purchase: bool,
    pub helpful_votes: u32,
    pub reported: bool,
}

/// Marketplace search filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSearchFilter {
    pub categories: Option<Vec<String>>,
    pub price_min: Option<f64>,
    pub price_max: Option<f64>,
    pub rating_min: Option<f32>,
    pub seller_ids: Option<Vec<Uuid>>,
    pub featured_only: Option<bool>,
    pub demo_available: Option<bool>,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
}

/// Sort options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortBy {
    Relevance,
    Price,
    Rating,
    Sales,
    Date,
    Popularity,
}

/// Sort order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Marketplace analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceAnalytics {
    pub total_listings: u32,
    pub active_listings: u32,
    pub total_sales: u64,
    pub total_revenue: f64,
    pub currency: String,
    pub top_categories: Vec<CategoryStats>,
    pub top_sellers: Vec<SellerStats>,
    pub sales_by_period: HashMap<String, SalesData>,
}

/// Category statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    pub category: String,
    pub listing_count: u32,
    pub sales_count: u32,
    pub average_price: f64,
    pub average_rating: f32,
}

/// Seller statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellerStats {
    pub seller_id: Uuid,
    pub seller_name: String,
    pub listing_count: u32,
    pub sales_count: u32,
    pub total_revenue: f64,
    pub average_rating: f32,
}

/// Sales data by period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesData {
    pub period: String,
    pub sales_count: u32,
    pub revenue: f64,
    pub unique_buyers: u32,
}

/// Escrow entry for held funds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowEntry {
    pub escrow_id: Uuid,
    pub transaction_id: Uuid,
    pub seller_id: Uuid,
    pub buyer_id: Uuid,
    pub amount: f64,
    pub currency: String,
    pub status: EscrowStatus,
    pub created_at: DateTime<Utc>,
    pub release_at: Option<DateTime<Utc>>,
    pub released_at: Option<DateTime<Utc>>,
}

/// Escrow status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EscrowStatus {
    Held,
    Released,
    Refunded,
    Disputed,
}

/// Payout request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutRequest {
    pub payout_id: Uuid,
    pub seller_id: Uuid,
    pub amount: f64,
    pub currency: String,
    pub status: PayoutStatus,
    pub requested_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub payment_method: PaymentMethod,
    pub reference: Option<String>,
}

/// Payout status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PayoutStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

/// Seller balance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellerBalance {
    pub seller_id: Uuid,
    pub available_balance: f64,
    pub pending_balance: f64,
    pub total_earned: f64,
    pub total_paid_out: f64,
    pub currency: String,
    pub last_updated: DateTime<Utc>,
}

/// Marketplace manager
pub struct MarketplaceManager {
    listings: Arc<RwLock<HashMap<Uuid, MarketplaceListing>>>,
    transactions: Arc<RwLock<HashMap<Uuid, MarketplaceTransaction>>>,
    reviews: Arc<RwLock<HashMap<Uuid, Vec<MarketplaceReview>>>>,
    seller_profiles: Arc<RwLock<HashMap<Uuid, SellerProfile>>>,
    escrow_entries: Arc<RwLock<HashMap<Uuid, EscrowEntry>>>,
    seller_balances: Arc<RwLock<HashMap<Uuid, SellerBalance>>>,
    payout_requests: Arc<RwLock<HashMap<Uuid, PayoutRequest>>>,
    config: MarketplaceConfig,
}

/// Seller profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellerProfile {
    pub seller_id: Uuid,
    pub display_name: String,
    pub description: Option<String>,
    pub verified: bool,
    pub rating: f32,
    pub total_sales: u32,
    pub member_since: DateTime<Utc>,
    pub commission_rate: f32,
    pub payment_info: Option<PaymentInfo>,
}

/// Payment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentInfo {
    pub payment_methods: Vec<PaymentMethod>,
    pub payout_schedule: PayoutSchedule,
    pub tax_info: Option<TaxInfo>,
}

/// Payout schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PayoutSchedule {
    Daily,
    Weekly,
    Monthly,
    OnDemand,
}

/// Tax information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxInfo {
    pub tax_id: String,
    pub tax_rate: f32,
    pub tax_exempt: bool,
}

/// Marketplace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceConfig {
    pub commission_rate: f32,
    pub minimum_price: f64,
    pub maximum_price: f64,
    pub auto_approval: bool,
    pub review_moderation: bool,
    pub escrow_enabled: bool,
    pub refund_period_days: u32,
    pub featured_listing_cost: f64,
}

impl MarketplaceManager {
    pub fn new(config: MarketplaceConfig) -> Self {
        Self {
            listings: Arc::new(RwLock::new(HashMap::new())),
            transactions: Arc::new(RwLock::new(HashMap::new())),
            reviews: Arc::new(RwLock::new(HashMap::new())),
            seller_profiles: Arc::new(RwLock::new(HashMap::new())),
            escrow_entries: Arc::new(RwLock::new(HashMap::new())),
            seller_balances: Arc::new(RwLock::new(HashMap::new())),
            payout_requests: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Create a marketplace listing
    pub async fn create_listing(
        &mut self,
        content_metadata: ContentMetadata,
        seller_id: Uuid,
        price: ContentPrice,
        listing_details: ListingDetails,
    ) -> ContentResult<Uuid> {
        let listing_id = Uuid::new_v4();

        let listing = MarketplaceListing {
            listing_id,
            content_id: content_metadata.content_id,
            seller_id,
            title: listing_details.title,
            description: listing_details.description,
            price,
            category: listing_details.category,
            tags: listing_details.tags,
            listing_status: if self.config.auto_approval {
                ListingStatus::Active
            } else {
                ListingStatus::Pending
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
            featured: false,
            sales_count: 0,
            rating: 0.0,
            review_count: 0,
            demo_available: listing_details.demo_available,
            auto_delivery: listing_details.auto_delivery,
        };

        self.listings.write().await.insert(listing_id, listing);

        tracing::info!(
            "Marketplace listing created: {} by seller {}",
            listing_id,
            seller_id
        );
        Ok(listing_id)
    }

    /// Purchase content from marketplace
    pub async fn purchase_content(
        &mut self,
        listing_id: Uuid,
        buyer_id: Uuid,
        payment_method: PaymentMethod,
    ) -> ContentResult<Uuid> {
        let listing = self
            .listings
            .read()
            .await
            .get(&listing_id)
            .cloned()
            .ok_or(ContentError::ContentNotFound { id: listing_id })?;

        if listing.listing_status != ListingStatus::Active {
            return Err(ContentError::MarketplaceError {
                reason: "Listing is not active".to_string(),
            });
        }

        let transaction_id = Uuid::new_v4();

        let transaction = MarketplaceTransaction {
            transaction_id,
            listing_id,
            buyer_id,
            seller_id: listing.seller_id,
            content_id: listing.content_id,
            amount: listing.price.amount,
            currency: listing.price.currency.clone(),
            transaction_status: TransactionStatus::Processing,
            created_at: Utc::now(),
            completed_at: None,
            payment_method,
            delivery_status: DeliveryStatus::Pending,
            refund_eligible: true,
        };

        self.transactions
            .write()
            .await
            .insert(transaction_id, transaction);

        // Process payment (stub implementation)
        self.process_payment(transaction_id).await?;

        tracing::info!(
            "Content purchase initiated: {} by buyer {}",
            listing_id,
            buyer_id
        );
        Ok(transaction_id)
    }

    /// Search marketplace listings
    pub async fn search_listings(
        &self,
        query: Option<String>,
        filter: MarketplaceSearchFilter,
        page: u32,
        page_size: u32,
    ) -> ContentResult<Vec<MarketplaceListing>> {
        let all_listings = self.listings.read().await;
        let mut results: Vec<MarketplaceListing> = all_listings
            .values()
            .filter(|listing| {
                // Active listings only
                listing.listing_status == ListingStatus::Active &&
                // Apply filters
                self.matches_marketplace_filter(listing, &filter) &&
                // Apply text search
                self.matches_query(listing, &query)
            })
            .cloned()
            .collect();

        // Sort results
        self.sort_listings(&mut results, &filter.sort_by, &filter.sort_order);

        // Paginate
        let start_idx = (page * page_size) as usize;
        let end_idx = ((page + 1) * page_size) as usize;

        if start_idx < results.len() {
            results = results[start_idx..end_idx.min(results.len())].to_vec();
        } else {
            results = Vec::new();
        }

        Ok(results)
    }

    /// Add review for a listing
    pub async fn add_review(
        &mut self,
        listing_id: Uuid,
        reviewer_id: Uuid,
        rating: u8,
        title: Option<String>,
        comment: Option<String>,
    ) -> ContentResult<Uuid> {
        if rating < 1 || rating > 5 {
            return Err(ContentError::ValidationFailed {
                reason: "Rating must be between 1 and 5".to_string(),
            });
        }

        // Verify purchase (simplified)
        let has_purchased = self.verify_purchase(listing_id, reviewer_id).await?;

        let review_id = Uuid::new_v4();
        let review = MarketplaceReview {
            review_id,
            listing_id,
            reviewer_id,
            rating,
            title,
            comment,
            created_at: Utc::now(),
            verified_purchase: has_purchased,
            helpful_votes: 0,
            reported: false,
        };

        // Add review
        self.reviews
            .write()
            .await
            .entry(listing_id)
            .or_default()
            .push(review);

        // Update listing rating
        self.update_listing_rating(listing_id).await?;

        tracing::info!(
            "Review added for listing {} by reviewer {}",
            listing_id,
            reviewer_id
        );
        Ok(review_id)
    }

    /// Get marketplace analytics
    pub async fn get_analytics(&self) -> ContentResult<MarketplaceAnalytics> {
        let listings = self.listings.read().await;
        let transactions = self.transactions.read().await;

        let total_listings = listings.len() as u32;
        let active_listings = listings
            .values()
            .filter(|l| l.listing_status == ListingStatus::Active)
            .count() as u32;

        let total_sales = transactions
            .values()
            .filter(|t| t.transaction_status == TransactionStatus::Completed)
            .count() as u64;

        let total_revenue = transactions
            .values()
            .filter(|t| t.transaction_status == TransactionStatus::Completed)
            .map(|t| t.amount)
            .sum();

        // Calculate category stats
        let mut category_stats = HashMap::new();
        for listing in listings.values() {
            let entry = category_stats
                .entry(listing.category.clone())
                .or_insert(CategoryStats {
                    category: listing.category.clone(),
                    listing_count: 0,
                    sales_count: listing.sales_count,
                    average_price: 0.0,
                    average_rating: 0.0,
                });
            entry.listing_count += 1;
        }

        let top_categories: Vec<CategoryStats> = category_stats.into_values().take(10).collect();

        // Calculate seller stats (simplified)
        let top_sellers = Vec::new(); // Would implement seller ranking

        // Sales by period (simplified)
        let sales_by_period = HashMap::new(); // Would implement time-based aggregation

        Ok(MarketplaceAnalytics {
            total_listings,
            active_listings,
            total_sales,
            total_revenue,
            currency: "USD".to_string(), // Would be configurable
            top_categories,
            top_sellers,
            sales_by_period,
        })
    }

    /// Get seller profile
    pub async fn get_seller_profile(&self, seller_id: Uuid) -> ContentResult<SellerProfile> {
        self.seller_profiles
            .read()
            .await
            .get(&seller_id)
            .cloned()
            .ok_or(ContentError::ContentNotFound { id: seller_id })
    }

    /// Update seller profile
    pub async fn update_seller_profile(
        &mut self,
        seller_id: Uuid,
        profile_data: SellerProfileUpdate,
    ) -> ContentResult<()> {
        let mut profiles = self.seller_profiles.write().await;
        if let Some(profile) = profiles.get_mut(&seller_id) {
            if let Some(display_name) = profile_data.display_name {
                profile.display_name = display_name;
            }
            if let Some(description) = profile_data.description {
                profile.description = Some(description);
            }
            if let Some(payment_info) = profile_data.payment_info {
                profile.payment_info = Some(payment_info);
            }
        }
        Ok(())
    }

    // Private helper methods

    fn matches_marketplace_filter(
        &self,
        listing: &MarketplaceListing,
        filter: &MarketplaceSearchFilter,
    ) -> bool {
        if let Some(categories) = &filter.categories {
            if !categories.contains(&listing.category) {
                return false;
            }
        }

        if let Some(price_min) = filter.price_min {
            if listing.price.amount < price_min {
                return false;
            }
        }

        if let Some(price_max) = filter.price_max {
            if listing.price.amount > price_max {
                return false;
            }
        }

        if let Some(rating_min) = filter.rating_min {
            if listing.rating < rating_min {
                return false;
            }
        }

        if let Some(featured_only) = filter.featured_only {
            if featured_only && !listing.featured {
                return false;
            }
        }

        if let Some(demo_available) = filter.demo_available {
            if demo_available && !listing.demo_available {
                return false;
            }
        }

        true
    }

    fn matches_query(&self, listing: &MarketplaceListing, query: &Option<String>) -> bool {
        if let Some(q) = query {
            let q_lower = q.to_lowercase();
            listing.title.to_lowercase().contains(&q_lower)
                || listing.description.to_lowercase().contains(&q_lower)
                || listing
                    .tags
                    .iter()
                    .any(|tag| tag.to_lowercase().contains(&q_lower))
        } else {
            true
        }
    }

    fn sort_listings(
        &self,
        listings: &mut Vec<MarketplaceListing>,
        sort_by: &SortBy,
        sort_order: &SortOrder,
    ) {
        match sort_by {
            SortBy::Price => {
                listings.sort_by(|a, b| {
                    let cmp = a
                        .price
                        .amount
                        .partial_cmp(&b.price.amount)
                        .unwrap_or(std::cmp::Ordering::Equal);
                    match sort_order {
                        SortOrder::Ascending => cmp,
                        SortOrder::Descending => cmp.reverse(),
                    }
                });
            }
            SortBy::Rating => {
                listings.sort_by(|a, b| {
                    let cmp = a
                        .rating
                        .partial_cmp(&b.rating)
                        .unwrap_or(std::cmp::Ordering::Equal);
                    match sort_order {
                        SortOrder::Ascending => cmp,
                        SortOrder::Descending => cmp.reverse(),
                    }
                });
            }
            SortBy::Sales => {
                listings.sort_by(|a, b| {
                    let cmp = a.sales_count.cmp(&b.sales_count);
                    match sort_order {
                        SortOrder::Ascending => cmp,
                        SortOrder::Descending => cmp.reverse(),
                    }
                });
            }
            SortBy::Date => {
                listings.sort_by(|a, b| {
                    let cmp = a.created_at.cmp(&b.created_at);
                    match sort_order {
                        SortOrder::Ascending => cmp,
                        SortOrder::Descending => cmp.reverse(),
                    }
                });
            }
            _ => {
                // Default sorting by relevance (created date descending)
                listings.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            }
        }
    }

    async fn process_payment(&mut self, transaction_id: Uuid) -> ContentResult<()> {
        let transaction_data = {
            let transactions = self.transactions.read().await;
            transactions.get(&transaction_id).cloned()
        };

        let transaction = match transaction_data {
            Some(t) => t,
            None => {
                error!(
                    "Transaction {} not found for payment processing",
                    transaction_id
                );
                return Err(ContentError::NotFound { id: transaction_id });
            }
        };

        let seller_commission_rate = {
            let profiles = self.seller_profiles.read().await;
            profiles
                .get(&transaction.seller_id)
                .map(|p| p.commission_rate)
                .unwrap_or(self.config.commission_rate)
        };

        let commission = transaction.amount * seller_commission_rate as f64;
        let seller_amount = transaction.amount - commission;

        let payment_valid = self.validate_payment(&transaction).await?;
        if !payment_valid {
            let mut transactions = self.transactions.write().await;
            if let Some(t) = transactions.get_mut(&transaction_id) {
                t.transaction_status = TransactionStatus::Failed;
            }
            warn!(
                "Payment validation failed for transaction {}",
                transaction_id
            );
            return Err(ContentError::MarketplaceError {
                reason: "Payment validation failed".to_string(),
            });
        }

        if self.config.escrow_enabled {
            let escrow_id = Uuid::new_v4();
            let release_date = Utc::now() + Duration::days(self.config.refund_period_days as i64);

            let escrow = EscrowEntry {
                escrow_id,
                transaction_id,
                seller_id: transaction.seller_id,
                buyer_id: transaction.buyer_id,
                amount: seller_amount,
                currency: transaction.currency.clone(),
                status: EscrowStatus::Held,
                created_at: Utc::now(),
                release_at: Some(release_date),
                released_at: None,
            };

            self.escrow_entries.write().await.insert(escrow_id, escrow);

            {
                let mut balances = self.seller_balances.write().await;
                let balance =
                    balances
                        .entry(transaction.seller_id)
                        .or_insert_with(|| SellerBalance {
                            seller_id: transaction.seller_id,
                            available_balance: 0.0,
                            pending_balance: 0.0,
                            total_earned: 0.0,
                            total_paid_out: 0.0,
                            currency: transaction.currency.clone(),
                            last_updated: Utc::now(),
                        });
                balance.pending_balance += seller_amount;
                balance.last_updated = Utc::now();
            }

            info!(
                "Payment {} held in escrow until {}",
                transaction_id, release_date
            );
        } else {
            {
                let mut balances = self.seller_balances.write().await;
                let balance =
                    balances
                        .entry(transaction.seller_id)
                        .or_insert_with(|| SellerBalance {
                            seller_id: transaction.seller_id,
                            available_balance: 0.0,
                            pending_balance: 0.0,
                            total_earned: 0.0,
                            total_paid_out: 0.0,
                            currency: transaction.currency.clone(),
                            last_updated: Utc::now(),
                        });
                balance.available_balance += seller_amount;
                balance.total_earned += seller_amount;
                balance.last_updated = Utc::now();
            }

            info!(
                "Payment {} directly credited to seller {}",
                transaction_id, transaction.seller_id
            );
        }

        {
            let mut transactions = self.transactions.write().await;
            if let Some(t) = transactions.get_mut(&transaction_id) {
                t.transaction_status = TransactionStatus::Completed;
                t.completed_at = Some(Utc::now());
                t.delivery_status = DeliveryStatus::Delivered;
            }
        }

        self.update_listing_sales_count(transaction.listing_id)
            .await?;

        info!(
            "Payment processed: transaction={}, amount={}, commission={}, seller_receives={}",
            transaction_id, transaction.amount, commission, seller_amount
        );

        Ok(())
    }

    async fn validate_payment(&self, transaction: &MarketplaceTransaction) -> ContentResult<bool> {
        if transaction.amount <= 0.0 {
            return Ok(false);
        }

        if transaction.amount < self.config.minimum_price {
            return Ok(false);
        }

        if transaction.amount > self.config.maximum_price {
            return Ok(false);
        }

        match &transaction.payment_method {
            PaymentMethod::VirtualCurrency(currency) => {
                debug!("Validating virtual currency payment: {}", currency);
                Ok(true)
            }
            PaymentMethod::CreditCard => {
                debug!("Validating credit card payment");
                Ok(true)
            }
            PaymentMethod::PayPal => {
                debug!("Validating PayPal payment");
                Ok(true)
            }
            PaymentMethod::Cryptocurrency(crypto) => {
                debug!("Validating cryptocurrency payment: {}", crypto);
                Ok(true)
            }
            PaymentMethod::BankTransfer => {
                debug!("Validating bank transfer payment");
                Ok(true)
            }
        }
    }

    pub async fn process_refund(
        &mut self,
        transaction_id: Uuid,
        reason: &str,
    ) -> ContentResult<()> {
        let transaction_data = {
            let transactions = self.transactions.read().await;
            transactions.get(&transaction_id).cloned()
        };

        let transaction = match transaction_data {
            Some(t) => t,
            None => {
                return Err(ContentError::NotFound { id: transaction_id });
            }
        };

        if transaction.transaction_status != TransactionStatus::Completed {
            return Err(ContentError::MarketplaceError {
                reason: "Can only refund completed transactions".to_string(),
            });
        }

        if !transaction.refund_eligible {
            return Err(ContentError::MarketplaceError {
                reason: "Transaction is not eligible for refund".to_string(),
            });
        }

        let commission_rate = {
            let profiles = self.seller_profiles.read().await;
            profiles
                .get(&transaction.seller_id)
                .map(|p| p.commission_rate)
                .unwrap_or(self.config.commission_rate)
        };

        let commission = transaction.amount * commission_rate as f64;
        let seller_amount = transaction.amount - commission;

        {
            let mut escrow_entries = self.escrow_entries.write().await;
            for (_, escrow) in escrow_entries.iter_mut() {
                if escrow.transaction_id == transaction_id && escrow.status == EscrowStatus::Held {
                    escrow.status = EscrowStatus::Refunded;

                    let mut balances = self.seller_balances.write().await;
                    if let Some(balance) = balances.get_mut(&escrow.seller_id) {
                        balance.pending_balance -= escrow.amount;
                        balance.last_updated = Utc::now();
                    }
                    break;
                }
            }
        }

        {
            let mut balances = self.seller_balances.write().await;
            if let Some(balance) = balances.get_mut(&transaction.seller_id) {
                if balance.available_balance >= seller_amount {
                    balance.available_balance -= seller_amount;
                    balance.total_earned -= seller_amount;
                    balance.last_updated = Utc::now();
                }
            }
        }

        {
            let mut transactions = self.transactions.write().await;
            if let Some(t) = transactions.get_mut(&transaction_id) {
                t.transaction_status = TransactionStatus::Refunded;
                t.refund_eligible = false;
            }
        }

        info!(
            "Refund processed for transaction {}: {}",
            transaction_id, reason
        );
        Ok(())
    }

    pub async fn release_escrow(&mut self, escrow_id: Uuid) -> ContentResult<()> {
        let escrow_data = {
            let escrow_entries = self.escrow_entries.read().await;
            escrow_entries.get(&escrow_id).cloned()
        };

        let escrow = match escrow_data {
            Some(e) => e,
            None => {
                return Err(ContentError::NotFound { id: escrow_id });
            }
        };

        if escrow.status != EscrowStatus::Held {
            return Err(ContentError::MarketplaceError {
                reason: "Escrow is not in held status".to_string(),
            });
        }

        {
            let mut balances = self.seller_balances.write().await;
            if let Some(balance) = balances.get_mut(&escrow.seller_id) {
                balance.pending_balance -= escrow.amount;
                balance.available_balance += escrow.amount;
                balance.total_earned += escrow.amount;
                balance.last_updated = Utc::now();
            }
        }

        {
            let mut escrow_entries = self.escrow_entries.write().await;
            if let Some(e) = escrow_entries.get_mut(&escrow_id) {
                e.status = EscrowStatus::Released;
                e.released_at = Some(Utc::now());
            }
        }

        info!(
            "Escrow {} released: {} {} to seller {}",
            escrow_id, escrow.amount, escrow.currency, escrow.seller_id
        );
        Ok(())
    }

    pub async fn process_pending_escrows(&mut self) -> ContentResult<u32> {
        let now = Utc::now();
        let mut released_count = 0u32;

        let escrow_ids: Vec<Uuid> = {
            let escrow_entries = self.escrow_entries.read().await;
            escrow_entries
                .iter()
                .filter(|(_, e)| {
                    e.status == EscrowStatus::Held
                        && e.release_at.map(|r| r <= now).unwrap_or(false)
                })
                .map(|(id, _)| *id)
                .collect()
        };

        for escrow_id in escrow_ids {
            if self.release_escrow(escrow_id).await.is_ok() {
                released_count += 1;
            }
        }

        if released_count > 0 {
            info!("Released {} pending escrow entries", released_count);
        }

        Ok(released_count)
    }

    pub async fn request_payout(
        &mut self,
        seller_id: Uuid,
        amount: f64,
        payment_method: PaymentMethod,
    ) -> ContentResult<Uuid> {
        let balance = {
            let balances = self.seller_balances.read().await;
            balances.get(&seller_id).cloned()
        };

        let seller_balance = match balance {
            Some(b) => b,
            None => {
                return Err(ContentError::MarketplaceError {
                    reason: "Seller has no balance".to_string(),
                });
            }
        };

        if amount > seller_balance.available_balance {
            return Err(ContentError::MarketplaceError {
                reason: format!(
                    "Insufficient balance. Available: {}",
                    seller_balance.available_balance
                ),
            });
        }

        if amount <= 0.0 {
            return Err(ContentError::MarketplaceError {
                reason: "Payout amount must be positive".to_string(),
            });
        }

        let payout_id = Uuid::new_v4();

        let payout = PayoutRequest {
            payout_id,
            seller_id,
            amount,
            currency: seller_balance.currency.clone(),
            status: PayoutStatus::Pending,
            requested_at: Utc::now(),
            processed_at: None,
            payment_method,
            reference: None,
        };

        {
            let mut balances = self.seller_balances.write().await;
            if let Some(b) = balances.get_mut(&seller_id) {
                b.available_balance -= amount;
                b.last_updated = Utc::now();
            }
        }

        self.payout_requests.write().await.insert(payout_id, payout);

        info!(
            "Payout request created: {} for {} {}",
            payout_id, amount, seller_balance.currency
        );
        Ok(payout_id)
    }

    pub async fn process_payout(&mut self, payout_id: Uuid) -> ContentResult<()> {
        let payout_data = {
            let payouts = self.payout_requests.read().await;
            payouts.get(&payout_id).cloned()
        };

        let payout = match payout_data {
            Some(p) => p,
            None => {
                return Err(ContentError::NotFound { id: payout_id });
            }
        };

        if payout.status != PayoutStatus::Pending {
            return Err(ContentError::MarketplaceError {
                reason: "Payout is not in pending status".to_string(),
            });
        }

        {
            let mut payouts = self.payout_requests.write().await;
            if let Some(p) = payouts.get_mut(&payout_id) {
                p.status = PayoutStatus::Processing;
            }
        }

        let reference = format!(
            "PAYOUT-{}-{}",
            payout.seller_id.to_string()[..8].to_uppercase(),
            Utc::now().format("%Y%m%d")
        );

        {
            let mut payouts = self.payout_requests.write().await;
            if let Some(p) = payouts.get_mut(&payout_id) {
                p.status = PayoutStatus::Completed;
                p.processed_at = Some(Utc::now());
                p.reference = Some(reference.clone());
            }
        }

        {
            let mut balances = self.seller_balances.write().await;
            if let Some(b) = balances.get_mut(&payout.seller_id) {
                b.total_paid_out += payout.amount;
                b.last_updated = Utc::now();
            }
        }

        info!(
            "Payout {} processed: {} {} to seller {}, ref: {}",
            payout_id, payout.amount, payout.currency, payout.seller_id, reference
        );
        Ok(())
    }

    pub async fn get_seller_balance(&self, seller_id: Uuid) -> ContentResult<SellerBalance> {
        let balances = self.seller_balances.read().await;
        balances
            .get(&seller_id)
            .cloned()
            .ok_or(ContentError::NotFound { id: seller_id })
    }

    pub async fn get_seller_transactions(
        &self,
        seller_id: Uuid,
    ) -> ContentResult<Vec<MarketplaceTransaction>> {
        let transactions = self.transactions.read().await;
        let seller_txns: Vec<MarketplaceTransaction> = transactions
            .values()
            .filter(|t| t.seller_id == seller_id)
            .cloned()
            .collect();
        Ok(seller_txns)
    }

    pub async fn get_buyer_transactions(
        &self,
        buyer_id: Uuid,
    ) -> ContentResult<Vec<MarketplaceTransaction>> {
        let transactions = self.transactions.read().await;
        let buyer_txns: Vec<MarketplaceTransaction> = transactions
            .values()
            .filter(|t| t.buyer_id == buyer_id)
            .cloned()
            .collect();
        Ok(buyer_txns)
    }

    pub async fn get_pending_payouts(
        &self,
        seller_id: Option<Uuid>,
    ) -> ContentResult<Vec<PayoutRequest>> {
        let payouts = self.payout_requests.read().await;
        let pending: Vec<PayoutRequest> = payouts
            .values()
            .filter(|p| {
                p.status == PayoutStatus::Pending
                    && seller_id.map(|sid| p.seller_id == sid).unwrap_or(true)
            })
            .cloned()
            .collect();
        Ok(pending)
    }

    async fn verify_purchase(&self, listing_id: Uuid, buyer_id: Uuid) -> ContentResult<bool> {
        let transactions = self.transactions.read().await;
        Ok(transactions.values().any(|t| {
            t.listing_id == listing_id
                && t.buyer_id == buyer_id
                && t.transaction_status == TransactionStatus::Completed
        }))
    }

    async fn update_listing_rating(&mut self, listing_id: Uuid) -> ContentResult<()> {
        let reviews = self.reviews.read().await;
        if let Some(listing_reviews) = reviews.get(&listing_id) {
            if !listing_reviews.is_empty() {
                let total_rating: u32 = listing_reviews.iter().map(|r| r.rating as u32).sum();
                let average_rating = total_rating as f32 / listing_reviews.len() as f32;

                let mut listings = self.listings.write().await;
                if let Some(listing) = listings.get_mut(&listing_id) {
                    listing.rating = average_rating;
                    listing.review_count = listing_reviews.len() as u32;
                }
            }
        }
        Ok(())
    }

    async fn update_listing_sales_count(&mut self, listing_id: Uuid) -> ContentResult<()> {
        let mut listings = self.listings.write().await;
        if let Some(listing) = listings.get_mut(&listing_id) {
            listing.sales_count += 1;
        }
        Ok(())
    }
}

/// Listing creation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListingDetails {
    pub title: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub demo_available: bool,
    pub auto_delivery: bool,
}

/// Seller profile update data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellerProfileUpdate {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub payment_info: Option<PaymentInfo>,
}

impl Default for MarketplaceConfig {
    fn default() -> Self {
        Self {
            commission_rate: 0.05, // 5%
            minimum_price: 0.01,
            maximum_price: 10000.0,
            auto_approval: false,
            review_moderation: true,
            escrow_enabled: true,
            refund_period_days: 30,
            featured_listing_cost: 10.0,
        }
    }
}

impl Default for MarketplaceSearchFilter {
    fn default() -> Self {
        Self {
            categories: None,
            price_min: None,
            price_max: None,
            rating_min: None,
            seller_ids: None,
            featured_only: None,
            demo_available: None,
            sort_by: SortBy::Relevance,
            sort_order: SortOrder::Descending,
        }
    }
}
