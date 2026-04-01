-- Phase 95.5: Populate COF (Current Outfit Folder) with default wearable links
-- for ALL users in opensim_pg
--
-- Run with: psql -U opensim -d opensim_pg -f rust/scripts/populate_cof_links.sql
--
-- This script:
-- 1. Creates COF folders (type=46) for users that don't have one
-- 2. Inserts 6 default wearable link items into each user's COF
-- 3. Fixes existing incorrect entries (wrong invtype or flags)
-- 4. Skips users that already have correct COF links
--
-- Matches OpenSim-master CreateDefaultAppearanceEntries() / CreateCurrentOutfitLink()
-- AssetType=24 (Link), InvType=18 (Wearable), Flags=WearableType, Perms=32768 (Copy)

BEGIN;

-- Step 1: Create COF folders for users that don't have one
INSERT INTO inventoryfolders (folderid, agentid, parentfolderid, foldername, type, version)
SELECT
    gen_random_uuid(),
    u.principalid,
    root.folderid,
    'Current Outfit',
    46,
    1
FROM useraccounts u
JOIN inventoryfolders root ON root.agentid = u.principalid AND root.type = 8
WHERE NOT EXISTS (
    SELECT 1 FROM inventoryfolders f WHERE f.agentid = u.principalid AND f.type = 46
);

-- Step 2: Fix existing COF links with wrong invtype (24 should be 18)
UPDATE inventoryitems SET invtype = 18
WHERE parentfolderid IN (SELECT folderid FROM inventoryfolders WHERE type = 46)
  AND assettype = 24
  AND invtype = 24;

-- Step 3: Fix existing COF links with wrong flags
UPDATE inventoryitems SET flags = 4
WHERE parentfolderid IN (SELECT folderid FROM inventoryfolders WHERE type = 46)
  AND assettype = 24
  AND inventoryname = 'Default Shirt'
  AND flags != 4;

UPDATE inventoryitems SET flags = 5
WHERE parentfolderid IN (SELECT folderid FROM inventoryfolders WHERE type = 46)
  AND assettype = 24
  AND inventoryname = 'Default Pants'
  AND flags != 5;

-- Step 4: Insert missing COF link items for all users
-- Uses a CTE to avoid inserting duplicates (checks by name + folder)

-- Shape (wearableType=0)
INSERT INTO inventoryitems (
    inventoryid, avatarid, parentfolderid, assetid, assettype, invtype,
    inventoryname, inventorydescription, creationdate, creatorid,
    inventorybasepermissions, inventorycurrentpermissions,
    inventoryeveryonepermissions, inventorygrouppermissions, inventorynextpermissions,
    groupid, groupowned, saleprice, saletype, flags
)
SELECT
    gen_random_uuid(),
    f.agentid,
    f.folderid,
    '66c41e39-38f9-f75a-024e-585989bfaba9'::uuid,  -- Shape ITEM UUID
    24, 18,
    'Default Shape', '@Default Shape',
    EXTRACT(EPOCH FROM now())::int,
    '11111111-1111-0000-0000-000100bba000',
    32768, 32768, 32768, 32768, 32768,
    NULL, 0, 0, 0, 0
FROM inventoryfolders f
WHERE f.type = 46
  AND NOT EXISTS (
    SELECT 1 FROM inventoryitems i
    WHERE i.parentfolderid = f.folderid AND i.inventoryname = 'Default Shape' AND i.assettype = 24
  );

-- Skin (wearableType=1)
INSERT INTO inventoryitems (
    inventoryid, avatarid, parentfolderid, assetid, assettype, invtype,
    inventoryname, inventorydescription, creationdate, creatorid,
    inventorybasepermissions, inventorycurrentpermissions,
    inventoryeveryonepermissions, inventorygrouppermissions, inventorynextpermissions,
    groupid, groupowned, saleprice, saletype, flags
)
SELECT
    gen_random_uuid(),
    f.agentid,
    f.folderid,
    '77c41e39-38f9-f75a-024e-585989bfabc9'::uuid,  -- Skin ITEM UUID
    24, 18,
    'Default Skin', '@Default Skin',
    EXTRACT(EPOCH FROM now())::int,
    '11111111-1111-0000-0000-000100bba000',
    32768, 32768, 32768, 32768, 32768,
    NULL, 0, 0, 0, 1
FROM inventoryfolders f
WHERE f.type = 46
  AND NOT EXISTS (
    SELECT 1 FROM inventoryitems i
    WHERE i.parentfolderid = f.folderid AND i.inventoryname = 'Default Skin' AND i.assettype = 24
  );

