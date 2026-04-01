-- Migration 011: Create landaccesslist table for parcel access control
-- Matches C# OpenSim RegionStore schema (final version after all migrations)

CREATE TABLE IF NOT EXISTS landaccesslist (
    "LandUUID" uuid NOT NULL,
    "AccessUUID" uuid NOT NULL,
    "Flags" integer NOT NULL DEFAULT 0,
    "Expires" integer NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_landaccesslist_landuuid ON landaccesslist("LandUUID");
CREATE INDEX IF NOT EXISTS idx_landaccesslist_accessuuid ON landaccesslist("AccessUUID");
