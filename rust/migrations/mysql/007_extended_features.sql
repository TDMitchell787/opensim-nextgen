-- MySQL Extended Features Complete Implementation
-- This migration creates all remaining advanced store systems

-- XAssetStore: Extended asset metadata
CREATE TABLE `xassetsmeta` (
    `ID` CHAR(36) NOT NULL PRIMARY KEY,
    `Hash` CHAR(32) NOT NULL,
    `Name` VARCHAR(64) NOT NULL,
    `Description` VARCHAR(64) NOT NULL,
    `AssetType` TINYINT NOT NULL,
    `Local` TINYINT NOT NULL,
    `Temporary` TINYINT NOT NULL,
    `CreateTime` INT NOT NULL,
    `AccessTime` INT NOT NULL,
    `AssetFlags` INT NOT NULL,
    `CreatorID` VARCHAR(128) NOT NULL,
    KEY `xassetsmeta_hash` (`Hash`),
    KEY `xassetsmeta_name` (`Name`),
    KEY `xassetsmeta_assettype` (`AssetType`),
    KEY `xassetsmeta_creatorid` (`CreatorID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- XAssetStore: Extended asset data
CREATE TABLE `xassetsdata` (
    `Hash` CHAR(32) NOT NULL PRIMARY KEY,
    `Data` LONGBLOB NOT NULL,
    KEY `xassetsdata_hash` (`Hash`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- IM_Store: Offline instant messages
CREATE TABLE `im_offline` (
    `ID` MEDIUMINT(9) NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `PrincipalID` CHAR(36) NOT NULL DEFAULT '',
    `FromID` CHAR(36) NOT NULL DEFAULT '',
    `Message` TEXT NOT NULL,
    `TMStamp` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    KEY `PrincipalID` (`PrincipalID`),
    KEY `FromID` (`FromID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- HGTravelStore: Hypergrid travel data
CREATE TABLE `hg_traveling_data` (
    `SessionID` VARCHAR(36) NOT NULL PRIMARY KEY,
    `UserID` VARCHAR(36) NOT NULL,
    `GridExternalName` VARCHAR(255) NOT NULL DEFAULT '',
    `ServiceToken` VARCHAR(255) NOT NULL DEFAULT '',
    `ClientIPAddress` VARCHAR(16) NOT NULL DEFAULT '',
    `MyIPAddress` VARCHAR(16) NOT NULL DEFAULT '',
    `TMStamp` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    KEY `UserID` (`UserID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- MuteListStore: User mute lists
CREATE TABLE `MuteList` (
    `AgentID` CHAR(36) NOT NULL,
    `MuteID` CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    `MuteName` VARCHAR(64) NOT NULL DEFAULT '',
    `MuteType` INT(11) NOT NULL DEFAULT 1,
    `MuteFlags` INT(11) NOT NULL DEFAULT 0,
    `Stamp` INT(11) NOT NULL,
    UNIQUE KEY `AgentID_2` (`AgentID`,`MuteID`,`MuteName`),
    KEY `AgentID` (`AgentID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- os_groups_Store: Groups system
CREATE TABLE `os_groups_groups` (
    `GroupID` CHAR(36) NOT NULL PRIMARY KEY,
    `Location` VARCHAR(255) NOT NULL DEFAULT '',
    `Name` VARCHAR(255) CHARACTER SET utf8mb4 NOT NULL DEFAULT '',
    `Charter` TEXT CHARACTER SET utf8mb4 NOT NULL,
    `InsigniaID` CHAR(36) NOT NULL DEFAULT '',
    `FounderID` CHAR(36) NOT NULL DEFAULT '',
    `MembershipFee` INT(11) NOT NULL DEFAULT 0,
    `OpenEnrollment` VARCHAR(255) NOT NULL DEFAULT '',
    `ShowInList` INT(4) NOT NULL DEFAULT 0,
    `AllowPublish` INT(4) NOT NULL DEFAULT 0,
    `MaturePublish` INT(4) NOT NULL DEFAULT 0,
    `OwnerRoleID` CHAR(36) NOT NULL DEFAULT '',
    UNIQUE KEY `Name` (`Name`),
    FULLTEXT KEY `Name_2` (`Name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- os_groups_Store: Group membership
CREATE TABLE `os_groups_membership` (
    `GroupID` CHAR(36) NOT NULL DEFAULT '',
    `PrincipalID` VARCHAR(255) NOT NULL DEFAULT '',
    `SelectedRoleID` CHAR(36) NOT NULL DEFAULT '',
    `Contribution` INT(11) NOT NULL DEFAULT 0,
    `ListInProfile` INT(4) NOT NULL DEFAULT 1,
    `AcceptNotices` INT(4) NOT NULL DEFAULT 1,
    `AccessToken` CHAR(36) NOT NULL DEFAULT '',
    PRIMARY KEY (`GroupID`,`PrincipalID`),
    KEY `PrincipalID` (`PrincipalID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- os_groups_Store: Group roles
CREATE TABLE `os_groups_roles` (
    `GroupID` CHAR(36) NOT NULL DEFAULT '',
    `RoleID` CHAR(36) NOT NULL DEFAULT '',
    `Name` VARCHAR(255) CHARACTER SET utf8mb4 NOT NULL DEFAULT '',
    `Description` VARCHAR(255) CHARACTER SET utf8mb4 NOT NULL DEFAULT '',
    `Title` VARCHAR(255) CHARACTER SET utf8mb4 NOT NULL DEFAULT '',
    `Powers` BIGINT(20) UNSIGNED NOT NULL DEFAULT 0,
    PRIMARY KEY (`GroupID`,`RoleID`),
    KEY `GroupID` (`GroupID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- os_groups_Store: Group role membership
CREATE TABLE `os_groups_rolemembership` (
    `GroupID` CHAR(36) NOT NULL DEFAULT '',
    `RoleID` CHAR(36) NOT NULL DEFAULT '',
    `PrincipalID` VARCHAR(255) NOT NULL DEFAULT '',
    PRIMARY KEY (`GroupID`,`RoleID`,`PrincipalID`),
    KEY `PrincipalID` (`PrincipalID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- os_groups_Store: Group invites
CREATE TABLE `os_groups_invites` (
    `InviteID` CHAR(36) NOT NULL PRIMARY KEY,
    `GroupID` CHAR(36) NOT NULL DEFAULT '',
    `RoleID` CHAR(36) NOT NULL DEFAULT '',
    `PrincipalID` VARCHAR(255) NOT NULL DEFAULT '',
    `TMStamp` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY `PrincipalGroup` (`GroupID`,`PrincipalID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- os_groups_Store: Group notices
CREATE TABLE `os_groups_notices` (
    `GroupID` CHAR(36) NOT NULL DEFAULT '',
    `NoticeID` CHAR(36) NOT NULL PRIMARY KEY,
    `TMStamp` INT(10) UNSIGNED NOT NULL DEFAULT 0,
    `FromName` VARCHAR(255) NOT NULL DEFAULT '',
    `Subject` VARCHAR(255) CHARACTER SET utf8mb4 NOT NULL DEFAULT '',
    `Message` TEXT CHARACTER SET utf8mb4 NOT NULL,
    `HasAttachment` INT(4) NOT NULL DEFAULT 0,
    `AttachmentType` INT(4) NOT NULL DEFAULT 0,
    `AttachmentName` VARCHAR(128) NOT NULL DEFAULT '',
    `AttachmentItemID` CHAR(36) NOT NULL DEFAULT '',
    `AttachmentOwnerID` VARCHAR(255) NOT NULL DEFAULT '',
    KEY `GroupID` (`GroupID`),
    KEY `TMStamp` (`TMStamp`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- os_groups_Store: Group principals
CREATE TABLE `os_groups_principals` (
    `PrincipalID` VARCHAR(255) NOT NULL PRIMARY KEY,
    `ActiveGroupID` CHAR(36) NOT NULL DEFAULT ''
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- LogStore: System logs
CREATE TABLE `logs` (
    `logID` INT(10) UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `target` VARCHAR(36) DEFAULT NULL,
    `server` VARCHAR(64) DEFAULT NULL,
    `method` VARCHAR(64) DEFAULT NULL,
    `arguments` VARCHAR(255) DEFAULT NULL,
    `priority` INT(11) DEFAULT NULL,
    `message` TEXT,
    KEY `logs_target` (`target`),
    KEY `logs_server` (`server`),
    KEY `logs_method` (`method`),
    KEY `logs_priority` (`priority`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;