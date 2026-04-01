-- PostgreSQL Complete Stores Final Implementation
-- This migration creates all remaining store systems for complete OpenSim master compatibility

-- Drop existing estates table and recreate with full functionality
DROP TABLE IF EXISTS estates CASCADE;

-- EstateStore: Estate Settings (main configuration)
CREATE TABLE estate_settings (
    EstateID SERIAL PRIMARY KEY,
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
    EstateOwner UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    DenyMinors INTEGER NOT NULL DEFAULT 0,
    AllowLandmark INTEGER NOT NULL DEFAULT 1,
    AllowParcelChanges INTEGER NOT NULL DEFAULT 1,
    AllowSetHome INTEGER NOT NULL DEFAULT 1,
    AllowEnviromentOverride INTEGER NOT NULL DEFAULT 0
);

-- EstateStore: Region-Estate mapping
CREATE TABLE estate_map (
    RegionID UUID NOT NULL PRIMARY KEY,
    EstateID INTEGER NOT NULL,
    FOREIGN KEY (EstateID) REFERENCES estate_settings(EstateID) ON DELETE CASCADE
);

-- EstateStore: Estate managers
CREATE TABLE estate_managers (
    EstateID INTEGER NOT NULL,
    uuid UUID NOT NULL,
    PRIMARY KEY (EstateID, uuid),
    FOREIGN KEY (EstateID) REFERENCES estate_settings(EstateID) ON DELETE CASCADE
);

-- EstateStore: Estate users
CREATE TABLE estate_users (
    EstateID INTEGER NOT NULL,
    uuid UUID NOT NULL,
    PRIMARY KEY (EstateID, uuid),
    FOREIGN KEY (EstateID) REFERENCES estate_settings(EstateID) ON DELETE CASCADE
);

-- EstateStore: Estate groups
CREATE TABLE estate_groups (
    EstateID INTEGER NOT NULL,
    uuid UUID NOT NULL,
    PRIMARY KEY (EstateID, uuid),
    FOREIGN KEY (EstateID) REFERENCES estate_settings(EstateID) ON DELETE CASCADE
);

