//! Virtual Marketplace for OpenSim Next
//!
//! Provides comprehensive marketplace functionality including item listings,
//! purchase processing, escrow services, and automated delivery.

use super::*;
use crate::database::DatabaseManager;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Virtual marketplace management system
pub struct MarketplaceManager {
    database: Arc<DatabaseManager>,
    transaction_processor: Arc<super::transactions::TransactionProcessor>,
    config: MarketplaceConfig,
    active_listings: Arc<RwLock<HashMap<Uuid, MarketplaceListing>>>,
    pending_orders: Arc<RwLock<HashMap<Uuid, PendingOrder>>>,
    escrow_service: Arc<EscrowService>,
    delivery_service: Arc<DeliveryService>,
}

/// Pending order with additional processing info
#[derive(Debug, Clone)]
struct PendingOrder {
    order: PurchaseOrder,
    escrow_transaction_id: Option<Uuid>,
    delivery_attempts: u32,
    created_at: std::time::Instant,
    expires_at: std::time::Instant,
}

/// Escrow service for secure transactions
pub struct EscrowService {
    database: Arc<DatabaseManager>,
    currency_system: Arc<super::currency::CurrencySystem>,
    escrow_accounts: Arc<RwLock<HashMap<Uuid, EscrowAccount>>>,
}

/// Escrow account holding funds
#[derive(Debug, Clone)]
struct EscrowAccount {
    escrow_id: Uuid,
    order_id: Uuid,
    buyer_id: Uuid,
    seller_id: Uuid,
    amount: i64,
    currency_code: String,
    status: EscrowStatus,
    created_at: chrono::DateTime<chrono::Utc>,
    expires_at: chrono::DateTime<chrono::Utc>,
}

/// Escrow status
#[derive(Debug, Clone)]
enum EscrowStatus {
    Holding,
    Released,
    Refunded,
    Disputed,
}

/// Delivery service for automated item delivery
pub struct DeliveryService {
    database: Arc<DatabaseManager>,
    delivery_methods: Arc<RwLock<HashMap<DeliveryMethod, Box<dyn DeliveryHandler + Send + Sync>>>>,
}

/// Trait for delivery method handlers
#[async_trait::async_trait]
trait DeliveryHandler {
    async fn deliver_item(
        &self,
        order: &PurchaseOrder,
        item: &MarketplaceItem,
    ) -> Result<DeliveryResult>;
    async fn verify_delivery(&self, order: &PurchaseOrder) -> Result<bool>;
}

/// Marketplace item data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceItem {
    pub item_id: Uuid,
    pub seller_id: Uuid,
    pub item_type: ItemType,
    pub name: String,
    pub description: String,
    pub asset_data: Option<Vec<u8>>,
    pub asset_url: Option<String>,
    pub metadata: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Delivery result
#[derive(Debug, Clone)]
pub struct DeliveryResult {
    pub success: bool,
    pub delivery_id: Uuid,
    pub delivery_method: DeliveryMethod,
    pub delivered_at: chrono::DateTime<chrono::Utc>,
    pub tracking_info: Option<String>,
    pub error_message: Option<String>,
}

/// Marketplace search criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSearchCriteria {
    pub query: Option<String>,
    pub category_id: Option<Uuid>,
    pub item_type: Option<ItemType>,
    pub price_min: Option<i64>,
    pub price_max: Option<i64>,
    pub currency_code: Option<String>,
    pub seller_id: Option<Uuid>,
    pub tags: Vec<String>,
    pub featured_only: bool,
    pub sort_by: SortOption,
    pub sort_order: SortOrder,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Sort options for marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOption {
    Price,
    Created,
    Updated,
    Name,
    Popularity,
    Rating,
}

