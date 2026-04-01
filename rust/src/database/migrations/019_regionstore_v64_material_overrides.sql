-- RegionStore Migration Version 64: Material Overrides
-- This migration adds support for material override data

-- Create primshapes table if it doesn't exist (SQLite compatible)
CREATE TABLE IF NOT EXISTS primshapes (
    UUID CHAR(36) NOT NULL PRIMARY KEY,
    Shape INTEGER DEFAULT NULL,
    ScaleX REAL DEFAULT NULL,
    ScaleY REAL DEFAULT NULL,
    ScaleZ REAL DEFAULT NULL,
    PCode INTEGER DEFAULT NULL,
    PathBegin INTEGER DEFAULT NULL,
    PathEnd INTEGER DEFAULT NULL,
    PathScaleX INTEGER DEFAULT NULL,
    PathScaleY INTEGER DEFAULT NULL,
    PathShearX INTEGER DEFAULT NULL,
    PathShearY INTEGER DEFAULT NULL,
    PathSkew INTEGER DEFAULT NULL,
    PathCurve INTEGER DEFAULT NULL,
    PathRadiusOffset INTEGER DEFAULT NULL,
    PathRevolutions INTEGER DEFAULT NULL,
    PathTaperX INTEGER DEFAULT NULL,
    PathTaperY INTEGER DEFAULT NULL,
    PathTwist INTEGER DEFAULT NULL,
    PathTwistBegin INTEGER DEFAULT NULL,
    ProfileBegin INTEGER DEFAULT NULL,
    ProfileEnd INTEGER DEFAULT NULL,
    ProfileCurve INTEGER DEFAULT NULL,
    ProfileHollow INTEGER DEFAULT NULL,
    Texture BLOB DEFAULT NULL,
    ExtraParams BLOB DEFAULT NULL,
    State INTEGER DEFAULT NULL,
    Media TEXT DEFAULT NULL,
    MatOvrd BLOB DEFAULT NULL
);

-- Create index for performance
CREATE INDEX IF NOT EXISTS primshapes_uuid ON primshapes(UUID);