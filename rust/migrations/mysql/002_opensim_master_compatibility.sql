-- MySQL OpenSim Master Compatibility Migration
-- This migration creates all OpenSim master compatible tables with MySQL optimizations

-- Drop and recreate with OpenSim master compatibility
DROP TABLE IF EXISTS `regions`;
DROP TABLE IF EXISTS `users`;
DROP TABLE IF EXISTS `assets`;
DROP TABLE IF EXISTS `inventory_folders`;
DROP TABLE IF EXISTS `inventory_items`;
DROP TABLE IF EXISTS `estates`;
DROP TABLE IF EXISTS `sessions`;

-- OpenSim Master: UserAccounts table
CREATE TABLE `UserAccounts` (
    `PrincipalID` CHAR(36) NOT NULL PRIMARY KEY,
    `ScopeID` CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    `FirstName` VARCHAR(64) NOT NULL,
    `LastName` VARCHAR(64) NOT NULL,
    `Email` VARCHAR(64) DEFAULT NULL,
    `ServiceURLs` TEXT,
    `Created` INT DEFAULT NULL,
    `UserLevel` INT NOT NULL DEFAULT 0,
    `UserFlags` INT NOT NULL DEFAULT 0,
    `UserTitle` VARCHAR(64) NOT NULL DEFAULT '',
    `active` INT NOT NULL DEFAULT 1
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- OpenSim Master: Authentication table
CREATE TABLE `auth` (
    `UUID` CHAR(36) NOT NULL PRIMARY KEY,
    `passwordHash` CHAR(32) NOT NULL DEFAULT '',
    `passwordSalt` CHAR(32) NOT NULL DEFAULT '',
    `webLoginKey` VARCHAR(255) NOT NULL DEFAULT '',
    `accountType` VARCHAR(32) NOT NULL DEFAULT 'UserAccount'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- OpenSim Master: Authentication tokens
CREATE TABLE `tokens` (
    `UUID` CHAR(36) NOT NULL,
    `token` VARCHAR(255) NOT NULL,
    `validity` DATETIME NOT NULL,
    UNIQUE KEY `uuid_token` (`UUID`, `token`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- OpenSim Master: Assets table
CREATE TABLE `assets` (
    `name` VARCHAR(64) NOT NULL,
    `description` VARCHAR(64) NOT NULL,
    `assetType` TINYINT NOT NULL,
    `local` TINYINT NOT NULL,
    `temporary` TINYINT NOT NULL,
    `data` LONGBLOB NOT NULL,
    `id` CHAR(36) NOT NULL DEFAULT '' PRIMARY KEY,
    `create_time` INT DEFAULT 0,
    `access_time` INT DEFAULT 0,
    `asset_flags` INT NOT NULL DEFAULT 0,
    `CreatorID` VARCHAR(128) NOT NULL DEFAULT ''
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- OpenSim Master: Inventory folders
CREATE TABLE `inventoryfolders` (
    `folderName` VARCHAR(64) DEFAULT NULL,
    `type` SMALLINT DEFAULT NULL,
    `version` INT DEFAULT NULL,
    `folderID` CHAR(36) NOT NULL DEFAULT '' PRIMARY KEY,
    `agentID` CHAR(36) DEFAULT NULL,
    `parentFolderID` CHAR(36) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- OpenSim Master: Inventory items  
CREATE TABLE `inventoryitems` (
    `assetID` CHAR(36) DEFAULT NULL,
    `assetType` INT DEFAULT NULL,
    `inventoryName` VARCHAR(64) DEFAULT NULL,
    `inventoryDescription` VARCHAR(128) DEFAULT NULL,
    `inventoryNextPermissions` INT DEFAULT NULL,
    `inventoryCurrentPermissions` INT DEFAULT NULL,
    `invType` INT DEFAULT NULL,
    `creatorID` VARCHAR(255) DEFAULT NULL,
    `inventoryBasePermissions` INT DEFAULT NULL,
    `inventoryEveryOnePermissions` INT DEFAULT NULL,
    `salePrice` INT DEFAULT NULL,
    `saleType` TINYINT DEFAULT NULL,
    `creationDate` INT DEFAULT NULL,
    `groupID` CHAR(36) DEFAULT NULL,
    `groupOwned` TINYINT DEFAULT NULL,
    `flags` INT DEFAULT NULL,
    `inventoryID` CHAR(36) NOT NULL DEFAULT '' PRIMARY KEY,
    `parentFolderID` CHAR(36) DEFAULT NULL,
    `avatarID` CHAR(36) DEFAULT NULL,
    `inventoryGroupPermissions` INT DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Create indexes for UserAccounts
CREATE INDEX `UserAccounts_Email` ON `UserAccounts`(`Email`);
CREATE INDEX `UserAccounts_FirstName` ON `UserAccounts`(`FirstName`);
CREATE INDEX `UserAccounts_LastName` ON `UserAccounts`(`LastName`);
CREATE INDEX `UserAccounts_Name` ON `UserAccounts`(`FirstName`, `LastName`);

-- Create indexes for tokens
CREATE INDEX `tokens_UUID` ON `tokens`(`UUID`);
CREATE INDEX `tokens_token` ON `tokens`(`token`);
CREATE INDEX `tokens_validity` ON `tokens`(`validity`);

-- Create indexes for inventory
CREATE INDEX `inventoryfolders_agentID` ON `inventoryfolders`(`agentID`);
CREATE INDEX `inventoryfolders_parentFolderID` ON `inventoryfolders`(`parentFolderID`);
CREATE INDEX `inventoryitems_parentFolderID` ON `inventoryitems`(`parentFolderID`);
CREATE INDEX `inventoryitems_avatarID` ON `inventoryitems`(`avatarID`);