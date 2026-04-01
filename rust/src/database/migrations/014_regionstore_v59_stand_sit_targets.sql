-- RegionStore Migration Version 59: Add Stand Target and Sit Range
-- This migration adds support for stand target positioning and sit action range

-- Add stand target and sit range columns to prims table (SQLite compatible)
ALTER TABLE prims ADD COLUMN standtargetx REAL DEFAULT 0.0;
ALTER TABLE prims ADD COLUMN standtargety REAL DEFAULT 0.0;
ALTER TABLE prims ADD COLUMN standtargetz REAL DEFAULT 0.0;
ALTER TABLE prims ADD COLUMN sitactrange REAL DEFAULT 0.0;