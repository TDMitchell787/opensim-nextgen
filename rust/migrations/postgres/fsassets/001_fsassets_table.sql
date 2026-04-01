-- Phase 198: FSAssets metadata table for content-addressed filesystem storage
-- Stores UUID→SHA256 hash mapping; binary data lives on filesystem as .gz files
-- Separate namespace from upstream OpenSim migrations to avoid numbering collisions

CREATE TABLE IF NOT EXISTS fsassets (
    id uuid PRIMARY KEY,
    type integer NOT NULL DEFAULT 0,
    hash char(64) NOT NULL DEFAULT '',
    create_time integer NOT NULL DEFAULT 0,
    access_time integer NOT NULL DEFAULT 0,
    asset_flags integer NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_fsassets_hash ON fsassets(hash);
CREATE INDEX IF NOT EXISTS idx_fsassets_access_time ON fsassets(access_time);
