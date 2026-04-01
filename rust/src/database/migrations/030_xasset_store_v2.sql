-- XAssetStore Migration v1-2: Extended Asset Storage System
-- This migration creates the extended asset storage for enhanced asset management

-- Extended assets table (for asset references and metadata)
CREATE TABLE IF NOT EXISTS xassetsmeta (
    ID CHAR(36) NOT NULL PRIMARY KEY,
    Hash CHAR(32) NOT NULL,
    Name VARCHAR(64) NOT NULL,
    Description VARCHAR(64) NOT NULL,
    AssetType INTEGER NOT NULL,
    Local INTEGER NOT NULL,
    Temporary INTEGER NOT NULL,
    CreateTime INTEGER NOT NULL,
    AccessTime INTEGER NOT NULL,
    AssetFlags INTEGER NOT NULL,
    CreatorID VARCHAR(128) NOT NULL
);

-- Extended asset data table (for actual asset content)
CREATE TABLE IF NOT EXISTS xassetsdata (
    Hash CHAR(32) NOT NULL PRIMARY KEY,
    Data BLOB NOT NULL
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS xassetsmeta_hash ON xassetsmeta(Hash);
CREATE INDEX IF NOT EXISTS xassetsmeta_name ON xassetsmeta(Name);
CREATE INDEX IF NOT EXISTS xassetsmeta_assettype ON xassetsmeta(AssetType);
CREATE INDEX IF NOT EXISTS xassetsmeta_creatorid ON xassetsmeta(CreatorID);
CREATE INDEX IF NOT EXISTS xassetsdata_hash ON xassetsdata(Hash);