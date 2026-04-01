-- Phase 197: Economy system tables for currency, transactions, marketplace, and fraud detection
-- Supports both local L$ economy and third-party providers (Gloebit)

-- Handle pre-existing currency_balances table (created by economy module with text types)
-- Drop and recreate with proper types if old schema detected
DO $$ BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'currency_balances' AND column_name = 'user_id' AND data_type = 'text'
    ) THEN
        DROP TABLE currency_balances;
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS currency_balances (
    user_id uuid NOT NULL,
    currency_code varchar(10) NOT NULL DEFAULT 'LS',
    balance bigint NOT NULL DEFAULT 0,
    reserved bigint NOT NULL DEFAULT 0,
    available bigint NOT NULL DEFAULT 0,
    version integer NOT NULL DEFAULT 1,
    created_at timestamp with time zone NOT NULL DEFAULT now(),
    updated_at timestamp with time zone NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, currency_code)
);

CREATE INDEX IF NOT EXISTS idx_currency_balances_user ON currency_balances(user_id);

CREATE TABLE IF NOT EXISTS currency_definitions (
    currency_code varchar(10) NOT NULL PRIMARY KEY,
    currency_name varchar(64) NOT NULL DEFAULT '',
    currency_symbol varchar(8) NOT NULL DEFAULT '',
    decimal_places integer NOT NULL DEFAULT 0,
    exchange_rate_to_base double precision NOT NULL DEFAULT 1.0,
    enabled boolean NOT NULL DEFAULT true,
    minimum_balance bigint NOT NULL DEFAULT 0,
    maximum_balance bigint NOT NULL DEFAULT 999999999
);

CREATE TABLE IF NOT EXISTS economy_transactions (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    from_user uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    to_user uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    amount bigint NOT NULL DEFAULT 0,
    fee bigint NOT NULL DEFAULT 0,
    currency_code varchar(10) NOT NULL DEFAULT 'LS',
    transaction_type varchar(32) NOT NULL DEFAULT 'Transfer',
    status varchar(16) NOT NULL DEFAULT 'Pending',
    description text NOT NULL DEFAULT '',
    metadata jsonb NOT NULL DEFAULT '{}',
    created_at timestamp with time zone NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_economy_transactions_from ON economy_transactions(from_user);
CREATE INDEX IF NOT EXISTS idx_economy_transactions_to ON economy_transactions(to_user);
CREATE INDEX IF NOT EXISTS idx_economy_transactions_created ON economy_transactions(created_at);
CREATE INDEX IF NOT EXISTS idx_economy_transactions_status ON economy_transactions(status);
CREATE INDEX IF NOT EXISTS idx_economy_transactions_type ON economy_transactions(transaction_type);

CREATE TABLE IF NOT EXISTS marketplace_categories (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    name varchar(128) NOT NULL DEFAULT '',
    parent_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    sort_order integer NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS marketplace_listings (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    seller_id uuid NOT NULL,
    name varchar(255) NOT NULL DEFAULT '',
    description text NOT NULL DEFAULT '',
    price bigint NOT NULL DEFAULT 0,
    currency_code varchar(10) NOT NULL DEFAULT 'LS',
    category_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    tags text NOT NULL DEFAULT '',
    status varchar(16) NOT NULL DEFAULT 'Active',
    asset_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    created_at timestamp with time zone NOT NULL DEFAULT now(),
    updated_at timestamp with time zone NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_marketplace_listings_seller ON marketplace_listings(seller_id);
CREATE INDEX IF NOT EXISTS idx_marketplace_listings_status ON marketplace_listings(status);
CREATE INDEX IF NOT EXISTS idx_marketplace_listings_category ON marketplace_listings(category_id);

CREATE TABLE IF NOT EXISTS purchase_orders (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    buyer_id uuid NOT NULL,
    listing_id uuid NOT NULL,
    amount bigint NOT NULL DEFAULT 0,
    fee bigint NOT NULL DEFAULT 0,
    status varchar(16) NOT NULL DEFAULT 'Pending',
    created_at timestamp with time zone NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_purchase_orders_buyer ON purchase_orders(buyer_id);
CREATE INDEX IF NOT EXISTS idx_purchase_orders_listing ON purchase_orders(listing_id);

CREATE TABLE IF NOT EXISTS escrow_accounts (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id uuid NOT NULL,
    amount bigint NOT NULL DEFAULT 0,
    status varchar(16) NOT NULL DEFAULT 'Held',
    expires_at timestamp with time zone NOT NULL DEFAULT now() + interval '30 days',
    created_at timestamp with time zone NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_escrow_accounts_order ON escrow_accounts(order_id);
CREATE INDEX IF NOT EXISTS idx_escrow_accounts_status ON escrow_accounts(status);

CREATE TABLE IF NOT EXISTS fraud_alerts (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL,
    alert_type varchar(32) NOT NULL DEFAULT 'Unknown',
    severity varchar(16) NOT NULL DEFAULT 'Low',
    status varchar(16) NOT NULL DEFAULT 'Open',
    details jsonb NOT NULL DEFAULT '{}',
    created_at timestamp with time zone NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_fraud_alerts_user ON fraud_alerts(user_id);
CREATE INDEX IF NOT EXISTS idx_fraud_alerts_status ON fraud_alerts(status);
CREATE INDEX IF NOT EXISTS idx_fraud_alerts_severity ON fraud_alerts(severity);

CREATE TABLE IF NOT EXISTS gloebit_tokens (
    user_id uuid NOT NULL PRIMARY KEY,
    access_token text NOT NULL DEFAULT '',
    refresh_token text NOT NULL DEFAULT '',
    token_scope text NOT NULL DEFAULT '',
    expires_at timestamp with time zone NOT NULL DEFAULT now(),
    created_at timestamp with time zone NOT NULL DEFAULT now()
);
