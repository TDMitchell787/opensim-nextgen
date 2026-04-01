-- Phase 198: FSAssets metadata table for content-addressed filesystem storage
-- Stores UUID→SHA256 hash mapping; binary data lives on filesystem as .gz files
-- Separate namespace from upstream OpenSim migrations to avoid numbering collisions

CREATE TABLE IF NOT EXISTS fsassets (
    id char(36) NOT NULL PRIMARY KEY,
    name varchar(64) NOT NULL DEFAULT '',
    description varchar(64) DEFAULT '',
    type int(11) NOT NULL DEFAULT 0,
    hash char(80) NOT NULL DEFAULT '',
    create_time int(11) DEFAULT 0,
    access_time int(11) DEFAULT 0,
    asset_flags int(11) DEFAULT 0
) ENGINE=InnoDB;

CREATE INDEX idx_fsassets_hash ON fsassets(hash);
CREATE INDEX idx_fsassets_access_time ON fsassets(access_time);
