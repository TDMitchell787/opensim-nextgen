-- Migration 009: Create primitems table for prim inventory (scripts, notecards, objects in Contents tab)
-- Matches C# OpenSim RegionStore MySQL schema

CREATE TABLE IF NOT EXISTS `primitems` (
    `itemID` char(36) NOT NULL DEFAULT '',
    `primID` char(36) DEFAULT NULL,
    `assetID` char(36) DEFAULT NULL,
    `parentFolderID` char(36) DEFAULT NULL,
    `invType` int(11) DEFAULT 0,
    `assetType` int(11) DEFAULT 0,
    `name` varchar(255) DEFAULT '',
    `description` varchar(255) DEFAULT '',
    `creationDate` bigint(20) DEFAULT 0,
    `CreatorID` varchar(255) NOT NULL DEFAULT '',
    `ownerID` char(36) DEFAULT NULL,
    `lastOwnerID` char(36) DEFAULT NULL,
    `groupID` char(36) DEFAULT NULL,
    `nextPermissions` int(11) DEFAULT 0,
    `currentPermissions` int(11) DEFAULT 0,
    `basePermissions` int(11) DEFAULT 0,
    `everyonePermissions` int(11) DEFAULT 0,
    `groupPermissions` int(11) DEFAULT 0,
    `flags` int(11) NOT NULL DEFAULT 0,
    PRIMARY KEY (`itemID`),
    KEY `primitems_primid` (`primID`)
) ENGINE=InnoDB DEFAULT CHARSET=latin1;
