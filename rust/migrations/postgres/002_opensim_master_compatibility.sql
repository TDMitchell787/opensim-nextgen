-- PostgreSQL OpenSim Master Compatibility Migration
-- This migration creates all OpenSim master compatible tables with PostgreSQL optimizations

-- Drop and recreate users table with OpenSim master compatibility
DROP TABLE IF EXISTS users CASCADE;

-- OpenSim Master: UserAccounts table
CREATE TABLE UserAccounts (
    PrincipalID UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    ScopeID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    FirstName VARCHAR(64) NOT NULL,
    LastName VARCHAR(64) NOT NULL,
    Email VARCHAR(64) DEFAULT NULL,
    ServiceURLs TEXT,
    Created INTEGER DEFAULT NULL,
    UserLevel INTEGER NOT NULL DEFAULT 0,
    UserFlags INTEGER NOT NULL DEFAULT 0,
    UserTitle VARCHAR(64) NOT NULL DEFAULT '',
    active INTEGER NOT NULL DEFAULT 1
);

-- OpenSim Master: Authentication table
CREATE TABLE auth (
    UUID UUID NOT NULL PRIMARY KEY,
    passwordHash CHAR(32) NOT NULL DEFAULT '',
    passwordSalt CHAR(32) NOT NULL DEFAULT '',
    webLoginKey VARCHAR(255) NOT NULL DEFAULT '',
    accountType VARCHAR(32) NOT NULL DEFAULT 'UserAccount'
);

-- OpenSim Master: Authentication tokens
CREATE TABLE tokens (
    UUID UUID NOT NULL,
    token VARCHAR(255) NOT NULL,
    validity TIMESTAMP WITH TIME ZONE NOT NULL,
    UNIQUE(UUID, token)
);

-- OpenSim Master: Assets table
DROP TABLE IF EXISTS assets CASCADE;
CREATE TABLE assets (
    name VARCHAR(64) NOT NULL,
    description VARCHAR(64) NOT NULL,
    assetType INTEGER NOT NULL,
    local INTEGER NOT NULL,
    temporary INTEGER NOT NULL,
    data BYTEA NOT NULL,
    id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    create_time INTEGER DEFAULT 0,
    access_time INTEGER DEFAULT 0,
    asset_flags INTEGER NOT NULL DEFAULT 0,
    CreatorID VARCHAR(128) NOT NULL DEFAULT ''
);

-- OpenSim Master: Inventory folders
DROP TABLE IF EXISTS inventory_folders CASCADE;
CREATE TABLE inventoryfolders (
    folderName VARCHAR(64) DEFAULT NULL,
    type INTEGER DEFAULT NULL,
    version INTEGER DEFAULT NULL,
    folderID UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    agentID UUID DEFAULT NULL,
    parentFolderID UUID DEFAULT NULL
);

-- OpenSim Master: Inventory items  
DROP TABLE IF EXISTS inventory_items CASCADE;
CREATE TABLE inventoryitems (
    assetID UUID DEFAULT NULL,
    assetType INTEGER DEFAULT NULL,
    inventoryName VARCHAR(64) DEFAULT NULL,
    inventoryDescription VARCHAR(128) DEFAULT NULL,
    inventoryNextPermissions INTEGER DEFAULT NULL,
    inventoryCurrentPermissions INTEGER DEFAULT NULL,
    invType INTEGER DEFAULT NULL,
    creatorID VARCHAR(255) DEFAULT NULL,
    inventoryBasePermissions INTEGER DEFAULT NULL,
    inventoryEveryOnePermissions INTEGER DEFAULT NULL,
    salePrice INTEGER DEFAULT NULL,
    saleType INTEGER DEFAULT NULL,
    creationDate INTEGER DEFAULT NULL,
    groupID UUID DEFAULT NULL,
    groupOwned INTEGER DEFAULT NULL,
    flags INTEGER DEFAULT NULL,
    inventoryID UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    parentFolderID UUID DEFAULT NULL,
    avatarID UUID DEFAULT NULL,
    inventoryGroupPermissions INTEGER DEFAULT NULL
);

-- Create indexes for UserAccounts
CREATE INDEX UserAccounts_Email ON UserAccounts(Email);
CREATE INDEX UserAccounts_FirstName ON UserAccounts(FirstName);
CREATE INDEX UserAccounts_LastName ON UserAccounts(LastName);
CREATE INDEX UserAccounts_Name ON UserAccounts(FirstName, LastName);

-- Create indexes for tokens
CREATE INDEX tokens_UUID ON tokens(UUID);
CREATE INDEX tokens_token ON tokens(token);
CREATE INDEX tokens_validity ON tokens(validity);

-- Create indexes for inventory
CREATE INDEX inventoryfolders_agentID ON inventoryfolders(agentID);
CREATE INDEX inventoryfolders_parentFolderID ON inventoryfolders(parentFolderID);
CREATE INDEX inventoryitems_parentFolderID ON inventoryitems(parentFolderID);
CREATE INDEX inventoryitems_avatarID ON inventoryitems(avatarID);