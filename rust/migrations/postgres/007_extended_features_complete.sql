-- PostgreSQL Extended Features Complete Implementation
-- This migration creates all remaining advanced store systems

-- XAssetStore: Extended asset metadata
CREATE TABLE xassetsmeta (
    ID UUID NOT NULL PRIMARY KEY,
    Hash CHAR(32) NOT NULL,
    Name VARCHAR(64) NOT NULL,
    Description VARCHAR(64) NOT NULL,
    AssetType INTEGER NOT NULL,
    Local INTEGER NOT NULL,
    Temporary INTEGER NOT NULL,
    CreateTime INTEGER NOT NULL,
    AccessTime INTEGER NOT NULL,
    AssetFlags INTEGER NOT NULL,
    CreatorID VARCHAR(128) NOT NULL
);

-- XAssetStore: Extended asset data
CREATE TABLE xassetsdata (
    Hash CHAR(32) NOT NULL PRIMARY KEY,
    Data BYTEA NOT NULL
);

-- GridStore: Grid regions (for grid service)
-- Note: We already have regions table, so this creates grid-specific view
CREATE VIEW grid_regions AS 
SELECT 
    uuid, regionHandle, regionName, regionRecvKey, regionSendKey,
    regionSecret, regionDataURI, serverIP, serverPort, serverURI,
    locX, locY, locZ, eastOverrideHandle, westOverrideHandle,
    southOverrideHandle, northOverrideHandle, regionAssetURI,
    regionAssetRecvKey, regionAssetSendKey, regionUserURI,
    regionUserRecvKey, regionUserSendKey, regionMapTexture,
    serverHttpPort, serverRemotingPort, owner_uuid, originUUID,
    access, ScopeID, sizeX, sizeY, flags, last_seen
FROM regions;

