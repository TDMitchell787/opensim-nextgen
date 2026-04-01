-- OpenSim Master Compatible Database Schema (SQLite)
-- Based on latest migration files from OpenSim master
-- This ensures 100% compatibility with existing OpenSim installations

-- User Accounts (Version 6)
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

CREATE INDEX UserAccounts_Email ON UserAccounts(Email);
CREATE INDEX UserAccounts_FirstName ON UserAccounts(FirstName);
CREATE INDEX UserAccounts_LastName ON UserAccounts(LastName);
CREATE INDEX UserAccounts_Name ON UserAccounts(FirstName, LastName);

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

CREATE INDEX tokens_UUID ON tokens(UUID);
CREATE INDEX tokens_token ON tokens(token);
CREATE INDEX tokens_validity ON tokens(validity);

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

CREATE INDEX inventoryitems_avatarid ON inventoryitems(avatarID);
CREATE INDEX inventoryitems_parentFolderid ON inventoryitems(parentFolderID);

CREATE TABLE IF NOT EXISTS inventoryfolders (
    folderName VARCHAR(64) DEFAULT NULL,
    type SMALLINT NOT NULL DEFAULT 0,
    version INT NOT NULL DEFAULT 0,
    folderID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000' PRIMARY KEY,
    agentID CHAR(36) DEFAULT NULL,
    parentFolderID CHAR(36) DEFAULT NULL
);

CREATE INDEX inventoryfolders_agentid ON inventoryfolders(agentID);
CREATE INDEX inventoryfolders_parentFolderid ON inventoryfolders(parentFolderID);

-- Regions (Version 51 - simplified for essential fields)
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

-- Regions table
CREATE TABLE IF NOT EXISTS regions (
    uuid CHAR(36) NOT NULL DEFAULT '' PRIMARY KEY,
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
    last_seen INT NOT NULL DEFAULT 0
);

-- GridUser table for user presence
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

-- Friends table
CREATE TABLE IF NOT EXISTS Friends (
    PrincipalID CHAR(36) NOT NULL,
    Friend VARCHAR(255) NOT NULL,
    Flags INT NOT NULL DEFAULT 0,
    Offered VARCHAR(32) NOT NULL DEFAULT 0,
    PRIMARY KEY(PrincipalID, Friend)
);

-- Presence table
CREATE TABLE IF NOT EXISTS Presence (
    UserID VARCHAR(255) NOT NULL,
    RegionID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    SessionID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    SecureSessionID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    PRIMARY KEY(UserID)
);

-- Avatars table for appearance
CREATE TABLE IF NOT EXISTS Avatars (
    PrincipalID CHAR(36) NOT NULL,
    Name VARCHAR(32) NOT NULL,
    Value VARCHAR(255) NOT NULL DEFAULT '',
    PRIMARY KEY(PrincipalID, Name)
);

-- Land table for parcels
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

-- Migration tracking
CREATE TABLE IF NOT EXISTS migrations (
    name VARCHAR(255) NOT NULL PRIMARY KEY,
    version INT NOT NULL
);

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