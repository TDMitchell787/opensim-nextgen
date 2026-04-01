-- EstateStore Migration v1-36: Complete Estate Management System
-- This migration creates the full estate management system with all tables and features

-- 1. Main estate configuration table
CREATE TABLE IF NOT EXISTS estate_settings (
    EstateID INTEGER PRIMARY KEY AUTOINCREMENT,
    EstateName VARCHAR(64) NOT NULL DEFAULT 'My Estate',
    AbuseEmailToEstateOwner INTEGER NOT NULL DEFAULT 1,
    DenyAnonymous INTEGER NOT NULL DEFAULT 0,
    ResetHomeOnTeleport INTEGER NOT NULL DEFAULT 0,
    FixedSun INTEGER NOT NULL DEFAULT 0,
    DenyTransacted INTEGER NOT NULL DEFAULT 0,
    BlockDwell INTEGER NOT NULL DEFAULT 0,
    DenyIdentified INTEGER NOT NULL DEFAULT 0,
    AllowVoice INTEGER NOT NULL DEFAULT 1,
    UseGlobalTime INTEGER NOT NULL DEFAULT 1,
    PricePerMeter INTEGER NOT NULL DEFAULT 1,
    TaxFree INTEGER NOT NULL DEFAULT 0,
    AllowDirectTeleport INTEGER NOT NULL DEFAULT 1,
    RedirectGridX INTEGER NOT NULL DEFAULT 0,
    RedirectGridY INTEGER NOT NULL DEFAULT 0,
    ParentEstateID INTEGER NOT NULL DEFAULT 0,
    SunPosition REAL NOT NULL DEFAULT 0.0,
    EstateSkipScripts INTEGER NOT NULL DEFAULT 0,
    BillableFactor REAL NOT NULL DEFAULT 1.0,
    PublicAccess INTEGER NOT NULL DEFAULT 1,
    AbuseEmail VARCHAR(255) NOT NULL DEFAULT '',
    EstateOwner CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    DenyMinors INTEGER NOT NULL DEFAULT 0,
    AllowLandmark INTEGER NOT NULL DEFAULT 1,
    AllowParcelChanges INTEGER NOT NULL DEFAULT 1,
    AllowSetHome INTEGER NOT NULL DEFAULT 1,
    AllowEnviromentOverride INTEGER NOT NULL DEFAULT 0
);

-- 2. Region-Estate mapping table
CREATE TABLE IF NOT EXISTS estate_map (
    RegionID CHAR(36) NOT NULL PRIMARY KEY,
    EstateID INTEGER NOT NULL,
    FOREIGN KEY (EstateID) REFERENCES estate_settings(EstateID) ON DELETE CASCADE
);

-- 3. Estate managers table (users who can manage the estate)
CREATE TABLE IF NOT EXISTS estate_managers (
    EstateID INTEGER NOT NULL,
    uuid CHAR(36) NOT NULL,
    PRIMARY KEY (EstateID, uuid),
    FOREIGN KEY (EstateID) REFERENCES estate_settings(EstateID) ON DELETE CASCADE
);

-- 4. Estate users table (users with special access)
CREATE TABLE IF NOT EXISTS estate_users (
    EstateID INTEGER NOT NULL,
    uuid CHAR(36) NOT NULL,
    PRIMARY KEY (EstateID, uuid),
    FOREIGN KEY (EstateID) REFERENCES estate_settings(EstateID) ON DELETE CASCADE
);

-- 5. Estate groups table (groups with special access)
CREATE TABLE IF NOT EXISTS estate_groups (
    EstateID INTEGER NOT NULL,
    uuid CHAR(36) NOT NULL,
    PRIMARY KEY (EstateID, uuid),
    FOREIGN KEY (EstateID) REFERENCES estate_settings(EstateID) ON DELETE CASCADE
);

-- 6. Estate ban table (banned users and IP addresses)
CREATE TABLE IF NOT EXISTS estateban (
    EstateID INTEGER NOT NULL,
    bannedUUID VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    bannedIp VARCHAR(16) NOT NULL DEFAULT '',
    bannedIpHostMask VARCHAR(16) NOT NULL DEFAULT '',
    bannedNameMask VARCHAR(64) NOT NULL DEFAULT '',
    banningUUID VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    banTime INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (EstateID, bannedUUID),
    FOREIGN KEY (EstateID) REFERENCES estate_settings(EstateID) ON DELETE CASCADE
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS estate_map_estateid ON estate_map(EstateID);
CREATE INDEX IF NOT EXISTS estate_managers_estateid ON estate_managers(EstateID);
CREATE INDEX IF NOT EXISTS estate_users_estateid ON estate_users(EstateID);
CREATE INDEX IF NOT EXISTS estate_groups_estateid ON estate_groups(EstateID);
CREATE INDEX IF NOT EXISTS estateban_estateid ON estateban(EstateID);
CREATE INDEX IF NOT EXISTS estate_settings_owner ON estate_settings(EstateOwner);
CREATE INDEX IF NOT EXISTS estate_settings_name ON estate_settings(EstateName);

-- Insert default estate if none exists (ID will start at 1 due to AUTOINCREMENT)
INSERT OR IGNORE INTO estate_settings 
    (EstateID, EstateName, EstateOwner) 
VALUES 
    (1, 'Default Estate', '00000000-0000-0000-0000-000000000000');