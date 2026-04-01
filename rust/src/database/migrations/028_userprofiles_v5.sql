-- UserProfiles Migration v1-5: Complete User Profiles System
-- This migration creates the user profiles system for picks, classifieds, notes, and preferences

-- 1. User profiles table (main profile data)
CREATE TABLE IF NOT EXISTS userprofile (
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

-- 2. User picks table (favorite places)
CREATE TABLE IF NOT EXISTS userpicks (
    pickuuid VARCHAR(36) NOT NULL PRIMARY KEY,
    creatoruuid VARCHAR(36) NOT NULL,
    toppick INTEGER NOT NULL DEFAULT 0,
    parceluuid VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    name VARCHAR(255) NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    snapshotuuid VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    user VARCHAR(255) NOT NULL DEFAULT '',
    originalname VARCHAR(255) NOT NULL DEFAULT '',
    simname VARCHAR(255) NOT NULL DEFAULT '',
    posglobal VARCHAR(255) NOT NULL DEFAULT '',
    sortorder INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1,
    gatekeeper VARCHAR(255) DEFAULT NULL
);

-- 3. User classifieds table (classified ads)
CREATE TABLE IF NOT EXISTS userclassifieds (
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

-- 4. User notes table (user-to-user notes)
CREATE TABLE IF NOT EXISTS usernotes (
    useruuid VARCHAR(36) NOT NULL,
    targetuuid VARCHAR(36) NOT NULL,
    notes TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (useruuid, targetuuid)
);

-- 5. User settings table (user preferences)
CREATE TABLE IF NOT EXISTS usersettings (
    useruuid VARCHAR(36) NOT NULL PRIMARY KEY,
    imviaemail INTEGER NOT NULL DEFAULT 0,
    visible INTEGER NOT NULL DEFAULT 1,
    email VARCHAR(254) NOT NULL DEFAULT ''
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS userprofile_useruuid ON userprofile(useruuid);
CREATE INDEX IF NOT EXISTS userpicks_creatoruuid ON userpicks(creatoruuid);
CREATE INDEX IF NOT EXISTS userpicks_toppick ON userpicks(toppick);
CREATE INDEX IF NOT EXISTS userpicks_enabled ON userpicks(enabled);
CREATE INDEX IF NOT EXISTS userclassifieds_creatoruuid ON userclassifieds(creatoruuid);
CREATE INDEX IF NOT EXISTS userclassifieds_category ON userclassifieds(category);
CREATE INDEX IF NOT EXISTS userclassifieds_expirationdate ON userclassifieds(expirationdate);
CREATE INDEX IF NOT EXISTS usernotes_useruuid ON usernotes(useruuid);
CREATE INDEX IF NOT EXISTS usernotes_targetuuid ON usernotes(targetuuid);
CREATE INDEX IF NOT EXISTS usersettings_useruuid ON usersettings(useruuid);