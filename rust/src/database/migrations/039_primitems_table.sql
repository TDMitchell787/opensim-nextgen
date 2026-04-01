-- Migration 039: Create primitems table for prim inventory (scripts, notecards, objects in Contents tab)
-- This table was missing from the original migration set but is required by C# OpenSim RegionStore

CREATE TABLE IF NOT EXISTS primitems (
    itemid uuid NOT NULL PRIMARY KEY,
    primid uuid NOT NULL,
    assetid uuid,
    parentfolderid uuid,
    invtype integer DEFAULT 0,
    assettype integer DEFAULT 0,
    name varchar(255) DEFAULT '',
    description varchar(255) DEFAULT '',
    creationdate integer DEFAULT 0,
    creatorid varchar(255) DEFAULT '',
    ownerid varchar(255) DEFAULT '',
    lastownerid varchar(255) DEFAULT '',
    groupid varchar(255) DEFAULT '',
    nextpermissions integer DEFAULT 0,
    currentpermissions integer DEFAULT 0,
    basepermissions integer DEFAULT 0,
    everyonepermissions integer DEFAULT 0,
    grouppermissions integer DEFAULT 0,
    flags integer DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_primitems_primid ON primitems(primid);
CREATE INDEX IF NOT EXISTS idx_primitems_assettype ON primitems(primid, assettype);
