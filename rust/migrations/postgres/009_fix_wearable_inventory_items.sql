-- Migration 009: Fix Wearable Inventory Items
-- Phase 95: Avatar wearables fix - item IDs must match AgentWearablesUpdate exactly
-- Date: 2026-02-03
--
-- Problem: Viewer cannot render avatar because inventory item IDs don't match
-- the hardcoded item IDs sent in AgentWearablesUpdate message.
--
-- Solution: Insert correct wearable items with exact UUIDs for each user.

-- Step 1: Delete any existing default wearable items that might have wrong IDs
-- We identify them by assetid (the asset UUIDs are stable)
DELETE FROM inventoryitems WHERE assetid IN (
    '66c41e39-38f9-f75a-024e-585989bfab73',  -- Shape asset
    '77c41e39-38f9-f75a-024e-585989bbabbb',  -- Skin asset
    'd342e6c0-b9d2-11dc-95ff-0800200c9a66',  -- Hair asset
    '4bb6fa4d-1cd2-498a-a84c-95c1a0e745a7',  -- Eyes asset
    '00000000-38f9-1111-024e-222222111110',  -- Shirt asset
    '00000000-38f9-1111-024e-222222111120'   -- Pants asset
);

-- Step 2: Insert correct wearable items for each user
-- Body Parts (assettype 13) go in Body Parts folder (type 13)
-- Clothing (assettype 5) go in Clothing folder (type 5)

-- Insert Shape for all users (wearable type 0)
INSERT INTO inventoryitems (inventoryid, assetid, assettype, parentfolderid, avatarid, creatorid,
    inventoryname, inventorydescription, inventorynextpermissions, inventorycurrentpermissions,
    inventorybasepermissions, inventoryeveryonepermissions, inventorygrouppermissions,
    invtype, saleprice, saletype, creationdate, groupid, groupowned, flags)
SELECT
    '66c41e39-38f9-f75a-024e-585989bfaba9'::uuid,  -- EXACT item ID from AgentWearablesUpdate
    '66c41e39-38f9-f75a-024e-585989bfab73'::uuid,  -- Shape asset ID
    13,  -- assettype: body part
    f.folderid,  -- Body Parts folder
    f.agentid,
    '11111111-1111-0000-0000-000100bba000',  -- System creator (VARCHAR)
    'Default Shape',
    'Default avatar shape',
    647168, 647168, 647168, 0, 0,
    13,  -- invtype: body part
    0, 0,  -- saleprice, saletype
    EXTRACT(EPOCH FROM NOW())::bigint,  -- creationdate
    '00000000-0000-0000-0000-000000000000'::uuid,  -- groupid
    0, 0  -- groupowned, flags
FROM inventoryfolders f
WHERE f.type = 13  -- Body Parts folder
ON CONFLICT (inventoryid) DO NOTHING;

-- Insert Skin for all users (wearable type 1)
INSERT INTO inventoryitems (inventoryid, assetid, assettype, parentfolderid, avatarid, creatorid,
    inventoryname, inventorydescription, inventorynextpermissions, inventorycurrentpermissions,
    inventorybasepermissions, inventoryeveryonepermissions, inventorygrouppermissions,
    invtype, saleprice, saletype, creationdate, groupid, groupowned, flags)
SELECT
    '77c41e39-38f9-f75a-024e-585989bfabc9'::uuid,  -- EXACT item ID from AgentWearablesUpdate
    '77c41e39-38f9-f75a-024e-585989bbabbb'::uuid,  -- Skin asset ID
    13,  -- assettype: body part
    f.folderid,
    f.agentid,
    '11111111-1111-0000-0000-000100bba000',
    'Default Skin',
    'Default avatar skin',
    647168, 647168, 647168, 0, 0,
    13, 0, 0,
    EXTRACT(EPOCH FROM NOW())::bigint,
    '00000000-0000-0000-0000-000000000000'::uuid,
    0, 0
FROM inventoryfolders f
WHERE f.type = 13
ON CONFLICT (inventoryid) DO NOTHING;

-- Insert Hair for all users (wearable type 2)
INSERT INTO inventoryitems (inventoryid, assetid, assettype, parentfolderid, avatarid, creatorid,
    inventoryname, inventorydescription, inventorynextpermissions, inventorycurrentpermissions,
    inventorybasepermissions, inventoryeveryonepermissions, inventorygrouppermissions,
    invtype, saleprice, saletype, creationdate, groupid, groupowned, flags)
