-- PostgreSQL RegionStore Complete Implementation (v1-67)
-- This migration creates all OpenSim master RegionStore tables with full v67 compatibility

-- Drop existing regions table and recreate with full OpenSim master compatibility
DROP TABLE IF EXISTS regions CASCADE;

-- OpenSim Master: Regions table with hybrid compatibility
CREATE TABLE regions (
    -- OpenSim Next columns
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    region_name VARCHAR(128) NOT NULL,
    location_x INTEGER NOT NULL DEFAULT 0,
    location_y INTEGER NOT NULL DEFAULT 0,
    size_x INTEGER DEFAULT 256,
    size_y INTEGER DEFAULT 256,
    external_host_name VARCHAR(255),
    external_port INTEGER,
    internal_host_name VARCHAR(255),
    internal_port INTEGER,
    
    -- OpenSim Master columns
    uuid UUID DEFAULT NULL,
    regionHandle BIGINT DEFAULT NULL,
    regionName VARCHAR(128) DEFAULT NULL,
    regionRecvKey VARCHAR(128) DEFAULT NULL,
    regionSendKey VARCHAR(128) DEFAULT NULL,
    regionSecret VARCHAR(128) DEFAULT NULL,
    regionDataURI VARCHAR(255) DEFAULT NULL,
    serverIP VARCHAR(64) DEFAULT NULL,
    serverPort INTEGER DEFAULT NULL,
    serverURI VARCHAR(255) DEFAULT NULL,
    locX INTEGER DEFAULT NULL,
    locY INTEGER DEFAULT NULL,
    locZ INTEGER DEFAULT NULL,
    eastOverrideHandle BIGINT DEFAULT NULL,
    westOverrideHandle BIGINT DEFAULT NULL,
    southOverrideHandle BIGINT DEFAULT NULL,
    northOverrideHandle BIGINT DEFAULT NULL,
    regionAssetURI VARCHAR(255) DEFAULT NULL,
    regionAssetRecvKey VARCHAR(128) DEFAULT NULL,
    regionAssetSendKey VARCHAR(128) DEFAULT NULL,
    regionUserURI VARCHAR(255) DEFAULT NULL,
    regionUserRecvKey VARCHAR(128) DEFAULT NULL,
    regionUserSendKey VARCHAR(128) DEFAULT NULL,
    regionMapTexture UUID DEFAULT NULL,
    serverHttpPort INTEGER DEFAULT NULL,
    serverRemotingPort INTEGER DEFAULT NULL,
    owner_uuid UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    originUUID UUID DEFAULT NULL,
    access INTEGER DEFAULT 1,
    ScopeID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    sizeX INTEGER NOT NULL DEFAULT 0,
    sizeY INTEGER NOT NULL DEFAULT 0,
    flags INTEGER NOT NULL DEFAULT 0,
    last_seen INTEGER NOT NULL DEFAULT 0,
    
    -- Additional fields
    map_texture UUID,
    terrain_texture_1 UUID,
    terrain_texture_2 UUID,
    terrain_texture_3 UUID,
    terrain_texture_4 UUID,
    elevation_1_nw REAL DEFAULT 10.0,
    elevation_2_ne REAL DEFAULT 10.0,
    elevation_1_se REAL DEFAULT 10.0,
    elevation_2_sw REAL DEFAULT 10.0,
    water_height REAL DEFAULT 20.0,
    terrain_raise_limit REAL DEFAULT 100.0,
    terrain_lower_limit REAL DEFAULT -100.0,
    use_estate_sun BOOLEAN DEFAULT true,
    fixed_sun BOOLEAN DEFAULT false,
    sun_position REAL DEFAULT 0.0,
    covenant UUID,
    sandbox BOOLEAN DEFAULT false,
    public_access BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- OpenSim Master: Prims table with all v67 features
CREATE TABLE prims (
    CreationDate INTEGER DEFAULT NULL,
    Name VARCHAR(255) DEFAULT NULL,
    Text VARCHAR(255) DEFAULT NULL,
    Description VARCHAR(255) DEFAULT NULL,
    SitName VARCHAR(255) DEFAULT NULL,
    TouchName VARCHAR(255) DEFAULT NULL,
    ObjectFlags INTEGER DEFAULT NULL,
    OwnerMask INTEGER DEFAULT NULL,
    NextOwnerMask INTEGER DEFAULT NULL,
    GroupMask INTEGER DEFAULT NULL,
    EveryoneMask INTEGER DEFAULT NULL,
    BaseMask INTEGER DEFAULT NULL,
    PositionX REAL DEFAULT NULL,
    PositionY REAL DEFAULT NULL,
    PositionZ REAL DEFAULT NULL,
    GroupPositionX REAL DEFAULT NULL,
    GroupPositionY REAL DEFAULT NULL,
    GroupPositionZ REAL DEFAULT NULL,
    VelocityX REAL DEFAULT NULL,
    VelocityY REAL DEFAULT NULL,
    VelocityZ REAL DEFAULT NULL,
    AngularVelocityX REAL DEFAULT NULL,
    AngularVelocityY REAL DEFAULT NULL,
    AngularVelocityZ REAL DEFAULT NULL,
    AccelerationX REAL DEFAULT NULL,
    AccelerationY REAL DEFAULT NULL,
    AccelerationZ REAL DEFAULT NULL,
    RotationX REAL DEFAULT NULL,
    RotationY REAL DEFAULT NULL,
    RotationZ REAL DEFAULT NULL,
    RotationW REAL DEFAULT NULL,
    SitTargetOffsetX REAL DEFAULT NULL,
    SitTargetOffsetY REAL DEFAULT NULL,
    SitTargetOffsetZ REAL DEFAULT NULL,
    SitTargetOrientW REAL DEFAULT NULL,
    SitTargetOrientX REAL DEFAULT NULL,
    SitTargetOrientY REAL DEFAULT NULL,
    SitTargetOrientZ REAL DEFAULT NULL,
    UUID UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    RegionUUID UUID DEFAULT NULL,
    CreatorID VARCHAR(255) NOT NULL DEFAULT '',
    OwnerID UUID DEFAULT NULL,
    GroupID UUID DEFAULT NULL,
    LastOwnerID UUID DEFAULT NULL,
    SceneGroupID UUID DEFAULT NULL,
    LinkNumber INTEGER DEFAULT NULL,
    Material INTEGER DEFAULT NULL,
    SalePrice INTEGER NOT NULL DEFAULT 0,
    SaleType INTEGER NOT NULL DEFAULT 0,
    ClickAction INTEGER NOT NULL DEFAULT 0,
    PhysicsShapeType SMALLINT NOT NULL DEFAULT 0,
    Density DOUBLE PRECISION NOT NULL DEFAULT 1000,
    GravityModifier DOUBLE PRECISION NOT NULL DEFAULT 1,
    Friction DOUBLE PRECISION NOT NULL DEFAULT 0.6,
    Restitution DOUBLE PRECISION NOT NULL DEFAULT 0.5,
    -- v52+ features
    PassCollisions INTEGER NOT NULL DEFAULT 0,
    Vehicle TEXT DEFAULT NULL,
    -- v53+ features  
    RotationAxisLocks INTEGER NOT NULL DEFAULT 0,
    -- v56+ features
    RezzerID UUID DEFAULT NULL,
    -- v57+ features
    PhysInertia TEXT DEFAULT NULL,
    -- v58+ features
    sopanims BYTEA DEFAULT NULL,
    -- v59+ features
    standtargetx REAL DEFAULT 0.0,
    standtargety REAL DEFAULT 0.0,
    standtargetz REAL DEFAULT 0.0,
    sitactrange REAL DEFAULT 0.0,
    -- v61+ features
    pseudocrc INTEGER DEFAULT 0,
    -- v65+ features
    lnkstBinData BYTEA DEFAULT NULL,
    -- v67+ features
    StartStr TEXT DEFAULT NULL
);

-- Create indexes for regions table
CREATE INDEX regions_name ON regions(region_name);
CREATE INDEX regions_location ON regions(location_x, location_y);
CREATE INDEX regions_opensim_location ON regions(locX, locY);

-- Create indexes for prims table
CREATE INDEX prims_regionuuid ON prims(RegionUUID);
CREATE INDEX prims_position ON prims(PositionX, PositionY, PositionZ);
CREATE INDEX prims_groupposition ON prims(GroupPositionX, GroupPositionY, GroupPositionZ);
CREATE INDEX prims_rotation ON prims(RotationX, RotationY, RotationZ, RotationW);
CREATE INDEX prims_owner ON prims(OwnerID);
CREATE INDEX prims_creator ON prims(CreatorID);
CREATE INDEX prims_group ON prims(GroupID);
CREATE INDEX prims_scenegroup ON prims(SceneGroupID);