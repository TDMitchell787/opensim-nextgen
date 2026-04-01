-- PostgreSQL INT4/INT8 Compatibility Fix Template
-- 
-- This script fixes the common INT4 vs INT8 mismatch issue when using 
-- PostgreSQL with Rust applications that expect i64 (INT8/BIGINT) types.
--
-- Issue: Rust expects i64 (BIGINT/INT8) but PostgreSQL migration creates 
--        integer (INT4) columns, causing "mismatched types" errors.
--
-- Solution: Convert all integer columns to bigint for Rust compatibility.
--
-- USAGE:
--   psql -U opensim -d opensim_db -f POSTGRESQL_INT4_FIX_TEMPLATE.sql

-- Remove views that depend on columns we need to alter
DROP VIEW IF EXISTS grid_regions CASCADE;

-- Convert all critical integer columns to bigint for Rust i64 compatibility

-- User and Authentication Tables
ALTER TABLE useraccounts ALTER COLUMN created TYPE bigint;
ALTER TABLE useraccounts ALTER COLUMN userlevel TYPE bigint;
ALTER TABLE useraccounts ALTER COLUMN active TYPE bigint;
ALTER TABLE useraccounts ALTER COLUMN userflags TYPE bigint;

-- Session and Circuit Code Tables  
ALTER TABLE sessions ALTER COLUMN circuit_code TYPE bigint;

-- Asset Tables
ALTER TABLE assets ALTER COLUMN create_time TYPE bigint;
ALTER TABLE assets ALTER COLUMN access_time TYPE bigint;
ALTER TABLE assets ALTER COLUMN assettype TYPE bigint;
ALTER TABLE assets ALTER COLUMN local TYPE bigint;
ALTER TABLE assets ALTER COLUMN temporary TYPE bigint;
ALTER TABLE assets ALTER COLUMN asset_flags TYPE bigint;

-- Inventory Tables
ALTER TABLE inventoryfolders ALTER COLUMN type TYPE bigint;
ALTER TABLE inventoryfolders ALTER COLUMN version TYPE bigint;
ALTER TABLE inventoryitems ALTER COLUMN assettype TYPE bigint;
ALTER TABLE inventoryitems ALTER COLUMN creationdate TYPE bigint;
ALTER TABLE inventoryitems ALTER COLUMN flags TYPE bigint;
ALTER TABLE inventoryitems ALTER COLUMN invtype TYPE bigint;
ALTER TABLE inventoryitems ALTER COLUMN groupowned TYPE bigint;
ALTER TABLE inventoryitems ALTER COLUMN inventorybasepermissions TYPE bigint;
ALTER TABLE inventoryitems ALTER COLUMN inventorycurrentpermissions TYPE bigint;
ALTER TABLE inventoryitems ALTER COLUMN inventoryeveryonepermissions TYPE bigint;
ALTER TABLE inventoryitems ALTER COLUMN inventorygrouppermissions TYPE bigint;
ALTER TABLE inventoryitems ALTER COLUMN inventorynextpermissions TYPE bigint;
ALTER TABLE inventoryitems ALTER COLUMN saleprice TYPE bigint;
ALTER TABLE inventoryitems ALTER COLUMN saletype TYPE bigint;

-- Region Tables
ALTER TABLE regions ALTER COLUMN locx TYPE bigint;
ALTER TABLE regions ALTER COLUMN locy TYPE bigint;
ALTER TABLE regions ALTER COLUMN size_x TYPE bigint;
ALTER TABLE regions ALTER COLUMN size_y TYPE bigint;
ALTER TABLE regions ALTER COLUMN access TYPE bigint;
ALTER TABLE regions ALTER COLUMN flags TYPE bigint;
ALTER TABLE regions ALTER COLUMN serverport TYPE bigint;
ALTER TABLE regions ALTER COLUMN external_port TYPE bigint;
ALTER TABLE regions ALTER COLUMN internal_port TYPE bigint;
ALTER TABLE regions ALTER COLUMN last_seen TYPE bigint;
ALTER TABLE regions ALTER COLUMN serverhttpport TYPE bigint;
ALTER TABLE regions ALTER COLUMN serverremotingport TYPE bigint;
ALTER TABLE regions ALTER COLUMN sizex TYPE bigint;
ALTER TABLE regions ALTER COLUMN sizey TYPE bigint;
ALTER TABLE regions ALTER COLUMN locz TYPE bigint;

-- Friends and Social Tables
ALTER TABLE friends ALTER COLUMN flags TYPE bigint;

