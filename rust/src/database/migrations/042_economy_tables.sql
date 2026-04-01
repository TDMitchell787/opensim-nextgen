-- Phase 197: Economy system tables for currency, transactions, marketplace, and fraud detection
-- Supports both local L$ economy and third-party providers (Gloebit)

CREATE TABLE IF NOT EXISTS currency_balances (
    user_id TEXT NOT NULL,
    currency_code TEXT NOT NULL DEFAULT 'LS',
    balance INTEGER NOT NULL DEFAULT 0,
    reserved INTEGER NOT NULL DEFAULT 0,
    available INTEGER NOT NULL DEFAULT 0,
    version INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, currency_code)
);

CREATE INDEX IF NOT EXISTS idx_currency_balances_user ON currency_balances(user_id);

CREATE TABLE IF NOT EXISTS currency_definitions (
    currency_code TEXT NOT NULL PRIMARY KEY,
    currency_name TEXT NOT NULL DEFAULT '',
    currency_symbol TEXT NOT NULL DEFAULT '',
    decimal_places INTEGER NOT NULL DEFAULT 0,
    exchange_rate_to_base REAL NOT NULL DEFAULT 1.0,
    enabled INTEGER NOT NULL DEFAULT 1,
    minimum_balance INTEGER NOT NULL DEFAULT 0,
    maximum_balance INTEGER NOT NULL DEFAULT 999999999
);

CREATE TABLE IF NOT EXISTS economy_transactions (
    id TEXT NOT NULL PRIMARY KEY,
    from_user TEXT NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    to_user TEXT NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    amount INTEGER NOT NULL DEFAULT 0,
    fee INTEGER NOT NULL DEFAULT 0,
    currency_code TEXT NOT NULL DEFAULT 'LS',
    transaction_type TEXT NOT NULL DEFAULT 'Transfer',
    status TEXT NOT NULL DEFAULT 'Pending',
    description TEXT NOT NULL DEFAULT '',
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_economy_transactions_from ON economy_transactions(from_user);
CREATE INDEX IF NOT EXISTS idx_economy_transactions_to ON economy_transactions(to_user);
CREATE INDEX IF NOT EXISTS idx_economy_transactions_created ON economy_transactions(created_at);
CREATE INDEX IF NOT EXISTS idx_economy_transactions_status ON economy_transactions(status);
CREATE INDEX IF NOT EXISTS idx_economy_transactions_type ON economy_transactions(transaction_type);

CREATE TABLE IF NOT EXISTS marketplace_categories (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL DEFAULT '',
    parent_id TEXT NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS marketplace_listings (
    id TEXT NOT NULL PRIMARY KEY,
    seller_id TEXT NOT NULL,
    name TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    price INTEGER NOT NULL DEFAULT 0,
    currency_code TEXT NOT NULL DEFAULT 'LS',
    category_id TEXT NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    tags TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'Active',
    asset_id TEXT NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    created_at INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_marketplace_listings_seller ON marketplace_listings(seller_id);
CREATE INDEX IF NOT EXISTS idx_marketplace_listings_status ON marketplace_listings(status);
CREATE INDEX IF NOT EXISTS idx_marketplace_listings_category ON marketplace_listings(category_id);

CREATE TABLE IF NOT EXISTS purchase_orders (
    id TEXT NOT NULL PRIMARY KEY,
    buyer_id TEXT NOT NULL,
    listing_id TEXT NOT NULL,
    amount INTEGER NOT NULL DEFAULT 0,
    fee INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'Pending',
    created_at INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_purchase_orders_buyer ON purchase_orders(buyer_id);
CREATE INDEX IF NOT EXISTS idx_purchase_orders_listing ON purchase_orders(listing_id);

CREATE TABLE IF NOT EXISTS escrow_accounts (
    id TEXT NOT NULL PRIMARY KEY,
    order_id TEXT NOT NULL,
    amount INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'Held',
    expires_at INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_escrow_accounts_order ON escrow_accounts(order_id);
CREATE INDEX IF NOT EXISTS idx_escrow_accounts_status ON escrow_accounts(status);

CREATE TABLE IF NOT EXISTS fraud_alerts (
    id TEXT NOT NULL PRIMARY KEY,
    user_id TEXT NOT NULL,
    alert_type TEXT NOT NULL DEFAULT 'Unknown',
    severity TEXT NOT NULL DEFAULT 'Low',
    status TEXT NOT NULL DEFAULT 'Open',
    details TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_fraud_alerts_user ON fraud_alerts(user_id);
CREATE INDEX IF NOT EXISTS idx_fraud_alerts_status ON fraud_alerts(status);
CREATE INDEX IF NOT EXISTS idx_fraud_alerts_severity ON fraud_alerts(severity);

CREATE TABLE IF NOT EXISTS gloebit_tokens (
    user_id TEXT NOT NULL PRIMARY KEY,
    access_token TEXT NOT NULL DEFAULT '',
    refresh_token TEXT NOT NULL DEFAULT '',
    token_scope TEXT NOT NULL DEFAULT '',
    expires_at INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT 0
);