SELECT
    'd342e6c1-b9d2-11dc-95ff-0800200c9a66'::uuid,  -- EXACT item ID from AgentWearablesUpdate
    'd342e6c0-b9d2-11dc-95ff-0800200c9a66'::uuid,  -- Hair asset ID
    13,  -- assettype: body part
    f.folderid,
    f.agentid,
    '11111111-1111-0000-0000-000100bba000',
    'Default Hair',
    'Default avatar hair',
    647168, 647168, 647168, 0, 0,
    13, 0, 0,
    EXTRACT(EPOCH FROM NOW())::bigint,
    '00000000-0000-0000-0000-000000000000'::uuid,
    0, 0
FROM inventoryfolders f
WHERE f.type = 13
ON CONFLICT (inventoryid) DO NOTHING;

-- Insert Eyes for all users (wearable type 3)
INSERT INTO inventoryitems (inventoryid, assetid, assettype, parentfolderid, avatarid, creatorid,
    inventoryname, inventorydescription, inventorynextpermissions, inventorycurrentpermissions,
    inventorybasepermissions, inventoryeveryonepermissions, inventorygrouppermissions,
    invtype, saleprice, saletype, creationdate, groupid, groupowned, flags)
SELECT
    'cdc31054-eed8-4021-994f-4e0c6e861b50'::uuid,  -- EXACT item ID from AgentWearablesUpdate
    '4bb6fa4d-1cd2-498a-a84c-95c1a0e745a7'::uuid,  -- Eyes asset ID
    13,  -- assettype: body part
    f.folderid,
    f.agentid,
    '11111111-1111-0000-0000-000100bba000',
    'Default Eyes',
    'Default avatar eyes',
    647168, 647168, 647168, 0, 0,
    13, 0, 0,
    EXTRACT(EPOCH FROM NOW())::bigint,
    '00000000-0000-0000-0000-000000000000'::uuid,
    0, 0
FROM inventoryfolders f
WHERE f.type = 13
ON CONFLICT (inventoryid) DO NOTHING;

-- Insert Shirt for all users (wearable type 4)
INSERT INTO inventoryitems (inventoryid, assetid, assettype, parentfolderid, avatarid, creatorid,
    inventoryname, inventorydescription, inventorynextpermissions, inventorycurrentpermissions,
    inventorybasepermissions, inventoryeveryonepermissions, inventorygrouppermissions,
    invtype, saleprice, saletype, creationdate, groupid, groupowned, flags)
SELECT
    '77c41e39-38f9-f75a-0000-585989bf0000'::uuid,  -- EXACT item ID from AgentWearablesUpdate
    '00000000-38f9-1111-024e-222222111110'::uuid,  -- Shirt asset ID
    5,  -- assettype: clothing
    f.folderid,
    f.agentid,
    '11111111-1111-0000-0000-000100bba000',
    'Default Shirt',
    'Default avatar shirt',
    647168, 647168, 647168, 0, 0,
    5,  -- invtype: clothing
    0, 0,
    EXTRACT(EPOCH FROM NOW())::bigint,
    '00000000-0000-0000-0000-000000000000'::uuid,
    0, 0
FROM inventoryfolders f
WHERE f.type = 5  -- Clothing folder
ON CONFLICT (inventoryid) DO NOTHING;

-- Insert Pants for all users (wearable type 5)
INSERT INTO inventoryitems (inventoryid, assetid, assettype, parentfolderid, avatarid, creatorid,
    inventoryname, inventorydescription, inventorynextpermissions, inventorycurrentpermissions,
    inventorybasepermissions, inventoryeveryonepermissions, inventorygrouppermissions,
    invtype, saleprice, saletype, creationdate, groupid, groupowned, flags)
SELECT
    '77c41e39-38f9-f75a-0000-5859892f1111'::uuid,  -- EXACT item ID from AgentWearablesUpdate
    '00000000-38f9-1111-024e-222222111120'::uuid,  -- Pants asset ID
    5,  -- assettype: clothing
    f.folderid,
    f.agentid,
    '11111111-1111-0000-0000-000100bba000',
    'Default Pants',
    'Default avatar pants',
    647168, 647168, 647168, 0, 0,
    5, 0, 0,
    EXTRACT(EPOCH FROM NOW())::bigint,
    '00000000-0000-0000-0000-000000000000'::uuid,
    0, 0
FROM inventoryfolders f
WHERE f.type = 5
ON CONFLICT (inventoryid) DO NOTHING;

-- Show results
SELECT inventoryid, inventoryname, assetid, avatarid FROM inventoryitems
WHERE inventoryid IN (
    '66c41e39-38f9-f75a-024e-585989bfaba9',
    '77c41e39-38f9-f75a-024e-585989bfabc9',
    'd342e6c1-b9d2-11dc-95ff-0800200c9a66',
    'cdc31054-eed8-4021-994f-4e0c6e861b50',
    '77c41e39-38f9-f75a-0000-585989bf0000',
    '77c41e39-38f9-f75a-0000-5859892f1111'
);