-- Agent Preferences
ALTER TABLE agentprefs ALTER COLUMN permeveryone TYPE bigint;
ALTER TABLE agentprefs ALTER COLUMN permgroup TYPE bigint;
ALTER TABLE agentprefs ALTER COLUMN permnextowner TYPE bigint;
ALTER TABLE agentprefs ALTER COLUMN languageispublic TYPE bigint;

-- Estate Management Tables
ALTER TABLE estate_settings ALTER COLUMN estateid TYPE bigint;
ALTER TABLE estate_settings ALTER COLUMN parentestateid TYPE bigint;
ALTER TABLE estate_settings ALTER COLUMN publicaccess TYPE bigint;
ALTER TABLE estate_settings ALTER COLUMN allowvoice TYPE bigint;
ALTER TABLE estate_settings ALTER COLUMN allowdirectteleport TYPE bigint;
ALTER TABLE estate_settings ALTER COLUMN denyanonymous TYPE bigint;
ALTER TABLE estate_settings ALTER COLUMN denyidentified TYPE bigint;
ALTER TABLE estate_settings ALTER COLUMN denytransacted TYPE bigint;
ALTER TABLE estate_settings ALTER COLUMN denyminors TYPE bigint;
ALTER TABLE estate_settings ALTER COLUMN allowlandmark TYPE bigint;
ALTER TABLE estate_settings ALTER COLUMN allowparcelchanges TYPE bigint;
ALTER TABLE estate_settings ALTER COLUMN allowsethome TYPE bigint;
ALTER TABLE estate_settings ALTER COLUMN resethomeonteleport TYPE bigint;
ALTER TABLE estate_settings ALTER COLUMN fixedsun TYPE bigint;
ALTER TABLE estate_settings ALTER COLUMN useglobaltime TYPE bigint;
ALTER TABLE estate_settings ALTER COLUMN pricepermeter TYPE bigint;
ALTER TABLE estate_settings ALTER COLUMN taxfree TYPE bigint;
ALTER TABLE estate_settings ALTER COLUMN allowenviromentoverride TYPE bigint;
ALTER TABLE estate_settings ALTER COLUMN blockdwell TYPE bigint;
ALTER TABLE estate_settings ALTER COLUMN abuseemailtoestateowner TYPE bigint;
ALTER TABLE estate_settings ALTER COLUMN estateskipscripts TYPE bigint;
ALTER TABLE estate_settings ALTER COLUMN redirectgridx TYPE bigint;
ALTER TABLE estate_settings ALTER COLUMN redirectgridy TYPE bigint;

-- Other common integer columns that should be bigint
ALTER TABLE estateban ALTER COLUMN estateid TYPE bigint;
ALTER TABLE estateban ALTER COLUMN bantime TYPE bigint;
ALTER TABLE estate_groups ALTER COLUMN estateid TYPE bigint;
ALTER TABLE estate_managers ALTER COLUMN estateid TYPE bigint;
ALTER TABLE estate_users ALTER COLUMN estateid TYPE bigint;
ALTER TABLE estate_map ALTER COLUMN estateid TYPE bigint;

-- Land and Parcel Tables
ALTER TABLE land ALTER COLUMN locallandid TYPE bigint;
ALTER TABLE land ALTER COLUMN landflags TYPE bigint;
ALTER TABLE land ALTER COLUMN landstatus TYPE bigint;
ALTER TABLE land ALTER COLUMN area TYPE bigint;
ALTER TABLE land ALTER COLUMN auctionid TYPE bigint;
ALTER TABLE land ALTER COLUMN claimdate TYPE bigint;
ALTER TABLE land ALTER COLUMN claimprice TYPE bigint;
ALTER TABLE land ALTER COLUMN isgroupowned TYPE bigint;
ALTER TABLE land ALTER COLUMN landingtype TYPE bigint;
ALTER TABLE land ALTER COLUMN mediaautoscale TYPE bigint;
ALTER TABLE land ALTER COLUMN medialoop TYPE bigint;
ALTER TABLE land ALTER COLUMN obscuremedia TYPE bigint;
ALTER TABLE land ALTER COLUMN obscuremusic TYPE bigint;
ALTER TABLE land ALTER COLUMN othercleantime TYPE bigint;
ALTER TABLE land ALTER COLUMN passprice TYPE bigint;
ALTER TABLE land ALTER COLUMN saleprice TYPE bigint;
ALTER TABLE land ALTER COLUMN seeavs TYPE bigint;
ALTER TABLE land ALTER COLUMN anyavsounds TYPE bigint;
ALTER TABLE land ALTER COLUMN groupavsounds TYPE bigint;
ALTER TABLE land ALTER COLUMN category TYPE bigint;
ALTER TABLE land ALTER COLUMN dwell TYPE bigint;