-- EstateStore: Estate bans
CREATE TABLE estateban (
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

-- UserProfiles: Main profile data
CREATE TABLE userprofile (
    useruuid VARCHAR(36) NOT NULL PRIMARY KEY,
    profilePartner VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    profileAllowPublish INTEGER NOT NULL DEFAULT 1,
    profileMaturePublish INTEGER NOT NULL DEFAULT 1,
    profileURL VARCHAR(255) NOT NULL DEFAULT '',
    profileWantToMask INTEGER NOT NULL DEFAULT 0,
    profileWantToText TEXT NOT NULL DEFAULT '',
    profileSkillsMask INTEGER NOT NULL DEFAULT 0,
    profileSkillsText TEXT NOT NULL DEFAULT '',
    profileLanguages TEXT NOT NULL DEFAULT '',
    profileImage VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    profileAboutText TEXT NOT NULL DEFAULT '',
    profileFirstImage VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    profileFirstText TEXT NOT NULL DEFAULT ''
);

-- UserProfiles: User picks
CREATE TABLE userpicks (
    pickuuid VARCHAR(36) NOT NULL PRIMARY KEY,
    creatoruuid VARCHAR(36) NOT NULL,
    toppick INTEGER NOT NULL DEFAULT 0,
    parceluuid VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    name VARCHAR(255) NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    snapshotuuid VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    "user" VARCHAR(255) NOT NULL DEFAULT '',
    originalname VARCHAR(255) NOT NULL DEFAULT '',
    simname VARCHAR(255) NOT NULL DEFAULT '',
    posglobal VARCHAR(255) NOT NULL DEFAULT '',
    sortorder INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1,
    gatekeeper VARCHAR(255) DEFAULT NULL
);

-- UserProfiles: User classifieds
CREATE TABLE userclassifieds (
    classifieduuid VARCHAR(36) NOT NULL PRIMARY KEY,
    creatoruuid VARCHAR(36) NOT NULL,
    creationdate INTEGER NOT NULL DEFAULT 0,
    expirationdate INTEGER NOT NULL DEFAULT 0,
    category VARCHAR(20) NOT NULL DEFAULT '0',
    name VARCHAR(255) NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    parceluuid VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    parentestate INTEGER NOT NULL DEFAULT 0,
    snapshotuuid VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    simname VARCHAR(255) NOT NULL DEFAULT '',
    posglobal VARCHAR(255) NOT NULL DEFAULT '',
    parcelname VARCHAR(255) NOT NULL DEFAULT '',
    classifiedflags INTEGER NOT NULL DEFAULT 0,
    priceforlisting INTEGER NOT NULL DEFAULT 0
);

-- UserProfiles: User notes
CREATE TABLE usernotes (
    useruuid VARCHAR(36) NOT NULL,
    targetuuid VARCHAR(36) NOT NULL,
    notes TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (useruuid, targetuuid)
);

-- UserProfiles: User settings
CREATE TABLE usersettings (
    useruuid VARCHAR(36) NOT NULL PRIMARY KEY,
    imviaemail INTEGER NOT NULL DEFAULT 0,
    visible INTEGER NOT NULL DEFAULT 1,
    email VARCHAR(254) NOT NULL DEFAULT ''
);

-- AgentPrefs: User agent preferences
CREATE TABLE AgentPrefs (
    PrincipalID UUID NOT NULL PRIMARY KEY,
    AccessPrefs CHAR(2) NOT NULL DEFAULT 'M',
    HoverHeight REAL NOT NULL DEFAULT 0.0,
    Language CHAR(5) NOT NULL DEFAULT 'en-us',
    LanguageIsPublic INTEGER NOT NULL DEFAULT 1,
    PermEveryone INTEGER NOT NULL DEFAULT 0,
    PermGroup INTEGER NOT NULL DEFAULT 0,
    PermNextOwner INTEGER NOT NULL DEFAULT 532480
);

-- Set initial value for estate_settings sequence
ALTER SEQUENCE estate_settings_estateid_seq RESTART WITH 101;

-- Insert default estate
INSERT INTO estate_settings (EstateID, EstateName, EstateOwner) 
VALUES (100, 'Default Estate', '00000000-0000-0000-0000-000000000000') 
ON CONFLICT (EstateID) DO NOTHING;

-- Create indexes for estate system
CREATE INDEX estate_map_estateid ON estate_map(EstateID);
CREATE INDEX estate_managers_estateid ON estate_managers(EstateID);
CREATE INDEX estate_users_estateid ON estate_users(EstateID);
CREATE INDEX estate_groups_estateid ON estate_groups(EstateID);
CREATE INDEX estateban_estateid ON estateban(EstateID);
CREATE INDEX estate_settings_owner ON estate_settings(EstateOwner);
CREATE INDEX estate_settings_name ON estate_settings(EstateName);

-- Create indexes for user profiles
CREATE INDEX userprofile_useruuid ON userprofile(useruuid);
CREATE INDEX userpicks_creatoruuid ON userpicks(creatoruuid);
CREATE INDEX userpicks_toppick ON userpicks(toppick);
CREATE INDEX userpicks_enabled ON userpicks(enabled);
CREATE INDEX userclassifieds_creatoruuid ON userclassifieds(creatoruuid);
CREATE INDEX userclassifieds_category ON userclassifieds(category);
CREATE INDEX userclassifieds_expirationdate ON userclassifieds(expirationdate);
CREATE INDEX usernotes_useruuid ON usernotes(useruuid);
CREATE INDEX usernotes_targetuuid ON usernotes(targetuuid);
CREATE INDEX usersettings_useruuid ON usersettings(useruuid);

-- Create index for agent prefs
CREATE INDEX AgentPrefs_PrincipalID ON AgentPrefs(PrincipalID);