/// Sort order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl MarketplaceManager {
    /// Create new marketplace manager
    pub fn new(
        database: Arc<DatabaseManager>,
        transaction_processor: Arc<super::transactions::TransactionProcessor>,
        currency_system: Arc<super::currency::CurrencySystem>,
        config: MarketplaceConfig,
    ) -> Self {
        let escrow_service = Arc::new(EscrowService::new(database.clone(), currency_system));
        let delivery_service = Arc::new(DeliveryService::new(database.clone()));

        Self {
            database,
            transaction_processor,
            config,
            active_listings: Arc::new(RwLock::new(HashMap::new())),
            pending_orders: Arc::new(RwLock::new(HashMap::new())),
            escrow_service,
            delivery_service,
        }
    }

    /// Initialize marketplace
    pub async fn initialize(&self) -> EconomyResult<()> {
        info!("Initializing marketplace system");

        if !self.config.enabled {
            warn!("Marketplace is disabled in configuration");
            return Ok(());
        }

        // Create database tables
        self.create_tables().await?;

        // Initialize delivery service
        self.delivery_service.initialize().await?;

        // Load active listings
        self.load_active_listings().await?;

        info!("Marketplace system initialized successfully");
        Ok(())
    }

    /// Create new marketplace listing
    pub async fn create_listing(
        &self,
        seller_id: Uuid,
        item: MarketplaceItem,
        listing_data: CreateListingRequest,
    ) -> EconomyResult<MarketplaceListing> {
        info!(
            "Creating marketplace listing for seller {} item {}",
            seller_id, item.item_id
        );

        // Validate listing
        self.validate_listing_request(&listing_data).await?;

        // Check listing fee
        if self.config.listing_fee > 0 {
            self.charge_listing_fee(seller_id, &listing_data.currency_code)
                .await?;
        }

        let listing = MarketplaceListing {
            listing_id: Uuid::new_v4(),
            seller_id,
            item_id: item.item_id,
            item_type: listing_data.item_type,
            category_id: listing_data.category_id,
            title: listing_data.title,
            description: listing_data.description,
            price: listing_data.price,
            currency_code: listing_data.currency_code,
            quantity_available: listing_data.quantity,
            quantity_sold: 0,
            images: listing_data.images,
            tags: listing_data.tags,
            permissions: listing_data.permissions,
            listing_status: ListingStatus::Active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            expires_at: listing_data.expires_at,
            featured: listing_data.featured,
            auto_delivery: listing_data.auto_delivery,
        };

        // Save to database
        self.save_listing(&listing).await?;
        self.save_item(&item).await?;

        // Add to active listings cache
        {
            let mut active = self.active_listings.write().await;
            active.insert(listing.listing_id, listing.clone());
        }

        info!(
            "Marketplace listing created successfully: {}",
            listing.listing_id
        );
        Ok(listing)
    }

    /// Purchase item from marketplace
    pub async fn purchase_item(
        &self,
        buyer_id: Uuid,
        listing_id: Uuid,
        quantity: u32,
        payment_method: PaymentMethod,
    ) -> EconomyResult<PurchaseOrder> {
        info!(
            "Processing purchase: buyer {} listing {} quantity {}",
            buyer_id, listing_id, quantity
        );

        // Get listing
        let listing = self.get_listing(listing_id).await?;

        // Validate purchase
        self.validate_purchase(&listing, buyer_id, quantity).await?;

        // Calculate total cost
        let unit_price = listing.price;
        let subtotal = unit_price * quantity as i64;
        let fees = self.calculate_marketplace_fees(subtotal).await?;
        let total_price = subtotal + fees;

        // Create purchase order
        let order = PurchaseOrder {
            order_id: Uuid::new_v4(),
            buyer_id,
            seller_id: listing.seller_id,
            listing_id,
            item_id: listing.item_id,
            quantity,
            unit_price,
            total_price,
            fees,
            currency_code: listing.currency_code.clone(),
            order_status: OrderStatus::Created,
            payment_method: payment_method.clone(),
            delivery_method: if listing.auto_delivery {
                DeliveryMethod::Automatic
            } else {
                DeliveryMethod::Manual
            },
            created_at: chrono::Utc::now(),
            paid_at: None,
            delivered_at: None,
            notes: None,
        };

        // Process payment based on method
        match &payment_method {
            PaymentMethod::VirtualCurrency => {
                self.process_virtual_currency_payment(&order).await?;
            }
            PaymentMethod::Escrow => {
                self.process_escrow_payment(&order).await?;
            }
            _ => {
                return Err(EconomyError::TransactionFailed {
                    reason: "Payment method not implemented".to_string(),
                });
            }
        }

        // Save order
        self.save_order(&order).await?;

        // Update listing quantities
        self.update_listing_quantities(listing_id, quantity).await?;

        info!("Purchase processed successfully: order {}", order.order_id);
        Ok(order)
    }

    /// Search marketplace listings
    pub async fn search_listings(
        &self,
        criteria: MarketplaceSearchCriteria,
    ) -> EconomyResult<Vec<MarketplaceListing>> {
        debug!("Searching marketplace with criteria: {:?}", criteria);

        // Implementation would query database with filters
        // For now, return filtered active listings
        let active_listings = self.active_listings.read().await;
        let mut results: Vec<MarketplaceListing> = active_listings.values().cloned().collect();

        // Apply basic filters
        if let Some(category_id) = criteria.category_id {
            results.retain(|l| l.category_id == category_id);
        }

        if let Some(item_type) = criteria.item_type {
            results.retain(|l| l.item_type == item_type);
        }

        if let Some(price_min) = criteria.price_min {
            results.retain(|l| l.price >= price_min);
        }

        if let Some(price_max) = criteria.price_max {
            results.retain(|l| l.price <= price_max);
        }

        if criteria.featured_only {
            results.retain(|l| l.featured);
        }

        // Apply sorting
        match criteria.sort_by {
            SortOption::Price => {
                results.sort_by(|a, b| match criteria.sort_order {
                    SortOrder::Ascending => a.price.cmp(&b.price),
                    SortOrder::Descending => b.price.cmp(&a.price),
                });
            }
            SortOption::Created => {
                results.sort_by(|a, b| match criteria.sort_order {
                    SortOrder::Ascending => a.created_at.cmp(&b.created_at),
                    SortOrder::Descending => b.created_at.cmp(&a.created_at),
                });
            }
            SortOption::Name => {
                results.sort_by(|a, b| match criteria.sort_order {
                    SortOrder::Ascending => a.title.cmp(&b.title),
                    SortOrder::Descending => b.title.cmp(&a.title),
                });
            }
            _ => {} // Other sort options not implemented
        }

        // Apply pagination
        if let Some(offset) = criteria.offset {
            if offset as usize >= results.len() {
                results.clear();
            } else {
                results = results.into_iter().skip(offset as usize).collect();
            }
        }

        if let Some(limit) = criteria.limit {
            results.truncate(limit as usize);
        }

        debug!("Search returned {} results", results.len());
        Ok(results)
    }

    /// Get listing by ID
    pub async fn get_listing(&self, listing_id: Uuid) -> EconomyResult<MarketplaceListing> {
        // Check cache first
        {
            let active = self.active_listings.read().await;
            if let Some(listing) = active.get(&listing_id) {
                return Ok(listing.clone());
            }
        }

        // Load from database
        let listing = self.load_listing_from_db(listing_id).await?;

        // Update cache if active
        if matches!(listing.listing_status, ListingStatus::Active) {
            let mut active = self.active_listings.write().await;
            active.insert(listing_id, listing.clone());
        }

        Ok(listing)
    }

    /// Update listing
    pub async fn update_listing(
        &self,
        listing_id: Uuid,
        seller_id: Uuid,
        updates: UpdateListingRequest,
    ) -> EconomyResult<MarketplaceListing> {
        info!("Updating listing {} by seller {}", listing_id, seller_id);

        let mut listing = self.get_listing(listing_id).await?;

        // Verify seller ownership
        if listing.seller_id != seller_id {
            return Err(EconomyError::AccessDenied {
                reason: "Only the seller can update their listing".to_string(),
            });
        }

        // Apply updates
        if let Some(title) = updates.title {
            listing.title = title;
        }
        if let Some(description) = updates.description {
            listing.description = description;
        }
        if let Some(price) = updates.price {
            listing.price = price;
        }
        if let Some(quantity) = updates.quantity_available {
            listing.quantity_available = quantity;
        }
        if let Some(images) = updates.images {
            listing.images = images;
        }
        if let Some(tags) = updates.tags {
            listing.tags = tags;
        }
        if let Some(featured) = updates.featured {
            listing.featured = featured;
        }

        listing.updated_at = chrono::Utc::now();

        // Save to database
        self.save_listing(&listing).await?;

        // Update cache
        {
            let mut active = self.active_listings.write().await;
            active.insert(listing_id, listing.clone());
        }

        info!("Listing updated successfully: {}", listing_id);
        Ok(listing)
    }

    /// Remove listing
    pub async fn remove_listing(&self, listing_id: Uuid, seller_id: Uuid) -> EconomyResult<()> {
        info!("Removing listing {} by seller {}", listing_id, seller_id);

        let mut listing = self.get_listing(listing_id).await?;

        // Verify seller ownership
        if listing.seller_id != seller_id {
            return Err(EconomyError::AccessDenied {
                reason: "Only the seller can remove their listing".to_string(),
            });
        }

        // Update status
        listing.listing_status = ListingStatus::Removed;
        listing.updated_at = chrono::Utc::now();

        // Save to database
        self.save_listing(&listing).await?;

        // Remove from cache
        {
            let mut active = self.active_listings.write().await;
            active.remove(&listing_id);
        }

        info!("Listing removed successfully: {}", listing_id);
        Ok(())
    }

    /// Get seller's listings
    pub async fn get_seller_listings(
        &self,
        seller_id: Uuid,
    ) -> EconomyResult<Vec<MarketplaceListing>> {
        debug!("Getting listings for seller {}", seller_id);

        // Implementation would query database
        // For now, filter active listings
        let active_listings = self.active_listings.read().await;
        let seller_listings: Vec<MarketplaceListing> = active_listings
            .values()
            .filter(|l| l.seller_id == seller_id)
            .cloned()
            .collect();

        debug!(
            "Found {} listings for seller {}",
            seller_listings.len(),
            seller_id
        );
        Ok(seller_listings)
    }

    /// Get order by ID
    pub async fn get_order(&self, order_id: Uuid) -> EconomyResult<PurchaseOrder> {
        debug!("Getting order {}", order_id);

        // Check pending orders first
        {
            let pending = self.pending_orders.read().await;
            if let Some(pending_order) = pending.get(&order_id) {
                return Ok(pending_order.order.clone());
            }
        }

        // Load from database
        self.load_order_from_db(order_id).await
    }

    /// Get marketplace statistics
    pub async fn get_marketplace_statistics(&self) -> EconomyResult<MarketplaceStatistics> {
        debug!("Generating marketplace statistics");

        let stats = MarketplaceStatistics {
            total_listings: self.get_total_listings_count().await?,
            active_listings: self.active_listings.read().await.len() as u64,
            total_sales: self.get_total_sales_count().await?,
            total_volume: self.get_total_sales_volume().await?,
            unique_sellers: self.get_unique_sellers_count().await?,
            unique_buyers: self.get_unique_buyers_count().await?,
            average_sale_price: self.get_average_sale_price().await?,
            top_categories: self.get_top_categories().await?,
            generated_at: chrono::Utc::now(),
        };

        Ok(stats)
    }

    // Private helper methods

    async fn validate_listing_request(&self, request: &CreateListingRequest) -> EconomyResult<()> {
        if request.title.is_empty() {
            return Err(EconomyError::TransactionFailed {
                reason: "Listing title cannot be empty".to_string(),
            });
        }

        if request.price <= 0 {
            return Err(EconomyError::TransactionFailed {
                reason: "Listing price must be positive".to_string(),
            });
        }

        if request.quantity == 0 {
            return Err(EconomyError::TransactionFailed {
                reason: "Listing quantity must be at least 1".to_string(),
            });
        }

        Ok(())
    }

    async fn validate_purchase(
        &self,
        listing: &MarketplaceListing,
        buyer_id: Uuid,
        quantity: u32,
    ) -> EconomyResult<()> {
        if listing.seller_id == buyer_id {
            return Err(EconomyError::TransactionFailed {
                reason: "Cannot purchase your own item".to_string(),
            });
        }

        if !matches!(listing.listing_status, ListingStatus::Active) {
            return Err(EconomyError::TransactionFailed {
                reason: "Listing is not active".to_string(),
            });
        }

        if quantity > listing.quantity_available {
            return Err(EconomyError::TransactionFailed {
                reason: format!("Only {} items available", listing.quantity_available),
            });
        }

        Ok(())
    }

    async fn charge_listing_fee(&self, seller_id: Uuid, currency_code: &str) -> EconomyResult<()> {
        // Implementation would charge listing fee from seller's balance
        Ok(())
    }

    async fn calculate_marketplace_fees(&self, subtotal: i64) -> EconomyResult<i64> {
        let commission = (subtotal as f64 * self.config.commission_rate) as i64;
        Ok(commission)
    }

    async fn process_virtual_currency_payment(&self, order: &PurchaseOrder) -> EconomyResult<()> {
        // Create transfer transaction
        let transfer_request = super::transactions::TransactionRequest {
            transaction_type: TransactionType::Purchase,
            from_user_id: Some(order.buyer_id),
            to_user_id: Some(order.seller_id),
            currency_code: order.currency_code.clone(),
            amount: order.total_price,
            description: format!("Marketplace purchase: order {}", order.order_id),
            reference_id: Some(order.order_id),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("order_id".to_string(), order.order_id.to_string());
                meta.insert("listing_id".to_string(), order.listing_id.to_string());
                meta
            },
            idempotency_key: Some(format!("marketplace-{}", order.order_id)),
        };

        let _result = self
            .transaction_processor
            .process_transaction(transfer_request)
            .await?;
        Ok(())
    }

    async fn process_escrow_payment(&self, order: &PurchaseOrder) -> EconomyResult<()> {
        self.escrow_service.create_escrow(order).await
    }

    // Database operations (simplified implementations)

    async fn create_tables(&self) -> EconomyResult<()> {
        Ok(())
    }

    async fn load_active_listings(&self) -> EconomyResult<()> {
        Ok(())
    }

    async fn save_listing(&self, _listing: &MarketplaceListing) -> EconomyResult<()> {
        Ok(())
    }

    async fn save_item(&self, _item: &MarketplaceItem) -> EconomyResult<()> {
        Ok(())
    }

    async fn save_order(&self, _order: &PurchaseOrder) -> EconomyResult<()> {
        Ok(())
    }

    async fn load_listing_from_db(&self, listing_id: Uuid) -> EconomyResult<MarketplaceListing> {
        Err(EconomyError::ListingNotFound { listing_id })
    }

    async fn load_order_from_db(&self, order_id: Uuid) -> EconomyResult<PurchaseOrder> {
        Err(EconomyError::TransactionFailed {
            reason: "Order not found".to_string(),
        })
    }

    async fn update_listing_quantities(
        &self,
        _listing_id: Uuid,
        _quantity: u32,
    ) -> EconomyResult<()> {
        Ok(())
    }

    // Statistics methods

    async fn get_total_listings_count(&self) -> EconomyResult<u64> {
        let active = self.active_listings.read().await;
        Ok(active.len() as u64)
    }

    async fn get_total_sales_count(&self) -> EconomyResult<u64> {
        let active = self.active_listings.read().await;
        let total_sold: u32 = active.values().map(|l| l.quantity_sold).sum();
        Ok(total_sold as u64)
    }

    async fn get_total_sales_volume(&self) -> EconomyResult<i64> {
        let active = self.active_listings.read().await;
        let volume: i64 = active
            .values()
            .map(|l| l.price * l.quantity_sold as i64)
            .sum();
        Ok(volume)
    }

    async fn get_unique_sellers_count(&self) -> EconomyResult<u64> {
        let active = self.active_listings.read().await;
        let unique_sellers: std::collections::HashSet<Uuid> =
            active.values().map(|l| l.seller_id).collect();
        Ok(unique_sellers.len() as u64)
    }

    async fn get_unique_buyers_count(&self) -> EconomyResult<u64> {
        let pending = self.pending_orders.read().await;
        let unique_buyers: std::collections::HashSet<Uuid> =
            pending.values().map(|p| p.order.buyer_id).collect();
        Ok(unique_buyers.len() as u64)
    }

    async fn get_average_sale_price(&self) -> EconomyResult<f64> {
        let active = self.active_listings.read().await;
        let listings_with_sales: Vec<&MarketplaceListing> =
            active.values().filter(|l| l.quantity_sold > 0).collect();

        if listings_with_sales.is_empty() {
            return Ok(0.0);
        }

        let total_price: i64 = listings_with_sales.iter().map(|l| l.price).sum();
        Ok(total_price as f64 / listings_with_sales.len() as f64)
    }

    async fn get_top_categories(&self) -> EconomyResult<Vec<CategoryMetrics>> {
        let active = self.active_listings.read().await;
        let mut category_stats: HashMap<Uuid, (u64, i64, Vec<Uuid>)> = HashMap::new();

        for listing in active.values() {
            let entry = category_stats
                .entry(listing.category_id)
                .or_insert((0, 0, Vec::new()));
            entry.0 += listing.quantity_sold as u64;
            entry.1 += listing.price * listing.quantity_sold as i64;
            if !entry.2.contains(&listing.seller_id) {
                entry.2.push(listing.seller_id);
            }
        }

        let mut categories: Vec<CategoryMetrics> = category_stats
            .into_iter()
            .map(
                |(category_id, (transaction_count, total_volume, sellers))| {
                    let average_price = if transaction_count > 0 {
                        total_volume as f64 / transaction_count as f64
                    } else {
                        0.0
                    };
                    let top_sellers: Vec<Uuid> = sellers.into_iter().take(5).collect();
                    CategoryMetrics {
                        category_id,
                        category_name: format!("Category-{}", &category_id.to_string()[..8]),
                        transaction_count,
                        total_volume,
                        average_price,
                        top_sellers,
                    }
                },
            )
            .collect();

        categories.sort_by(|a, b| b.total_volume.cmp(&a.total_volume));
        categories.truncate(10);

        Ok(categories)
    }
}

