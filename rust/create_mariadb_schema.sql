-- MariaDB Schema Creation for OpenSim Next
-- Run with: mysql -u opensim -p opensim_mariadb < create_mariadb_schema.sql

USE opensim_mariadb;

-- Create basic user accounts table
CREATE TABLE IF NOT EXISTS useraccounts (
    PrincipalID CHAR(36) NOT NULL,
    ScopeID CHAR(36) NOT NULL,
    FirstName VARCHAR(64) NOT NULL,
    LastName VARCHAR(64) NOT NULL,
    Email VARCHAR(64),
    ServiceURLs TEXT,
    Created BIGINT DEFAULT 0,
    UserLevel BIGINT DEFAULT 0,
    UserFlags BIGINT DEFAULT 0,
    UserTitle VARCHAR(64) DEFAULT '',
    Active BIGINT DEFAULT 1,
    PRIMARY KEY (PrincipalID),
    UNIQUE KEY Name (FirstName, LastName)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Create auth table for authentication
CREATE TABLE IF NOT EXISTS auth (
    UUID CHAR(36) NOT NULL,
    passwordHash CHAR(32) NOT NULL DEFAULT '',
    passwordSalt CHAR(32) NOT NULL DEFAULT '',
    webLoginKey CHAR(36) NOT NULL DEFAULT '',
    accountType VARCHAR(32) NOT NULL DEFAULT 'UserAccount',
    PRIMARY KEY (UUID)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Create assets table
CREATE TABLE IF NOT EXISTS assets (
    name VARCHAR(64),
    description VARCHAR(64),
    assetType BIGINT,
    local BIGINT,
    temporary BIGINT,
    data LONGBLOB,
    id CHAR(36) NOT NULL,
    create_time BIGINT,
    access_time BIGINT,
    asset_flags BIGINT,
    CreatorID VARCHAR(128),
    PRIMARY KEY (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Create regions table
CREATE TABLE IF NOT EXISTS regions (
    uuid CHAR(36) NOT NULL,
    regionHandle BIGINT,
    regionName VARCHAR(128),
    regionRecvKey VARCHAR(128),
    regionSendKey VARCHAR(128),
    regionSecret VARCHAR(128),
    regionDataURI VARCHAR(255),
    serverIP VARCHAR(64),
    serverPort BIGINT,
    serverURI VARCHAR(255),
    locX BIGINT,
    locY BIGINT,
    locZ BIGINT,
    eastOverrideHandle BIGINT,
    westOverrideHandle BIGINT,
    southOverrideHandle BIGINT,
    northOverrideHandle BIGINT,
    regionAssetURI VARCHAR(255),
    regionAssetRecvKey VARCHAR(128),
    regionAssetSendKey VARCHAR(128),
    regionUserURI VARCHAR(255),
    regionUserRecvKey VARCHAR(128),
    regionUserSendKey VARCHAR(128),
    regionMapTexture CHAR(36),
    serverHttpPort BIGINT,
    serverRemotingPort BIGINT,
    owner_uuid CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    originUUID CHAR(36),
    access BIGINT DEFAULT 1,
    ScopeID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    sizeX BIGINT NOT NULL DEFAULT 0,
    sizeY BIGINT NOT NULL DEFAULT 0,
    flags BIGINT NOT NULL DEFAULT 0,
    last_seen BIGINT NOT NULL DEFAULT 0,
    PrincipalID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    Token VARCHAR(255) NOT NULL DEFAULT '',
    parcelMapTexture CHAR(36) DEFAULT NULL,
    PRIMARY KEY (uuid),
    KEY regionName (regionName),
    KEY regionHandle (regionHandle),
    KEY overrideHandles (eastOverrideHandle, westOverrideHandle, southOverrideHandle, northOverrideHandle)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Create inventory folders
CREATE TABLE IF NOT EXISTS inventoryfolders (
    folderName VARCHAR(64),
    type BIGINT,
    version BIGINT,
    folderID CHAR(36) NOT NULL,
    agentID CHAR(36),
    parentFolderID CHAR(36),
    PRIMARY KEY (folderID),
    KEY owner (agentID),
    KEY parent (parentFolderID)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Create inventory items
CREATE TABLE IF NOT EXISTS inventoryitems (
    assetID CHAR(36),
    assetType BIGINT,
    inventoryName VARCHAR(64),
    inventoryDescription VARCHAR(128),
    inventoryNextPermissions BIGINT,
    inventoryCurrentPermissions BIGINT,
    invType BIGINT,
    creatorID CHAR(36),
    inventoryBasePermissions BIGINT,
    inventoryEveryOnePermissions BIGINT,
    salePrice BIGINT,
    saleType BIGINT,
    creationDate BIGINT,
    groupID CHAR(36),
    groupOwned BIGINT,
    flags BIGINT,
    inventoryID CHAR(36) NOT NULL,
    avatarID CHAR(36),
    parentFolderID CHAR(36),
    inventoryGroupPermissions BIGINT,
    PRIMARY KEY (inventoryID),
    KEY owner (avatarID),
    KEY folder (parentFolderID)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Create a test user for verification
INSERT IGNORE INTO useraccounts (PrincipalID, ScopeID, FirstName, LastName, Email, Created, UserLevel, UserFlags, Active) 
VALUES (
    '00000000-0000-0000-0000-000000000001',
    '00000000-0000-0000-0000-000000000000', 
    'Test',
    'User',
    'test@opensim.next',
    UNIX_TIMESTAMP(),
    0,
    0,
    1
);

-- Create auth record for test user (password: password123)
INSERT IGNORE INTO auth (UUID, passwordHash, passwordSalt, accountType)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'A0F83F6D6E08CB5CC0EC9B4C30F73C3E',
    '12345678901234567890123456789012',
    'UserAccount'
);

-- Create default region
INSERT IGNORE INTO regions (uuid, regionName, locX, locY, serverIP, serverPort, regionHandle, access, sizeX, sizeY, owner_uuid, ScopeID)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'OpenSim Test Region',
    1000,
    1000,
    '127.0.0.1',
    9000,
    4398046511104,
    1,
    256,
    256,
    '00000000-0000-0000-0000-000000000001',
    '00000000-0000-0000-0000-000000000000'
);

SELECT 'MariaDB schema created successfully for OpenSim Next!' AS Status;
SELECT COUNT(*) AS UserAccounts FROM useraccounts;
SELECT COUNT(*) AS Regions FROM regions;