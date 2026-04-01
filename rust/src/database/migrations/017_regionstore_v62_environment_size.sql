-- RegionStore Migration Version 62: Increase Environment Settings Size
-- This migration increases storage capacity for environment settings

-- Create regionenvironment table if it doesn't exist (SQLite compatible)
CREATE TABLE IF NOT EXISTS regionenvironment (
    region_id VARCHAR(36) NOT NULL DEFAULT '000000-0000-0000-0000-000000000000',
    llsd_settings TEXT,
    PRIMARY KEY (region_id)
);

-- Create index for performance
CREATE INDEX IF NOT EXISTS regionenvironment_region_id ON regionenvironment(region_id);