-- Log Tables
ALTER TABLE logs ALTER COLUMN logid TYPE bigint;
ALTER TABLE logs ALTER COLUMN priority TYPE bigint;

-- Mute List
ALTER TABLE mutelist ALTER COLUMN stamp TYPE bigint;
ALTER TABLE mutelist ALTER COLUMN muteflags TYPE bigint;
ALTER TABLE mutelist ALTER COLUMN mutetype TYPE bigint;

-- Groups Tables
ALTER TABLE os_groups_groups ALTER COLUMN allowpublish TYPE bigint;
ALTER TABLE os_groups_groups ALTER COLUMN maturepublish TYPE bigint;
ALTER TABLE os_groups_groups ALTER COLUMN membershipfee TYPE bigint;
ALTER TABLE os_groups_groups ALTER COLUMN showinlist TYPE bigint;
ALTER TABLE os_groups_membership ALTER COLUMN acceptnotices TYPE bigint;
ALTER TABLE os_groups_membership ALTER COLUMN contribution TYPE bigint;
ALTER TABLE os_groups_membership ALTER COLUMN listinprofile TYPE bigint;
ALTER TABLE os_groups_notices ALTER COLUMN tmstamp TYPE bigint;
ALTER TABLE os_groups_notices ALTER COLUMN hasattachment TYPE bigint;
ALTER TABLE os_groups_notices ALTER COLUMN attachmenttype TYPE bigint;

-- 3D Object Tables (Prims)
ALTER TABLE prims ALTER COLUMN linknumber TYPE bigint;
ALTER TABLE prims ALTER COLUMN objectflags TYPE bigint;
ALTER TABLE prims ALTER COLUMN creationdate TYPE bigint;
ALTER TABLE prims ALTER COLUMN basemask TYPE bigint;
ALTER TABLE prims ALTER COLUMN ownermask TYPE bigint;
ALTER TABLE prims ALTER COLUMN groupmask TYPE bigint;
ALTER TABLE prims ALTER COLUMN everyonemask TYPE bigint;
ALTER TABLE prims ALTER COLUMN nextownermask TYPE bigint;
ALTER TABLE prims ALTER COLUMN material TYPE bigint;
ALTER TABLE prims ALTER COLUMN passcollisions TYPE bigint;
ALTER TABLE prims ALTER COLUMN rotationaxislocks TYPE bigint;
ALTER TABLE prims ALTER COLUMN pseudocrc TYPE bigint;

-- Prim Shapes
ALTER TABLE primshapes ALTER COLUMN shape TYPE bigint;
ALTER TABLE primshapes ALTER COLUMN pcode TYPE bigint;
ALTER TABLE primshapes ALTER COLUMN pathbegin TYPE bigint;
ALTER TABLE primshapes ALTER COLUMN pathend TYPE bigint;
ALTER TABLE primshapes ALTER COLUMN pathscalex TYPE bigint;
ALTER TABLE primshapes ALTER COLUMN pathscaley TYPE bigint;
ALTER TABLE primshapes ALTER COLUMN pathcurve TYPE bigint;
ALTER TABLE primshapes ALTER COLUMN pathradiusoffset TYPE bigint;
ALTER TABLE primshapes ALTER COLUMN pathrevolutions TYPE bigint;
ALTER TABLE primshapes ALTER COLUMN pathtaperx TYPE bigint;
ALTER TABLE primshapes ALTER COLUMN pathtapery TYPE bigint;
ALTER TABLE primshapes ALTER COLUMN pathtwist TYPE bigint;
ALTER TABLE primshapes ALTER COLUMN pathtwistbegin TYPE bigint;
ALTER TABLE primshapes ALTER COLUMN pathskew TYPE bigint;
ALTER TABLE primshapes ALTER COLUMN profilebegin TYPE bigint;
ALTER TABLE primshapes ALTER COLUMN profileend TYPE bigint;
ALTER TABLE primshapes ALTER COLUMN profilecurve TYPE bigint;
ALTER TABLE primshapes ALTER COLUMN profilehollow TYPE bigint;
ALTER TABLE primshapes ALTER COLUMN state TYPE bigint;
ALTER TABLE primshapes ALTER COLUMN pathshearx TYPE bigint;
ALTER TABLE primshapes ALTER COLUMN pathsheary TYPE bigint;

