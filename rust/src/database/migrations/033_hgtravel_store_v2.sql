-- HGTravelStore Migration v1-2: Hypergrid Travel Data
-- This migration creates the hypergrid travel system for tracking user travel between grids

-- Hypergrid traveling data table
CREATE TABLE IF NOT EXISTS hg_traveling_data (
    SessionID VARCHAR(36) NOT NULL PRIMARY KEY,
    UserID VARCHAR(36) NOT NULL,
    GridExternalName VARCHAR(255) NOT NULL DEFAULT '',
    ServiceToken VARCHAR(255) NOT NULL DEFAULT '',
    ClientIPAddress VARCHAR(16) NOT NULL DEFAULT '',
    MyIPAddress VARCHAR(16) NOT NULL DEFAULT '',
    TMStamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS hg_traveling_data_userid ON hg_traveling_data(UserID);
CREATE INDEX IF NOT EXISTS hg_traveling_data_sessionid ON hg_traveling_data(SessionID);
CREATE INDEX IF NOT EXISTS hg_traveling_data_tmstamp ON hg_traveling_data(TMStamp);