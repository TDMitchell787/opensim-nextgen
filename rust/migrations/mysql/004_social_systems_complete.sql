-- MySQL Social Systems Complete Implementation
-- This migration creates all social system tables with OpenSim master compatibility

-- OpenSim Master: Friends table
CREATE TABLE `Friends` (
    `PrincipalID` VARCHAR(255) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    `Friend` VARCHAR(255) NOT NULL,
    `Flags` VARCHAR(16) NOT NULL DEFAULT '0',
    `Offered` VARCHAR(32) NOT NULL DEFAULT '0',
    PRIMARY KEY (`PrincipalID`(36),`Friend`(36)),
    KEY `PrincipalID` (`PrincipalID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- OpenSim Master: Presence table
CREATE TABLE `Presence` (
    `UserID` VARCHAR(255) NOT NULL,
    `RegionID` CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    `SessionID` CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    `SecureSessionID` CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    `LastSeen` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY `SessionID` (`SessionID`),
    KEY `UserID` (`UserID`),
    KEY `RegionID` (`RegionID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- OpenSim Master: Avatars table
CREATE TABLE `Avatars` (
    `PrincipalID` CHAR(36) NOT NULL,
    `Name` VARCHAR(32) NOT NULL,
    `Value` TEXT,
    PRIMARY KEY (`PrincipalID`,`Name`),
    KEY `PrincipalID` (`PrincipalID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- OpenSim Master: GridUser table
CREATE TABLE `GridUser` (
    `UserID` VARCHAR(255) NOT NULL PRIMARY KEY,
    `HomeRegionID` CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    `HomePosition` CHAR(64) NOT NULL DEFAULT '<0,0,0>',
    `HomeLookAt` CHAR(64) NOT NULL DEFAULT '<0,0,0>',
    `LastRegionID` CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    `LastPosition` CHAR(64) NOT NULL DEFAULT '<0,0,0>',
    `LastLookAt` CHAR(64) NOT NULL DEFAULT '<0,0,0>',
    `Online` CHAR(5) NOT NULL DEFAULT 'false',
    `Login` CHAR(16) NOT NULL DEFAULT '0',
    `Logout` CHAR(16) NOT NULL DEFAULT '0'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- OpenSim Master: Land table
CREATE TABLE `land` (
    `UUID` CHAR(36) NOT NULL PRIMARY KEY,
    `RegionUUID` CHAR(36) DEFAULT NULL,
    `LocalLandID` INT DEFAULT NULL,
    `Bitmap` LONGBLOB,
    `Name` VARCHAR(255) DEFAULT NULL,
    `Description` VARCHAR(255) DEFAULT NULL,
    `OwnerUUID` CHAR(36) DEFAULT NULL,
    `IsGroupOwned` TINYINT DEFAULT NULL,
    `Area` INT DEFAULT NULL,
    `AuctionID` INT UNSIGNED DEFAULT NULL,
    `Category` TINYINT DEFAULT NULL,
    `ClaimDate` INT DEFAULT NULL,
    `ClaimPrice` INT DEFAULT NULL,
    `GroupUUID` CHAR(36) DEFAULT NULL,
    `SalePrice` INT DEFAULT NULL,
    `LandStatus` TINYINT DEFAULT NULL,
    `LandFlags` INT UNSIGNED DEFAULT NULL,
    `LandingType` TINYINT UNSIGNED DEFAULT NULL,
    `MediaAutoScale` TINYINT DEFAULT NULL,
    `MediaTextureUUID` CHAR(36) DEFAULT NULL,
    `MediaURL` VARCHAR(255) DEFAULT NULL,
    `MusicURL` VARCHAR(255) DEFAULT NULL,
    `PassHours` FLOAT DEFAULT NULL,
    `PassPrice` INT UNSIGNED DEFAULT NULL,
    `SnapshotUUID` CHAR(36) DEFAULT NULL,
    `UserLocationX` FLOAT DEFAULT NULL,
    `UserLocationY` FLOAT DEFAULT NULL,
    `UserLocationZ` FLOAT DEFAULT NULL,
    `UserLookAtX` FLOAT DEFAULT NULL,
    `UserLookAtY` FLOAT DEFAULT NULL,
    `UserLookAtZ` FLOAT DEFAULT NULL,
    `AuthbuyerID` CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    `OtherCleanTime` INT NOT NULL DEFAULT 0,
    `Dwell` INT NOT NULL DEFAULT 0,
    `MediaType` VARCHAR(32) NOT NULL DEFAULT 'none/none',
    `MediaDescription` VARCHAR(255) NOT NULL DEFAULT '',
    `MediaSize` VARCHAR(16) NOT NULL DEFAULT '0,0',
    `MediaLoop` TINYINT NOT NULL DEFAULT 0,
    `ObscureMusic` TINYINT NOT NULL DEFAULT 0,
    `ObscureMedia` TINYINT NOT NULL DEFAULT 0,
    `SeeAVs` TINYINT(4) NOT NULL DEFAULT 1,
    `AnyAVSounds` TINYINT(4) NOT NULL DEFAULT 1,
    `GroupAVSounds` TINYINT(4) NOT NULL DEFAULT 1,
    `environment` MEDIUMTEXT DEFAULT NULL,
    KEY `land_regionuuid` (`RegionUUID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;