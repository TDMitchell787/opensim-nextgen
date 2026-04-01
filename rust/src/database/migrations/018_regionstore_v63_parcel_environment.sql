-- RegionStore Migration Version 63: Parcel Environment Store
-- This migration adds support for parcel-specific environment settings

-- Add environment column to land table (SQLite compatible)
ALTER TABLE land ADD COLUMN environment TEXT DEFAULT NULL;