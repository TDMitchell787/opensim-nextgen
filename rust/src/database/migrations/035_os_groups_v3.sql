-- os_groups_Store Migration v1-3: Groups System
-- This migration creates the complete groups system for OpenSim

-- Groups table
CREATE TABLE IF NOT EXISTS os_groups_groups (
    GroupID CHAR(36) NOT NULL PRIMARY KEY,
    Location VARCHAR(255) NOT NULL DEFAULT '',
    Name VARCHAR(255) NOT NULL DEFAULT '',
    Charter TEXT NOT NULL DEFAULT '',
    InsigniaID CHAR(36) NOT NULL DEFAULT '',
    FounderID CHAR(36) NOT NULL DEFAULT '',
    MembershipFee INTEGER NOT NULL DEFAULT 0,
    OpenEnrollment VARCHAR(255) NOT NULL DEFAULT '',
    ShowInList INTEGER NOT NULL DEFAULT 0,
    AllowPublish INTEGER NOT NULL DEFAULT 0,
    MaturePublish INTEGER NOT NULL DEFAULT 0,
    OwnerRoleID CHAR(36) NOT NULL DEFAULT ''
);

-- Group membership table
CREATE TABLE IF NOT EXISTS os_groups_membership (
    GroupID CHAR(36) NOT NULL DEFAULT '',
    PrincipalID VARCHAR(255) NOT NULL DEFAULT '',
    SelectedRoleID CHAR(36) NOT NULL DEFAULT '',
    Contribution INTEGER NOT NULL DEFAULT 0,
    ListInProfile INTEGER NOT NULL DEFAULT 1,
    AcceptNotices INTEGER NOT NULL DEFAULT 1,
    AccessToken CHAR(36) NOT NULL DEFAULT '',
    PRIMARY KEY (GroupID, PrincipalID)
);

-- Group roles table
CREATE TABLE IF NOT EXISTS os_groups_roles (
    GroupID CHAR(36) NOT NULL DEFAULT '',
    RoleID CHAR(36) NOT NULL DEFAULT '',
    Name VARCHAR(255) NOT NULL DEFAULT '',
    Description VARCHAR(255) NOT NULL DEFAULT '',
    Title VARCHAR(255) NOT NULL DEFAULT '',
    Powers BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (GroupID, RoleID)
);

-- Group role membership table
CREATE TABLE IF NOT EXISTS os_groups_rolemembership (
    GroupID CHAR(36) NOT NULL DEFAULT '',
    RoleID CHAR(36) NOT NULL DEFAULT '',
    PrincipalID VARCHAR(255) NOT NULL DEFAULT '',
    PRIMARY KEY (GroupID, RoleID, PrincipalID)
);

-- Group invites table
CREATE TABLE IF NOT EXISTS os_groups_invites (
    InviteID CHAR(36) NOT NULL PRIMARY KEY,
    GroupID CHAR(36) NOT NULL DEFAULT '',
    RoleID CHAR(36) NOT NULL DEFAULT '',
    PrincipalID VARCHAR(255) NOT NULL DEFAULT '',
    TMStamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Group notices table
CREATE TABLE IF NOT EXISTS os_groups_notices (
    GroupID CHAR(36) NOT NULL DEFAULT '',
    NoticeID CHAR(36) NOT NULL PRIMARY KEY,
    TMStamp INTEGER NOT NULL DEFAULT 0,
    FromName VARCHAR(255) NOT NULL DEFAULT '',
    Subject VARCHAR(255) NOT NULL DEFAULT '',
    Message TEXT NOT NULL DEFAULT '',
    HasAttachment INTEGER NOT NULL DEFAULT 0,
    AttachmentType INTEGER NOT NULL DEFAULT 0,
    AttachmentName VARCHAR(128) NOT NULL DEFAULT '',
    AttachmentItemID CHAR(36) NOT NULL DEFAULT '',
    AttachmentOwnerID VARCHAR(255) NOT NULL DEFAULT ''
);

-- Group principals table
CREATE TABLE IF NOT EXISTS os_groups_principals (
    PrincipalID VARCHAR(255) NOT NULL PRIMARY KEY,
    ActiveGroupID CHAR(36) NOT NULL DEFAULT ''
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS os_groups_groups_name ON os_groups_groups(Name);
CREATE INDEX IF NOT EXISTS os_groups_membership_principalid ON os_groups_membership(PrincipalID);
CREATE INDEX IF NOT EXISTS os_groups_roles_groupid ON os_groups_roles(GroupID);
CREATE INDEX IF NOT EXISTS os_groups_rolemembership_principalid ON os_groups_rolemembership(PrincipalID);
CREATE INDEX IF NOT EXISTS os_groups_invites_groupid ON os_groups_invites(GroupID);
CREATE INDEX IF NOT EXISTS os_groups_notices_groupid ON os_groups_notices(GroupID);
CREATE INDEX IF NOT EXISTS os_groups_notices_tmstamp ON os_groups_notices(TMStamp);
CREATE INDEX IF NOT EXISTS os_groups_principals_principalid ON os_groups_principals(PrincipalID);