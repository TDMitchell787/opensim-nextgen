-- GridStore Migration v1-10: Grid Registration and Discovery System
-- This migration creates the grid service for region registration and discovery

-- Regions table for grid service (different from local regions table)
CREATE TABLE IF NOT EXISTS regions (
    uuid VARCHAR(36) NOT NULL PRIMARY KEY,
    regionHandle BIGINT NOT NULL,
    regionName VARCHAR(128) DEFAULT NULL,
    regionRecvKey VARCHAR(128) DEFAULT NULL,
    regionSendKey VARCHAR(128) DEFAULT NULL,
    regionSecret VARCHAR(128) DEFAULT NULL,
    regionDataURI VARCHAR(255) DEFAULT NULL,
    serverIP VARCHAR(64) DEFAULT NULL,
    serverPort INTEGER DEFAULT NULL,
    serverURI VARCHAR(255) DEFAULT NULL,
    locX INTEGER DEFAULT NULL,
    locY INTEGER DEFAULT NULL,
    locZ INTEGER DEFAULT NULL,
    eastOverrideHandle BIGINT DEFAULT NULL,
    westOverrideHandle BIGINT DEFAULT NULL,
    southOverrideHandle BIGINT DEFAULT NULL,
    northOverrideHandle BIGINT DEFAULT NULL,
    regionAssetURI VARCHAR(255) DEFAULT NULL,
    regionAssetRecvKey VARCHAR(128) DEFAULT NULL,
    regionAssetSendKey VARCHAR(128) DEFAULT NULL,
    regionUserURI VARCHAR(255) DEFAULT NULL,
    regionUserRecvKey VARCHAR(128) DEFAULT NULL,
    regionUserSendKey VARCHAR(128) DEFAULT NULL,
    regionMapTexture CHAR(36) DEFAULT NULL,
    serverHttpPort INTEGER DEFAULT NULL,
    serverRemotingPort INTEGER DEFAULT NULL,
    owner_uuid CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    originUUID CHAR(36) DEFAULT NULL,
    access INTEGER DEFAULT 1,
    ScopeID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    sizeX INTEGER NOT NULL DEFAULT 0,
    sizeY INTEGER NOT NULL DEFAULT 0,
    flags INTEGER NOT NULL DEFAULT 0,
    last_seen INTEGER NOT NULL DEFAULT 0,
    PrincipalID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    Token VARCHAR(255) NOT NULL DEFAULT '',
    parcelMapTexture VARCHAR(36) DEFAULT NULL
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS grid_regions_regionname ON regions(regionName);
CREATE INDEX IF NOT EXISTS grid_regions_regionhandle ON regions(regionHandle);
CREATE INDEX IF NOT EXISTS grid_regions_overridehandles ON regions(eastOverrideHandle, westOverrideHandle, southOverrideHandle, northOverrideHandle);
CREATE INDEX IF NOT EXISTS grid_regions_scopeid ON regions(ScopeID);
CREATE INDEX IF NOT EXISTS grid_regions_flags ON regions(flags);