-- IM_Store: Offline instant messages
CREATE TABLE im_offline (
    ID SERIAL PRIMARY KEY,
    PrincipalID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    FromID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    Message TEXT NOT NULL,
    TMStamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- HGTravelStore: Hypergrid travel data
CREATE TABLE hg_traveling_data (
    SessionID VARCHAR(36) NOT NULL PRIMARY KEY,
    UserID VARCHAR(36) NOT NULL,
    GridExternalName VARCHAR(255) NOT NULL DEFAULT '',
    ServiceToken VARCHAR(255) NOT NULL DEFAULT '',
    ClientIPAddress VARCHAR(16) NOT NULL DEFAULT '',
    MyIPAddress VARCHAR(16) NOT NULL DEFAULT '',
    TMStamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- MuteListStore: User mute lists
CREATE TABLE MuteList (
    AgentID UUID NOT NULL,
    MuteID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    MuteName VARCHAR(64) NOT NULL DEFAULT '',
    MuteType INTEGER NOT NULL DEFAULT 1,
    MuteFlags INTEGER NOT NULL DEFAULT 0,
    Stamp INTEGER NOT NULL,
    PRIMARY KEY (AgentID, MuteID, MuteName)
);

-- os_groups_Store: Groups system
CREATE TABLE os_groups_groups (
    GroupID UUID NOT NULL PRIMARY KEY,
    Location VARCHAR(255) NOT NULL DEFAULT '',
    Name VARCHAR(255) NOT NULL DEFAULT '',
    Charter TEXT NOT NULL DEFAULT '',
    InsigniaID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    FounderID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    MembershipFee INTEGER NOT NULL DEFAULT 0,
    OpenEnrollment VARCHAR(255) NOT NULL DEFAULT '',
    ShowInList INTEGER NOT NULL DEFAULT 0,
    AllowPublish INTEGER NOT NULL DEFAULT 0,
    MaturePublish INTEGER NOT NULL DEFAULT 0,
    OwnerRoleID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
);

-- os_groups_Store: Group membership
CREATE TABLE os_groups_membership (
    GroupID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    PrincipalID VARCHAR(255) NOT NULL DEFAULT '',
    SelectedRoleID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    Contribution INTEGER NOT NULL DEFAULT 0,
    ListInProfile INTEGER NOT NULL DEFAULT 1,
    AcceptNotices INTEGER NOT NULL DEFAULT 1,
    AccessToken UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    PRIMARY KEY (GroupID, PrincipalID)
);

-- os_groups_Store: Group roles
CREATE TABLE os_groups_roles (
    GroupID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    RoleID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    Name VARCHAR(255) NOT NULL DEFAULT '',
    Description VARCHAR(255) NOT NULL DEFAULT '',
    Title VARCHAR(255) NOT NULL DEFAULT '',
    Powers BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (GroupID, RoleID)
);

-- os_groups_Store: Group role membership
CREATE TABLE os_groups_rolemembership (
    GroupID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    RoleID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    PrincipalID VARCHAR(255) NOT NULL DEFAULT '',
    PRIMARY KEY (GroupID, RoleID, PrincipalID)
);

-- os_groups_Store: Group invites
CREATE TABLE os_groups_invites (
    InviteID UUID NOT NULL PRIMARY KEY,
    GroupID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    RoleID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    PrincipalID VARCHAR(255) NOT NULL DEFAULT '',
    TMStamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- os_groups_Store: Group notices
CREATE TABLE os_groups_notices (
    GroupID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    NoticeID UUID NOT NULL PRIMARY KEY,
    TMStamp INTEGER NOT NULL DEFAULT 0,
    FromName VARCHAR(255) NOT NULL DEFAULT '',
    Subject VARCHAR(255) NOT NULL DEFAULT '',
    Message TEXT NOT NULL DEFAULT '',
    HasAttachment INTEGER NOT NULL DEFAULT 0,
    AttachmentType INTEGER NOT NULL DEFAULT 0,
    AttachmentName VARCHAR(128) NOT NULL DEFAULT '',
    AttachmentItemID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    AttachmentOwnerID VARCHAR(255) NOT NULL DEFAULT ''
);

-- os_groups_Store: Group principals
CREATE TABLE os_groups_principals (
    PrincipalID VARCHAR(255) NOT NULL PRIMARY KEY,
    ActiveGroupID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
);

-- LogStore: System logs
CREATE TABLE logs (
    logID SERIAL PRIMARY KEY,
    target VARCHAR(36) DEFAULT NULL,
    server VARCHAR(64) DEFAULT NULL,
    method VARCHAR(64) DEFAULT NULL,
    arguments VARCHAR(255) DEFAULT NULL,
    priority INTEGER DEFAULT NULL,
    message TEXT
);

-- Create indexes for extended asset system
CREATE INDEX xassetsmeta_hash ON xassetsmeta(Hash);
CREATE INDEX xassetsmeta_name ON xassetsmeta(Name);
CREATE INDEX xassetsmeta_assettype ON xassetsmeta(AssetType);
CREATE INDEX xassetsmeta_creatorid ON xassetsmeta(CreatorID);
CREATE INDEX xassetsdata_hash ON xassetsdata(Hash);

-- Create indexes for IM system
CREATE INDEX im_offline_principalid ON im_offline(PrincipalID);
CREATE INDEX im_offline_fromid ON im_offline(FromID);
CREATE INDEX im_offline_tmstamp ON im_offline(TMStamp);

-- Create indexes for hypergrid travel
CREATE INDEX hg_traveling_data_userid ON hg_traveling_data(UserID);
CREATE INDEX hg_traveling_data_sessionid ON hg_traveling_data(SessionID);
CREATE INDEX hg_traveling_data_tmstamp ON hg_traveling_data(TMStamp);

-- Create indexes for mute lists
CREATE INDEX MuteList_AgentID ON MuteList(AgentID);
CREATE INDEX MuteList_MuteID ON MuteList(MuteID);
CREATE INDEX MuteList_MuteName ON MuteList(MuteName);

-- Create indexes for groups system
CREATE INDEX os_groups_groups_name ON os_groups_groups(Name);
CREATE INDEX os_groups_membership_principalid ON os_groups_membership(PrincipalID);
CREATE INDEX os_groups_roles_groupid ON os_groups_roles(GroupID);
CREATE INDEX os_groups_rolemembership_principalid ON os_groups_rolemembership(PrincipalID);
CREATE INDEX os_groups_invites_groupid ON os_groups_invites(GroupID);
CREATE INDEX os_groups_notices_groupid ON os_groups_notices(GroupID);
CREATE INDEX os_groups_notices_tmstamp ON os_groups_notices(TMStamp);
CREATE INDEX os_groups_principals_principalid ON os_groups_principals(PrincipalID);

-- Create indexes for logging system
CREATE INDEX logs_target ON logs(target);
CREATE INDEX logs_server ON logs(server);
CREATE INDEX logs_method ON logs(method);
CREATE INDEX logs_priority ON logs(priority);