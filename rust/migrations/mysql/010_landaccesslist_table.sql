-- Migration 010: Create landaccesslist table for parcel access control
-- Matches C# OpenSim RegionStore MySQL schema

CREATE TABLE IF NOT EXISTS `landaccesslist` (
    `LandUUID` varchar(255) DEFAULT NULL,
    `AccessUUID` varchar(255) DEFAULT NULL,
    `Flags` int(11) DEFAULT NULL,
    `Expires` int(11) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=latin1;
