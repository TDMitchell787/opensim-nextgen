-- OpenSim-compatible inventory tables for PostgreSQL fresh installs
-- Also ensures all permission columns exist with proper defaults
-- Matches C# OpenSim InventoryStore Version 4 schema

CREATE TABLE IF NOT EXISTS inventoryfolders (
    folderid uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000' PRIMARY KEY,
    agentid uuid DEFAULT NULL,
    parentfolderid uuid DEFAULT NULL,
    foldername varchar(64) DEFAULT NULL,
    type smallint NOT NULL DEFAULT 0,
    version int NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_inventoryfolders_agentid ON inventoryfolders(agentid);
CREATE INDEX IF NOT EXISTS idx_inventoryfolders_parentfolderid ON inventoryfolders(parentfolderid);

CREATE TABLE IF NOT EXISTS inventoryitems (
    inventoryid uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000' PRIMARY KEY,
    assetid uuid DEFAULT NULL,
    assettype int DEFAULT NULL,
    parentfolderid uuid DEFAULT NULL,
    avatarid uuid DEFAULT NULL,
    inventoryname varchar(64) DEFAULT NULL,
    inventorydescription varchar(128) DEFAULT NULL,
    inventorynextpermissions int DEFAULT NULL,
    inventorycurrentpermissions int DEFAULT NULL,
    invtype int DEFAULT NULL,
    creatorid varchar(255) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    inventorybasepermissions int NOT NULL DEFAULT 0,
    inventoryeveryonepermissions int NOT NULL DEFAULT 0,
    saleprice int NOT NULL DEFAULT 0,
    saletype int NOT NULL DEFAULT 0,
    creationdate int NOT NULL DEFAULT 0,
    groupid uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    groupowned int NOT NULL DEFAULT 0,
    flags int NOT NULL DEFAULT 0,
    inventorygrouppermissions int NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_inventoryitems_avatarid ON inventoryitems(avatarid);
CREATE INDEX IF NOT EXISTS idx_inventoryitems_parentfolderid ON inventoryitems(parentfolderid);
CREATE INDEX IF NOT EXISTS idx_inventoryitems_assetid ON inventoryitems(assetid);

-- Ensure columns exist with defaults for existing tables (safe ALTER IF NOT EXISTS)
DO $$ BEGIN
    ALTER TABLE inventoryitems ALTER COLUMN inventorybasepermissions SET DEFAULT 0;
    ALTER TABLE inventoryitems ALTER COLUMN inventoryeveryonepermissions SET DEFAULT 0;
    ALTER TABLE inventoryitems ALTER COLUMN inventorygrouppermissions SET DEFAULT 0;
    ALTER TABLE inventoryitems ALTER COLUMN saleprice SET DEFAULT 0;
    ALTER TABLE inventoryitems ALTER COLUMN saletype SET DEFAULT 0;
    ALTER TABLE inventoryitems ALTER COLUMN groupowned SET DEFAULT 0;
    ALTER TABLE inventoryitems ALTER COLUMN flags SET DEFAULT 0;
    ALTER TABLE inventoryitems ALTER COLUMN creationdate SET DEFAULT 0;
EXCEPTION WHEN OTHERS THEN NULL;
END $$;

-- Fix any NULL permission values from older inserts
UPDATE inventoryitems SET inventorybasepermissions = 0 WHERE inventorybasepermissions IS NULL;
UPDATE inventoryitems SET inventorycurrentpermissions = 0 WHERE inventorycurrentpermissions IS NULL;
UPDATE inventoryitems SET inventoryeveryonepermissions = 0 WHERE inventoryeveryonepermissions IS NULL;
UPDATE inventoryitems SET inventorynextpermissions = 0 WHERE inventorynextpermissions IS NULL;
UPDATE inventoryitems SET inventorygrouppermissions = 0 WHERE inventorygrouppermissions IS NULL;
UPDATE inventoryitems SET saleprice = 0 WHERE saleprice IS NULL;
UPDATE inventoryitems SET saletype = 0 WHERE saletype IS NULL;
UPDATE inventoryitems SET groupowned = 0 WHERE groupowned IS NULL;
UPDATE inventoryitems SET flags = 0 WHERE flags IS NULL;