impl EscrowService {
    fn new(
        database: Arc<DatabaseManager>,
        currency_system: Arc<super::currency::CurrencySystem>,
    ) -> Self {
        Self {
            database,
            currency_system,
            escrow_accounts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn create_escrow(&self, order: &PurchaseOrder) -> EconomyResult<()> {
        info!("Creating escrow for order {}", order.order_id);

        // Reserve funds from buyer
        self.currency_system
            .reserve_currency(order.buyer_id, &order.currency_code, order.total_price)
            .await?;

        // Create escrow account
        let escrow = EscrowAccount {
            escrow_id: Uuid::new_v4(),
            order_id: order.order_id,
            buyer_id: order.buyer_id,
            seller_id: order.seller_id,
            amount: order.total_price,
            currency_code: order.currency_code.clone(),
            status: EscrowStatus::Holding,
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::days(30),
        };

        // Save escrow account
        {
            let mut accounts = self.escrow_accounts.write().await;
            accounts.insert(escrow.escrow_id, escrow);
        }

        info!("Escrow created successfully for order {}", order.order_id);
        Ok(())
    }
}

impl DeliveryService {
    fn new(database: Arc<DatabaseManager>) -> Self {
        Self {
            database,
            delivery_methods: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn initialize(&self) -> EconomyResult<()> {
        // Initialize delivery method handlers
        Ok(())
    }
}

/// Request to create new listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateListingRequest {
    pub item_type: ItemType,
    pub category_id: Uuid,
    pub title: String,
    pub description: String,
    pub price: i64,
    pub currency_code: String,
    pub quantity: u32,
    pub images: Vec<String>,
    pub tags: Vec<String>,
    pub permissions: ItemPermissions,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub featured: bool,
    pub auto_delivery: bool,
}

/// Request to update existing listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateListingRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub price: Option<i64>,
    pub quantity_available: Option<u32>,
    pub images: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub featured: Option<bool>,
}

/// Marketplace statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceStatistics {
    pub total_listings: u64,
    pub active_listings: u64,
    pub total_sales: u64,
    pub total_volume: i64,
    pub unique_sellers: u64,
    pub unique_buyers: u64,
    pub average_sale_price: f64,
    pub top_categories: Vec<CategoryMetrics>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

impl Default for SortOption {
    fn default() -> Self {
        Self::Created
    }
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Descending
    }
}

impl Default for MarketplaceSearchCriteria {
    fn default() -> Self {
        Self {
            query: None,
            category_id: None,
            item_type: None,
            price_min: None,
            price_max: None,
            currency_code: None,
            seller_id: None,
            tags: Vec::new(),
            featured_only: false,
            sort_by: SortOption::default(),
            sort_order: SortOrder::default(),
            limit: Some(20),
            offset: None,
        }
    }
}
