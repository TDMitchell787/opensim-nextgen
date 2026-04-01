-- Phase 197: Economy system tables for currency, transactions, marketplace, and fraud detection
-- Supports both local L$ economy and third-party providers (Gloebit)

CREATE TABLE IF NOT EXISTS currency_balances (
    user_id char(36) NOT NULL,
    currency_code varchar(10) NOT NULL DEFAULT 'LS',
    balance bigint NOT NULL DEFAULT 0,
    reserved bigint NOT NULL DEFAULT 0,
    available bigint NOT NULL DEFAULT 0,
    version int NOT NULL DEFAULT 1,
    created_at datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, currency_code)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE INDEX idx_currency_balances_user ON currency_balances(user_id);

CREATE TABLE IF NOT EXISTS currency_definitions (
    currency_code varchar(10) NOT NULL PRIMARY KEY,
    currency_name varchar(64) NOT NULL DEFAULT '',
    currency_symbol varchar(8) NOT NULL DEFAULT '',
    decimal_places int NOT NULL DEFAULT 0,
    exchange_rate_to_base double NOT NULL DEFAULT 1.0,
    enabled tinyint NOT NULL DEFAULT 1,
    minimum_balance bigint NOT NULL DEFAULT 0,
    maximum_balance bigint NOT NULL DEFAULT 999999999
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS economy_transactions (
    id char(36) NOT NULL PRIMARY KEY,
    from_user char(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    to_user char(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    amount bigint NOT NULL DEFAULT 0,
    fee bigint NOT NULL DEFAULT 0,
    currency_code varchar(10) NOT NULL DEFAULT 'LS',
    transaction_type varchar(32) NOT NULL DEFAULT 'Transfer',
    status varchar(16) NOT NULL DEFAULT 'Pending',
    description text NOT NULL,
    metadata json NOT NULL,
    created_at datetime NOT NULL DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE INDEX idx_economy_transactions_from ON economy_transactions(from_user);
CREATE INDEX idx_economy_transactions_to ON economy_transactions(to_user);
CREATE INDEX idx_economy_transactions_created ON economy_transactions(created_at);
CREATE INDEX idx_economy_transactions_status ON economy_transactions(status);
CREATE INDEX idx_economy_transactions_type ON economy_transactions(transaction_type);

CREATE TABLE IF NOT EXISTS marketplace_categories (
    id char(36) NOT NULL PRIMARY KEY,
    name varchar(128) NOT NULL DEFAULT '',
    parent_id char(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    sort_order int NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS marketplace_listings (
    id char(36) NOT NULL PRIMARY KEY,
    seller_id char(36) NOT NULL,
    name varchar(255) NOT NULL DEFAULT '',
    description text NOT NULL,
    price bigint NOT NULL DEFAULT 0,
    currency_code varchar(10) NOT NULL DEFAULT 'LS',
    category_id char(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    tags text NOT NULL,
    status varchar(16) NOT NULL DEFAULT 'Active',
    asset_id char(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    created_at datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE INDEX idx_marketplace_listings_seller ON marketplace_listings(seller_id);
CREATE INDEX idx_marketplace_listings_status ON marketplace_listings(status);
CREATE INDEX idx_marketplace_listings_category ON marketplace_listings(category_id);

CREATE TABLE IF NOT EXISTS purchase_orders (
    id char(36) NOT NULL PRIMARY KEY,
    buyer_id char(36) NOT NULL,
    listing_id char(36) NOT NULL,
    amount bigint NOT NULL DEFAULT 0,
    fee bigint NOT NULL DEFAULT 0,
    status varchar(16) NOT NULL DEFAULT 'Pending',
    created_at datetime NOT NULL DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE INDEX idx_purchase_orders_buyer ON purchase_orders(buyer_id);
CREATE INDEX idx_purchase_orders_listing ON purchase_orders(listing_id);

CREATE TABLE IF NOT EXISTS escrow_accounts (
    id char(36) NOT NULL PRIMARY KEY,
    order_id char(36) NOT NULL,
    amount bigint NOT NULL DEFAULT 0,
    status varchar(16) NOT NULL DEFAULT 'Held',
    expires_at datetime NOT NULL,
    created_at datetime NOT NULL DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE INDEX idx_escrow_accounts_order ON escrow_accounts(order_id);
CREATE INDEX idx_escrow_accounts_status ON escrow_accounts(status);

CREATE TABLE IF NOT EXISTS fraud_alerts (
    id char(36) NOT NULL PRIMARY KEY,
    user_id char(36) NOT NULL,
    alert_type varchar(32) NOT NULL DEFAULT 'Unknown',
    severity varchar(16) NOT NULL DEFAULT 'Low',
    status varchar(16) NOT NULL DEFAULT 'Open',
    details json NOT NULL,
    created_at datetime NOT NULL DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE INDEX idx_fraud_alerts_user ON fraud_alerts(user_id);
CREATE INDEX idx_fraud_alerts_status ON fraud_alerts(status);
CREATE INDEX idx_fraud_alerts_severity ON fraud_alerts(severity);

CREATE TABLE IF NOT EXISTS gloebit_tokens (
    user_id char(36) NOT NULL PRIMARY KEY,
    access_token text NOT NULL,
    refresh_token text NOT NULL,
    token_scope text NOT NULL,
    expires_at datetime NOT NULL,
    created_at datetime NOT NULL DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
