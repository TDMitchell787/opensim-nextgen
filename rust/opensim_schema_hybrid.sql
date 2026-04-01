-- OpenSim Master + OpenSim Next Hybrid Compatible Database Schema (SQLite)
-- This schema supports both OpenSim master and OpenSim Next migrations
-- Includes all required columns for full compatibility

-- Drop any existing incompatible tables
DROP TABLE IF EXISTS regions;
DROP TABLE IF EXISTS prims;

-- User Accounts (Version 6) - OpenSim Master Compatible
CREATE TABLE IF NOT EXISTS UserAccounts (
    PrincipalID CHAR(36) NOT NULL PRIMARY KEY,
    ScopeID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    FirstName VARCHAR(64) NOT NULL,
    LastName VARCHAR(64) NOT NULL,
    Email VARCHAR(64) DEFAULT NULL,
    ServiceURLs TEXT,
    Created INT DEFAULT NULL,
    UserLevel INT NOT NULL DEFAULT 0,
    UserFlags INT NOT NULL DEFAULT 0,
    UserTitle VARCHAR(64) NOT NULL DEFAULT '',
    active INT NOT NULL DEFAULT 1
);

CREATE INDEX IF NOT EXISTS UserAccounts_Email ON UserAccounts(Email);
CREATE INDEX IF NOT EXISTS UserAccounts_FirstName ON UserAccounts(FirstName);
CREATE INDEX IF NOT EXISTS UserAccounts_LastName ON UserAccounts(LastName);
CREATE INDEX IF NOT EXISTS UserAccounts_Name ON UserAccounts(FirstName, LastName);

-- Authentication (Version 4)
CREATE TABLE IF NOT EXISTS auth (
    UUID CHAR(36) NOT NULL PRIMARY KEY,
    passwordHash CHAR(32) NOT NULL DEFAULT '',
    passwordSalt CHAR(32) NOT NULL DEFAULT '',
    webLoginKey VARCHAR(255) NOT NULL DEFAULT '',
    accountType VARCHAR(32) NOT NULL DEFAULT 'UserAccount'
);

CREATE TABLE IF NOT EXISTS tokens (
    UUID CHAR(36) NOT NULL,
    token VARCHAR(255) NOT NULL,
    validity DATETIME NOT NULL,
    UNIQUE(UUID, token)
);

CREATE INDEX IF NOT EXISTS tokens_UUID ON tokens(UUID);
CREATE INDEX IF NOT EXISTS tokens_token ON tokens(token);
CREATE INDEX IF NOT EXISTS tokens_validity ON tokens(validity);

-- Assets (Version 10)
CREATE TABLE IF NOT EXISTS assets (
    name VARCHAR(64) NOT NULL,
    description VARCHAR(64) NOT NULL,
    assetType TINYINT NOT NULL,
    local TINYINT NOT NULL,
    temporary TINYINT NOT NULL,
    data BLOB NOT NULL,
    id CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000' PRIMARY KEY,
    create_time INT DEFAULT 0,
    access_time INT DEFAULT 0,
    asset_flags INT NOT NULL DEFAULT 0,
    CreatorID VARCHAR(128) NOT NULL DEFAULT ''
);