-- Hair (wearableType=2)
INSERT INTO inventoryitems (
    inventoryid, avatarid, parentfolderid, assetid, assettype, invtype,
    inventoryname, inventorydescription, creationdate, creatorid,
    inventorybasepermissions, inventorycurrentpermissions,
    inventoryeveryonepermissions, inventorygrouppermissions, inventorynextpermissions,
    groupid, groupowned, saleprice, saletype, flags
)
SELECT
    gen_random_uuid(),
    f.agentid,
    f.folderid,
    'd342e6c1-b9d2-11dc-95ff-0800200c9a66'::uuid,  -- Hair ITEM UUID
    24, 18,
    'Default Hair', '@Default Hair',
    EXTRACT(EPOCH FROM now())::int,
    '11111111-1111-0000-0000-000100bba000',
    32768, 32768, 32768, 32768, 32768,
    NULL, 0, 0, 0, 2
FROM inventoryfolders f
WHERE f.type = 46
  AND NOT EXISTS (
    SELECT 1 FROM inventoryitems i
    WHERE i.parentfolderid = f.folderid AND i.inventoryname = 'Default Hair' AND i.assettype = 24
  );

-- Eyes (wearableType=3)
INSERT INTO inventoryitems (
    inventoryid, avatarid, parentfolderid, assetid, assettype, invtype,
    inventoryname, inventorydescription, creationdate, creatorid,
    inventorybasepermissions, inventorycurrentpermissions,
    inventoryeveryonepermissions, inventorygrouppermissions, inventorynextpermissions,
    groupid, groupowned, saleprice, saletype, flags
)
SELECT
    gen_random_uuid(),
    f.agentid,
    f.folderid,
    'cdc31054-eed8-4021-994f-4e0c6e861b50'::uuid,  -- Eyes ITEM UUID
    24, 18,
    'Default Eyes', '@Default Eyes',
    EXTRACT(EPOCH FROM now())::int,
    '11111111-1111-0000-0000-000100bba000',
    32768, 32768, 32768, 32768, 32768,
    NULL, 0, 0, 0, 3
FROM inventoryfolders f
WHERE f.type = 46
  AND NOT EXISTS (
    SELECT 1 FROM inventoryitems i
    WHERE i.parentfolderid = f.folderid AND i.inventoryname = 'Default Eyes' AND i.assettype = 24
  );

-- Shirt (wearableType=4)
INSERT INTO inventoryitems (
    inventoryid, avatarid, parentfolderid, assetid, assettype, invtype,
    inventoryname, inventorydescription, creationdate, creatorid,
    inventorybasepermissions, inventorycurrentpermissions,
    inventoryeveryonepermissions, inventorygrouppermissions, inventorynextpermissions,
    groupid, groupowned, saleprice, saletype, flags
)
SELECT
    gen_random_uuid(),
    f.agentid,
    f.folderid,
    '77c41e39-38f9-f75a-0000-585989bf0000'::uuid,  -- Shirt ITEM UUID
    24, 18,
    'Default Shirt', '@Default Shirt',
    EXTRACT(EPOCH FROM now())::int,
    '11111111-1111-0000-0000-000100bba000',
    32768, 32768, 32768, 32768, 32768,
    NULL, 0, 0, 0, 4
FROM inventoryfolders f
WHERE f.type = 46
  AND NOT EXISTS (
    SELECT 1 FROM inventoryitems i
    WHERE i.parentfolderid = f.folderid AND i.inventoryname = 'Default Shirt' AND i.assettype = 24
  );

-- Pants (wearableType=5)
INSERT INTO inventoryitems (
    inventoryid, avatarid, parentfolderid, assetid, assettype, invtype,
    inventoryname, inventorydescription, creationdate, creatorid,
    inventorybasepermissions, inventorycurrentpermissions,
    inventoryeveryonepermissions, inventorygrouppermissions, inventorynextpermissions,
    groupid, groupowned, saleprice, saletype, flags
)
SELECT
    gen_random_uuid(),
    f.agentid,
    f.folderid,
    '77c41e39-38f9-f75a-0000-5859892f1111'::uuid,  -- Pants ITEM UUID
    24, 18,
    'Default Pants', '@Default Pants',
    EXTRACT(EPOCH FROM now())::int,
    '11111111-1111-0000-0000-000100bba000',
    32768, 32768, 32768, 32768, 32768,
    NULL, 0, 0, 0, 5
FROM inventoryfolders f
WHERE f.type = 46
  AND NOT EXISTS (
    SELECT 1 FROM inventoryitems i
    WHERE i.parentfolderid = f.folderid AND i.inventoryname = 'Default Pants' AND i.assettype = 24
  );

-- Step 5: Verification
SELECT
    a.firstname || ' ' || a.lastname AS user_name,
    COUNT(i.inventoryid) AS cof_link_count
FROM inventoryfolders f
JOIN useraccounts a ON a.principalid = f.agentid
LEFT JOIN inventoryitems i ON i.parentfolderid = f.folderid AND i.assettype = 24
WHERE f.type = 46
GROUP BY a.firstname, a.lastname
ORDER BY a.firstname;

COMMIT;
