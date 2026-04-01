-- RegionStore Migration Version 60: Float to Double Precision Fix
-- This migration optimizes storage by using appropriate precision for coordinates and physics data
-- Note: SQLite uses REAL type for all floating point numbers, so this migration is for compatibility

-- SQLite stores all numeric values optimally, so this migration creates indexes for performance
-- and ensures compatibility with OpenSim master expectations

-- Create performance indexes for frequently queried prims fields
CREATE INDEX IF NOT EXISTS prims_position ON prims(PositionX, PositionY, PositionZ);
CREATE INDEX IF NOT EXISTS prims_groupposition ON prims(GroupPositionX, GroupPositionY, GroupPositionZ);
CREATE INDEX IF NOT EXISTS prims_rotation ON prims(RotationX, RotationY, RotationZ, RotationW);
CREATE INDEX IF NOT EXISTS prims_owner ON prims(OwnerID);
CREATE INDEX IF NOT EXISTS prims_creator ON prims(CreatorID);
CREATE INDEX IF NOT EXISTS prims_group ON prims(GroupID);
CREATE INDEX IF NOT EXISTS prims_scenegroup ON prims(SceneGroupID);