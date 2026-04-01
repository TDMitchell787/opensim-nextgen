-- PostgreSQL Social Systems Complete Implementation
-- This migration creates all social system tables with OpenSim master compatibility

-- OpenSim Master: Friends table
CREATE TABLE Friends (
    PrincipalID UUID NOT NULL,
    Friend VARCHAR(255) NOT NULL,
    Flags INTEGER NOT NULL DEFAULT 0,
    Offered VARCHAR(32) NOT NULL DEFAULT '0',
    PRIMARY KEY(PrincipalID, Friend)
);

-- OpenSim Master: Presence table
CREATE TABLE Presence (
    UserID VARCHAR(255) NOT NULL,
    RegionID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    SessionID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    SecureSessionID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    LastSeen TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY(UserID)
);

-- OpenSim Master: Avatars table
CREATE TABLE Avatars (
    PrincipalID UUID NOT NULL,
    Name VARCHAR(32) NOT NULL,
    Value TEXT NOT NULL DEFAULT '',
    PRIMARY KEY(PrincipalID, Name)
);

-- OpenSim Master: GridUser table
CREATE TABLE GridUser (
    UserID VARCHAR(255) NOT NULL PRIMARY KEY,
    HomeRegionID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    HomePosition VARCHAR(64) NOT NULL DEFAULT '<0,0,0>',
    HomeLookAt VARCHAR(64) NOT NULL DEFAULT '<0,0,0>',
    LastRegionID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    LastPosition VARCHAR(64) NOT NULL DEFAULT '<0,0,0>',
    LastLookAt VARCHAR(64) NOT NULL DEFAULT '<0,0,0>',
    Online CHAR(5) NOT NULL DEFAULT 'false',
    Login CHAR(16) NOT NULL DEFAULT '0',
    Logout CHAR(16) NOT NULL DEFAULT '0'
);

-- OpenSim Master: Land table
CREATE TABLE land (
    UUID UUID NOT NULL PRIMARY KEY,
    RegionUUID UUID DEFAULT NULL,
    LocalLandID INTEGER DEFAULT NULL,
    Bitmap BYTEA,
    Name VARCHAR(255) DEFAULT NULL,
    Description VARCHAR(255) DEFAULT NULL,
    OwnerUUID UUID DEFAULT NULL,
    IsGroupOwned INTEGER DEFAULT NULL,
    Area INTEGER DEFAULT NULL,
    AuctionID INTEGER DEFAULT NULL,
    Category INTEGER DEFAULT NULL,
    ClaimDate INTEGER DEFAULT NULL,
    ClaimPrice INTEGER DEFAULT NULL,
    GroupUUID UUID DEFAULT NULL,
    SalePrice INTEGER DEFAULT NULL,
    LandStatus INTEGER DEFAULT NULL,
    LandFlags INTEGER DEFAULT NULL,
    LandingType INTEGER DEFAULT NULL,
    MediaAutoScale INTEGER DEFAULT NULL,
    MediaTextureUUID UUID DEFAULT NULL,
    MediaURL VARCHAR(255) DEFAULT NULL,
    MusicURL VARCHAR(255) DEFAULT NULL,
    PassHours REAL DEFAULT NULL,
    PassPrice INTEGER DEFAULT NULL,
    SnapshotUUID UUID DEFAULT NULL,
    UserLocationX REAL DEFAULT NULL,
    UserLocationY REAL DEFAULT NULL,
    UserLocationZ REAL DEFAULT NULL,
    UserLookAtX REAL DEFAULT NULL,
    UserLookAtY REAL DEFAULT NULL,
    UserLookAtZ REAL DEFAULT NULL,
    AuthbuyerID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    OtherCleanTime INTEGER NOT NULL DEFAULT 0,
    Dwell INTEGER NOT NULL DEFAULT 0,
    MediaType VARCHAR(32) NOT NULL DEFAULT 'none/none',
    MediaDescription VARCHAR(255) NOT NULL DEFAULT '',
    MediaSize VARCHAR(16) NOT NULL DEFAULT '0,0',
    MediaLoop INTEGER NOT NULL DEFAULT 0,
    ObscureMusic INTEGER NOT NULL DEFAULT 0,
    ObscureMedia INTEGER NOT NULL DEFAULT 0,
    SeeAVs INTEGER NOT NULL DEFAULT 1,
    AnyAVSounds INTEGER NOT NULL DEFAULT 1,
    GroupAVSounds INTEGER NOT NULL DEFAULT 1,
    environment TEXT DEFAULT NULL
);

-- Create indexes for social system tables
CREATE INDEX Friends_PrincipalID ON Friends(PrincipalID);
CREATE INDEX Friends_Friend ON Friends(Friend);
CREATE INDEX Presence_UserID ON Presence(UserID);
CREATE INDEX Presence_RegionID ON Presence(RegionID);
CREATE INDEX Presence_SessionID ON Presence(SessionID);
CREATE INDEX Avatars_PrincipalID ON Avatars(PrincipalID);
CREATE INDEX GridUser_UserID ON GridUser(UserID);
CREATE INDEX GridUser_LastRegionID ON GridUser(LastRegionID);
CREATE INDEX GridUser_HomeRegionID ON GridUser(HomeRegionID);
CREATE INDEX land_regionuuid ON land(RegionUUID);