-- Inventory (Version 7)
CREATE TABLE IF NOT EXISTS inventoryitems (
    assetID VARCHAR(36) DEFAULT NULL,
    assetType INT DEFAULT NULL,
    inventoryName VARCHAR(64) DEFAULT NULL,
    inventoryDescription VARCHAR(128) DEFAULT NULL,
    inventoryNextPermissions INT DEFAULT NULL,
    inventoryCurrentPermissions INT DEFAULT NULL,
    invType INT DEFAULT NULL,
    creatorID VARCHAR(255) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    inventoryBasePermissions INT NOT NULL DEFAULT 0,
    inventoryEveryOnePermissions INT NOT NULL DEFAULT 0,
    salePrice INT NOT NULL DEFAULT 0,
    saleType TINYINT NOT NULL DEFAULT 0,
    creationDate INT NOT NULL DEFAULT 0,
    groupID VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    groupOwned TINYINT NOT NULL DEFAULT 0,
    flags INT NOT NULL DEFAULT 0,
    inventoryID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000' PRIMARY KEY,
    avatarID CHAR(36) DEFAULT NULL,
    parentFolderID CHAR(36) DEFAULT NULL,
    inventoryGroupPermissions INT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS inventoryitems_avatarid ON inventoryitems(avatarID);
CREATE INDEX IF NOT EXISTS inventoryitems_parentFolderid ON inventoryitems(parentFolderID);

CREATE TABLE IF NOT EXISTS inventoryfolders (
    folderName VARCHAR(64) DEFAULT NULL,
    type SMALLINT NOT NULL DEFAULT 0,
    version INT NOT NULL DEFAULT 0,
    folderID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000' PRIMARY KEY,
    agentID CHAR(36) DEFAULT NULL,
    parentFolderID CHAR(36) DEFAULT NULL
);

CREATE INDEX IF NOT EXISTS inventoryfolders_agentid ON inventoryfolders(agentID);
CREATE INDEX IF NOT EXISTS inventoryfolders_parentFolderid ON inventoryfolders(parentFolderID);

-- Regions - HYBRID SCHEMA (OpenSim master + OpenSim Next compatible)
CREATE TABLE IF NOT EXISTS regions (
    -- OpenSim Next columns (for migration compatibility)
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', abs(random()) % 4 + 1, 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6)))),
    region_name TEXT NOT NULL,
    location_x INTEGER NOT NULL DEFAULT 0,
    location_y INTEGER NOT NULL DEFAULT 0,
    size_x INTEGER DEFAULT 256,
    size_y INTEGER DEFAULT 256,
    external_host_name TEXT,
    external_port INTEGER,
    internal_host_name TEXT,
    internal_port INTEGER,
    
    -- OpenSim Master columns (for data compatibility)
    uuid CHAR(36) DEFAULT NULL,
    regionHandle BIGINT DEFAULT NULL,
    regionName VARCHAR(128) DEFAULT NULL,
    regionRecvKey VARCHAR(128) DEFAULT NULL,
    regionSendKey VARCHAR(128) DEFAULT NULL,
    regionSecret VARCHAR(128) DEFAULT NULL,
    regionDataURI VARCHAR(255) DEFAULT NULL,
    serverIP VARCHAR(64) DEFAULT NULL,
    serverPort INT DEFAULT NULL,
    serverURI VARCHAR(255) DEFAULT NULL,
    locX INT DEFAULT NULL,
    locY INT DEFAULT NULL,
    locZ INT DEFAULT NULL,
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
    serverHttpPort INT DEFAULT NULL,
    serverRemotingPort INT DEFAULT NULL,
    owner_uuid CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    originUUID CHAR(36) DEFAULT NULL,
    access INT DEFAULT 1,
    ScopeID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    sizeX INT NOT NULL DEFAULT 0,
    sizeY INT NOT NULL DEFAULT 0,
    flags INT NOT NULL DEFAULT 0,
    last_seen INT NOT NULL DEFAULT 0,
    
    -- Additional OpenSim Next fields
    map_texture TEXT,
    terrain_texture_1 TEXT,
    terrain_texture_2 TEXT,
    terrain_texture_3 TEXT,
    terrain_texture_4 TEXT,
    elevation_1_nw REAL DEFAULT 10.0,
    elevation_2_ne REAL DEFAULT 10.0,
    elevation_1_se REAL DEFAULT 10.0,
    elevation_2_sw REAL DEFAULT 10.0,
    water_height REAL DEFAULT 20.0,
    terrain_raise_limit REAL DEFAULT 100.0,
    terrain_lower_limit REAL DEFAULT -100.0,
    use_estate_sun BOOLEAN DEFAULT true,
    fixed_sun BOOLEAN DEFAULT false,
    sun_position REAL DEFAULT 0.0,
    covenant CHAR(36),
    sandbox BOOLEAN DEFAULT false,
    public_access BOOLEAN DEFAULT true,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS regions_name ON regions(region_name);
CREATE INDEX IF NOT EXISTS regions_location ON regions(location_x, location_y);
CREATE INDEX IF NOT EXISTS regions_opensim_location ON regions(locX, locY);

-- Prims table (simplified essential structure)
CREATE TABLE IF NOT EXISTS prims (
    CreationDate INT DEFAULT NULL,
    Name VARCHAR(255) DEFAULT NULL,
    Text VARCHAR(255) DEFAULT NULL,
    Description VARCHAR(255) DEFAULT NULL,
    SitName VARCHAR(255) DEFAULT NULL,
    TouchName VARCHAR(255) DEFAULT NULL,
    ObjectFlags INT DEFAULT NULL,
    OwnerMask INT DEFAULT NULL,
    NextOwnerMask INT DEFAULT NULL,
    GroupMask INT DEFAULT NULL,
    EveryoneMask INT DEFAULT NULL,
    BaseMask INT DEFAULT NULL,
    PositionX DOUBLE DEFAULT NULL,
    PositionY DOUBLE DEFAULT NULL,
    PositionZ DOUBLE DEFAULT NULL,
    GroupPositionX DOUBLE DEFAULT NULL,
    GroupPositionY DOUBLE DEFAULT NULL,
    GroupPositionZ DOUBLE DEFAULT NULL,
    VelocityX DOUBLE DEFAULT NULL,
    VelocityY DOUBLE DEFAULT NULL,
    VelocityZ DOUBLE DEFAULT NULL,
    AngularVelocityX DOUBLE DEFAULT NULL,
    AngularVelocityY DOUBLE DEFAULT NULL,
    AngularVelocityZ DOUBLE DEFAULT NULL,
    AccelerationX DOUBLE DEFAULT NULL,
    AccelerationY DOUBLE DEFAULT NULL,
    AccelerationZ DOUBLE DEFAULT NULL,
    RotationX DOUBLE DEFAULT NULL,
    RotationY DOUBLE DEFAULT NULL,
    RotationZ DOUBLE DEFAULT NULL,
    RotationW DOUBLE DEFAULT NULL,
    SitTargetOffsetX DOUBLE DEFAULT NULL,
    SitTargetOffsetY DOUBLE DEFAULT NULL,
    SitTargetOffsetZ DOUBLE DEFAULT NULL,
    SitTargetOrientW DOUBLE DEFAULT NULL,
    SitTargetOrientX DOUBLE DEFAULT NULL,
    SitTargetOrientY DOUBLE DEFAULT NULL,
    SitTargetOrientZ DOUBLE DEFAULT NULL,
    UUID CHAR(36) NOT NULL DEFAULT '' PRIMARY KEY,
    RegionUUID CHAR(36) DEFAULT NULL,
    CreatorID VARCHAR(255) NOT NULL DEFAULT '',
    OwnerID CHAR(36) DEFAULT NULL,
    GroupID CHAR(36) DEFAULT NULL,
    LastOwnerID CHAR(36) DEFAULT NULL,
    SceneGroupID CHAR(36) DEFAULT NULL,
    LinkNumber INT DEFAULT NULL,
    Material TINYINT DEFAULT NULL
);

-- Other OpenSim tables (unchanged)
CREATE TABLE IF NOT EXISTS GridUser (
    UserID VARCHAR(255) NOT NULL PRIMARY KEY,
    HomeRegionID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    HomePosition VARCHAR(64) NOT NULL DEFAULT '<0,0,0>',
    HomeLookAt VARCHAR(64) NOT NULL DEFAULT '<0,0,0>',
    LastRegionID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    LastPosition VARCHAR(64) NOT NULL DEFAULT '<0,0,0>',
    LastLookAt VARCHAR(64) NOT NULL DEFAULT '<0,0,0>',
    Online CHAR(5) NOT NULL DEFAULT 'false',
    Login CHAR(16) NOT NULL DEFAULT '0',
    Logout CHAR(16) NOT NULL DEFAULT '0'
);

CREATE TABLE IF NOT EXISTS Friends (
    PrincipalID CHAR(36) NOT NULL,
    Friend VARCHAR(255) NOT NULL,
    Flags INT NOT NULL DEFAULT 0,
    Offered VARCHAR(32) NOT NULL DEFAULT 0,
    PRIMARY KEY(PrincipalID, Friend)
);

CREATE TABLE IF NOT EXISTS Presence (
    UserID VARCHAR(255) NOT NULL,
    RegionID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    SessionID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    SecureSessionID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    PRIMARY KEY(UserID)
);

CREATE TABLE IF NOT EXISTS Avatars (
    PrincipalID CHAR(36) NOT NULL,
    Name VARCHAR(32) NOT NULL,
    Value VARCHAR(255) NOT NULL DEFAULT '',
    PRIMARY KEY(PrincipalID, Name)
);

CREATE TABLE IF NOT EXISTS land (
    UUID CHAR(36) NOT NULL PRIMARY KEY,
    RegionUUID CHAR(36) DEFAULT NULL,
    LocalLandID INT DEFAULT NULL,
    Bitmap LONGBLOB,
    Name VARCHAR(255) DEFAULT NULL,
    Description VARCHAR(255) DEFAULT NULL,
    OwnerUUID CHAR(36) DEFAULT NULL,
    IsGroupOwned INT DEFAULT NULL,
    Area INT DEFAULT NULL,
    AuctionID INT DEFAULT NULL,
    Category INT DEFAULT NULL,
    ClaimDate INT DEFAULT NULL,
    ClaimPrice INT DEFAULT NULL,
    GroupUUID CHAR(36) DEFAULT NULL,
    SalePrice INT DEFAULT NULL,
    LandStatus INT DEFAULT NULL,
    LandFlags INT DEFAULT NULL,
    LandingType TINYINT DEFAULT NULL,
    MediaAutoScale TINYINT DEFAULT NULL,
    MediaTextureUUID CHAR(36) DEFAULT NULL,
    MediaURL VARCHAR(255) DEFAULT NULL,
    MusicURL VARCHAR(255) DEFAULT NULL,
    PassHours FLOAT DEFAULT NULL,
    PassPrice INT DEFAULT NULL,
    SnapshotUUID CHAR(36) DEFAULT NULL,
    UserLocationX FLOAT DEFAULT NULL,
    UserLocationY FLOAT DEFAULT NULL,
    UserLocationZ FLOAT DEFAULT NULL,
    UserLookAtX FLOAT DEFAULT NULL,
    UserLookAtY FLOAT DEFAULT NULL,
    UserLookAtZ FLOAT DEFAULT NULL,
    AuthbuyerID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    OtherCleanTime INT NOT NULL DEFAULT 0,
    Dwell INT NOT NULL DEFAULT 0,
    MediaType VARCHAR(32) NOT NULL DEFAULT 'none/none',
    MediaDescription VARCHAR(255) NOT NULL DEFAULT '',
    MediaSize VARCHAR(16) NOT NULL DEFAULT '0,0',
    MediaLoop TINYINT NOT NULL DEFAULT 0,
    ObscureMusic TINYINT NOT NULL DEFAULT 0,
    ObscureMedia TINYINT NOT NULL DEFAULT 0,
    SeeAVs TINYINT NOT NULL DEFAULT 1,
    AnyAVSounds TINYINT NOT NULL DEFAULT 1,
    GroupAVSounds TINYINT NOT NULL DEFAULT 1
);

-- Migration tracking (compatible with both systems)
CREATE TABLE IF NOT EXISTS migrations (
    name VARCHAR(255) NOT NULL PRIMARY KEY,
    version INT NOT NULL
);

-- Users table for OpenSim Next compatibility (shadow table)
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', abs(random()) % 4 + 1, 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6)))),
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    salt TEXT NOT NULL,
    user_level INTEGER DEFAULT 0,
    user_flags INTEGER DEFAULT 0,
    user_title TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    last_login TEXT,
    home_region TEXT,
    home_location_x REAL DEFAULT 128.0,
    home_location_y REAL DEFAULT 128.0,
    home_location_z REAL DEFAULT 21.0,
    home_look_at_x REAL DEFAULT 1.0,
    home_look_at_y REAL DEFAULT 0.0,
    home_look_at_z REAL DEFAULT 0.0,
    profile_about TEXT,
    profile_first_text TEXT,
    profile_image TEXT,
    profile_partner TEXT,
    profile_url TEXT,
    profile_wants_to_mask INTEGER DEFAULT 0,
    profile_wants_to_text TEXT,
    profile_skills_mask INTEGER DEFAULT 0,
    profile_skills_text TEXT,
    profile_languages TEXT,
    active INTEGER DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_name ON users(first_name, last_name);

-- Insert migration records for compatibility
INSERT OR IGNORE INTO migrations (name, version) VALUES ('UserAccount', 6);
INSERT OR IGNORE INTO migrations (name, version) VALUES ('AuthStore', 4);
INSERT OR IGNORE INTO migrations (name, version) VALUES ('AssetStore', 10);
INSERT OR IGNORE INTO migrations (name, version) VALUES ('InventoryStore', 7);
INSERT OR IGNORE INTO migrations (name, version) VALUES ('RegionStore', 51);
INSERT OR IGNORE INTO migrations (name, version) VALUES ('GridUserStore', 1);
INSERT OR IGNORE INTO migrations (name, version) VALUES ('FriendsStore', 1);
INSERT OR IGNORE INTO migrations (name, version) VALUES ('Presence', 2);
INSERT OR IGNORE INTO migrations (name, version) VALUES ('Avatar', 2);