-- Region Settings
ALTER TABLE regionsettings ALTER COLUMN agent_limit TYPE bigint;
ALTER TABLE regionsettings ALTER COLUMN allow_damage TYPE bigint;
ALTER TABLE regionsettings ALTER COLUMN allow_land_join_divide TYPE bigint;
ALTER TABLE regionsettings ALTER COLUMN allow_land_resell TYPE bigint;
ALTER TABLE regionsettings ALTER COLUMN block_fly TYPE bigint;
ALTER TABLE regionsettings ALTER COLUMN block_search TYPE bigint;
ALTER TABLE regionsettings ALTER COLUMN block_show_in_search TYPE bigint;
ALTER TABLE regionsettings ALTER COLUMN block_terraform TYPE bigint;
ALTER TABLE regionsettings ALTER COLUMN casino TYPE bigint;
ALTER TABLE regionsettings ALTER COLUMN covenant_datetime TYPE bigint;
ALTER TABLE regionsettings ALTER COLUMN disable_collisions TYPE bigint;
ALTER TABLE regionsettings ALTER COLUMN disable_physics TYPE bigint;
ALTER TABLE regionsettings ALTER COLUMN disable_scripts TYPE bigint;
ALTER TABLE regionsettings ALTER COLUMN fixed_sun TYPE bigint;
ALTER TABLE regionsettings ALTER COLUMN loaded_creation_datetime TYPE bigint;
ALTER TABLE regionsettings ALTER COLUMN maturity TYPE bigint;
ALTER TABLE regionsettings ALTER COLUMN minimum_age TYPE bigint;
ALTER TABLE regionsettings ALTER COLUMN restrict_pushing TYPE bigint;
ALTER TABLE regionsettings ALTER COLUMN sandbox TYPE bigint;
ALTER TABLE regionsettings ALTER COLUMN use_estate_sun TYPE bigint;

-- Windlight Settings
ALTER TABLE regionwindlight ALTER COLUMN cloud_scroll_x_lock TYPE bigint;
ALTER TABLE regionwindlight ALTER COLUMN cloud_scroll_y_lock TYPE bigint;
ALTER TABLE regionwindlight ALTER COLUMN draw_classic_clouds TYPE bigint;
ALTER TABLE regionwindlight ALTER COLUMN max_altitude TYPE bigint;

-- User Profile Tables
ALTER TABLE userclassifieds ALTER COLUMN classifiedflags TYPE bigint;
ALTER TABLE userclassifieds ALTER COLUMN creationdate TYPE bigint;
ALTER TABLE userclassifieds ALTER COLUMN expirationdate TYPE bigint;
ALTER TABLE userclassifieds ALTER COLUMN parentestate TYPE bigint;
ALTER TABLE userclassifieds ALTER COLUMN priceforlisting TYPE bigint;
ALTER TABLE userpicks ALTER COLUMN enabled TYPE bigint;
ALTER TABLE userpicks ALTER COLUMN sortorder TYPE bigint;
ALTER TABLE userpicks ALTER COLUMN toppick TYPE bigint;
ALTER TABLE userprofile ALTER COLUMN profileallowpublish TYPE bigint;
ALTER TABLE userprofile ALTER COLUMN profilematurepublish TYPE bigint;
ALTER TABLE userprofile ALTER COLUMN profileskillsmask TYPE bigint;
ALTER TABLE userprofile ALTER COLUMN profilewanttomask TYPE bigint;
ALTER TABLE usersettings ALTER COLUMN imviaemail TYPE bigint;
ALTER TABLE usersettings ALTER COLUMN visible TYPE bigint;

-- Extended Assets (XAssets)
ALTER TABLE xassetsmeta ALTER COLUMN accesstime TYPE bigint;
ALTER TABLE xassetsmeta ALTER COLUMN assetflags TYPE bigint;
ALTER TABLE xassetsmeta ALTER COLUMN assettype TYPE bigint;
ALTER TABLE xassetsmeta ALTER COLUMN createtime TYPE bigint;
ALTER TABLE xassetsmeta ALTER COLUMN local TYPE bigint;
ALTER TABLE xassetsmeta ALTER COLUMN temporary TYPE bigint;

-- Other miscellaneous tables
ALTER TABLE bakedterrain ALTER COLUMN revision TYPE bigint;
ALTER TABLE im_offline ALTER COLUMN id TYPE bigint;

-- Recreate the grid_regions view if needed
-- CREATE VIEW grid_regions AS SELECT * FROM regions;

-- Verification query - check for remaining integer columns
-- Uncomment the following to see any remaining integer columns:
-- SELECT table_name, column_name, data_type 
-- FROM information_schema.columns 
-- WHERE data_type = 'integer' 
-- AND table_schema = 'public' 
-- ORDER BY table_name, column_name;

-- Success message
DO $$
BEGIN
    RAISE NOTICE 'PostgreSQL INT4/INT8 compatibility fix completed successfully.';
    RAISE NOTICE 'All integer columns have been converted to bigint for Rust i64 compatibility.';
END $$;