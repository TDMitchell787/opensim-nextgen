-- MySQL Complete Stores Implementation
-- This migration creates estate, user profiles, and agent preference systems

-- EstateStore: Estate Settings (main configuration)
CREATE TABLE `estate_settings` (
    `EstateID` INT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `EstateName` VARCHAR(64) NOT NULL DEFAULT 'My Estate',
    `AbuseEmailToEstateOwner` TINYINT(4) NOT NULL DEFAULT 1,
    `DenyAnonymous` TINYINT(4) NOT NULL DEFAULT 0,
    `ResetHomeOnTeleport` TINYINT(4) NOT NULL DEFAULT 0,
    `FixedSun` TINYINT(4) NOT NULL DEFAULT 0,
    `DenyTransacted` TINYINT(4) NOT NULL DEFAULT 0,
    `BlockDwell` TINYINT(4) NOT NULL DEFAULT 0,
    `DenyIdentified` TINYINT(4) NOT NULL DEFAULT 0,
    `AllowVoice` TINYINT(4) NOT NULL DEFAULT 1,
    `UseGlobalTime` TINYINT(4) NOT NULL DEFAULT 1,
    `PricePerMeter` INT NOT NULL DEFAULT 1,
    `TaxFree` TINYINT(4) NOT NULL DEFAULT 0,
    `AllowDirectTeleport` TINYINT(4) NOT NULL DEFAULT 1,
    `RedirectGridX` INT NOT NULL DEFAULT 0,
    `RedirectGridY` INT NOT NULL DEFAULT 0,
    `ParentEstateID` INT NOT NULL DEFAULT 0,
    `SunPosition` DOUBLE NOT NULL DEFAULT 0.0,
    `EstateSkipScripts` TINYINT(4) NOT NULL DEFAULT 0,
    `BillableFactor` FLOAT NOT NULL DEFAULT 1.0,
    `PublicAccess` TINYINT(4) NOT NULL DEFAULT 1,
    `AbuseEmail` VARCHAR(255) NOT NULL DEFAULT '',
    `EstateOwner` CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    `DenyMinors` TINYINT(4) NOT NULL DEFAULT 0,
    `AllowLandmark` TINYINT(4) NOT NULL DEFAULT 1,
    `AllowParcelChanges` TINYINT(4) NOT NULL DEFAULT 1,
    `AllowSetHome` TINYINT(4) NOT NULL DEFAULT 1,
    `AllowEnviromentOverride` TINYINT(4) NOT NULL DEFAULT 0,
    KEY `estate_settings_owner` (`EstateOwner`),
    KEY `estate_settings_name` (`EstateName`)
) ENGINE=InnoDB AUTO_INCREMENT=101 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- EstateStore: Region-Estate mapping
CREATE TABLE `estate_map` (
    `RegionID` CHAR(36) NOT NULL PRIMARY KEY,
    `EstateID` INT UNSIGNED NOT NULL,
    KEY `estate_map_estateid` (`EstateID`),
    FOREIGN KEY (`EstateID`) REFERENCES `estate_settings`(`EstateID`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- EstateStore: Estate managers
CREATE TABLE `estate_managers` (
    `EstateID` INT UNSIGNED NOT NULL,
    `uuid` CHAR(36) NOT NULL,
    PRIMARY KEY (`EstateID`, `uuid`),
    KEY `estate_managers_estateid` (`EstateID`),
    FOREIGN KEY (`EstateID`) REFERENCES `estate_settings`(`EstateID`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- EstateStore: Estate users
CREATE TABLE `estate_users` (
    `EstateID` INT UNSIGNED NOT NULL,
    `uuid` CHAR(36) NOT NULL,
    PRIMARY KEY (`EstateID`, `uuid`),
    KEY `estate_users_estateid` (`EstateID`),
    FOREIGN KEY (`EstateID`) REFERENCES `estate_settings`(`EstateID`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- EstateStore: Estate groups
CREATE TABLE `estate_groups` (
    `EstateID` INT UNSIGNED NOT NULL,
    `uuid` CHAR(36) NOT NULL,
    PRIMARY KEY (`EstateID`, `uuid`),
    KEY `estate_groups_estateid` (`EstateID`),
    FOREIGN KEY (`EstateID`) REFERENCES `estate_settings`(`EstateID`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- EstateStore: Estate bans
CREATE TABLE `estateban` (
    `EstateID` INT UNSIGNED NOT NULL,
    `bannedUUID` VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    `bannedIp` VARCHAR(16) NOT NULL DEFAULT '',
    `bannedIpHostMask` VARCHAR(16) NOT NULL DEFAULT '',
    `bannedNameMask` VARCHAR(64) NOT NULL DEFAULT '',
    `banningUUID` VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    `banTime` INT NOT NULL DEFAULT 0,
    PRIMARY KEY (`EstateID`, `bannedUUID`),
    KEY `estateban_estateid` (`EstateID`),
    FOREIGN KEY (`EstateID`) REFERENCES `estate_settings`(`EstateID`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- UserProfiles: Main profile data
CREATE TABLE `userprofile` (
    `useruuid` VARCHAR(36) NOT NULL PRIMARY KEY,
    `profilePartner` VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    `profileAllowPublish` TINYINT(4) NOT NULL DEFAULT 1,
    `profileMaturePublish` TINYINT(4) NOT NULL DEFAULT 1,
    `profileURL` VARCHAR(255) NOT NULL DEFAULT '',
    `profileWantToMask` INT NOT NULL DEFAULT 0,
    `profileWantToText` TEXT NOT NULL,
    `profileSkillsMask` INT NOT NULL DEFAULT 0,
    `profileSkillsText` TEXT NOT NULL,
    `profileLanguages` TEXT NOT NULL,
    `profileImage` VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    `profileAboutText` TEXT NOT NULL,
    `profileFirstImage` VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    `profileFirstText` TEXT NOT NULL,
    KEY `userprofile_useruuid` (`useruuid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- UserProfiles: User picks
CREATE TABLE `userpicks` (
    `pickuuid` VARCHAR(36) NOT NULL PRIMARY KEY,
    `creatoruuid` VARCHAR(36) NOT NULL,
    `toppick` TINYINT(4) NOT NULL DEFAULT 0,
    `parceluuid` VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    `name` VARCHAR(255) NOT NULL DEFAULT '',
    `description` TEXT NOT NULL,
    `snapshotuuid` VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    `user` VARCHAR(255) NOT NULL DEFAULT '',
    `originalname` VARCHAR(255) NOT NULL DEFAULT '',
    `simname` VARCHAR(255) NOT NULL DEFAULT '',
    `posglobal` VARCHAR(255) NOT NULL DEFAULT '',
    `sortorder` INT NOT NULL DEFAULT 0,
    `enabled` TINYINT(4) NOT NULL DEFAULT 1,
    `gatekeeper` VARCHAR(255) DEFAULT NULL,
    KEY `userpicks_creatoruuid` (`creatoruuid`),
    KEY `userpicks_toppick` (`toppick`),
    KEY `userpicks_enabled` (`enabled`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- UserProfiles: User classifieds
CREATE TABLE `userclassifieds` (
    `classifieduuid` VARCHAR(36) NOT NULL PRIMARY KEY,
    `creatoruuid` VARCHAR(36) NOT NULL,
    `creationdate` INT NOT NULL DEFAULT 0,
    `expirationdate` INT NOT NULL DEFAULT 0,
    `category` VARCHAR(20) NOT NULL DEFAULT '0',
    `name` VARCHAR(255) NOT NULL DEFAULT '',
    `description` TEXT NOT NULL,
    `parceluuid` VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    `parentestate` INT NOT NULL DEFAULT 0,
    `snapshotuuid` VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    `simname` VARCHAR(255) NOT NULL DEFAULT '',
    `posglobal` VARCHAR(255) NOT NULL DEFAULT '',
    `parcelname` VARCHAR(255) NOT NULL DEFAULT '',
    `classifiedflags` INT NOT NULL DEFAULT 0,
    `priceforlisting` INT NOT NULL DEFAULT 0,
    KEY `userclassifieds_creatoruuid` (`creatoruuid`),
    KEY `userclassifieds_category` (`category`),
    KEY `userclassifieds_expirationdate` (`expirationdate`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- UserProfiles: User notes
CREATE TABLE `usernotes` (
    `useruuid` VARCHAR(36) NOT NULL,
    `targetuuid` VARCHAR(36) NOT NULL,
    `notes` TEXT NOT NULL,
    PRIMARY KEY (`useruuid`, `targetuuid`),
    KEY `usernotes_useruuid` (`useruuid`),
    KEY `usernotes_targetuuid` (`targetuuid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- UserProfiles: User settings
CREATE TABLE `usersettings` (
    `useruuid` VARCHAR(36) NOT NULL PRIMARY KEY,
    `imviaemail` TINYINT(4) NOT NULL DEFAULT 0,
    `visible` TINYINT(4) NOT NULL DEFAULT 1,
    `email` VARCHAR(254) NOT NULL DEFAULT '',
    KEY `usersettings_useruuid` (`useruuid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- AgentPrefs: User agent preferences
CREATE TABLE `AgentPrefs` (
    `PrincipalID` CHAR(36) NOT NULL PRIMARY KEY,
    `AccessPrefs` CHAR(2) NOT NULL DEFAULT 'M',
    `HoverHeight` DOUBLE(30, 27) NOT NULL DEFAULT 0,
    `Language` CHAR(5) NOT NULL DEFAULT 'en-us',
    `LanguageIsPublic` TINYINT(4) NOT NULL DEFAULT 1,
    `PermEveryone` INT(6) NOT NULL DEFAULT 0,
    `PermGroup` INT(6) NOT NULL DEFAULT 0,
    `PermNextOwner` INT(6) NOT NULL DEFAULT 532480,
    UNIQUE KEY `PrincipalID` (`PrincipalID`),
    KEY `AgentPrefs_PrincipalID` (`PrincipalID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Insert default estate
INSERT INTO `estate_settings` (`EstateID`, `EstateName`, `EstateOwner`) 
VALUES (100, 'Default Estate', '00000000-0000-0000-0000-000000000000') 
ON DUPLICATE KEY UPDATE `EstateName` = VALUES(`EstateName`);