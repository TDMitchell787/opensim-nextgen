-- GridUser Migration v2: Update GridUser table for OpenSim master compatibility
-- This migration ensures GridUser table matches OpenSim master v2 specifications

-- Create indexes for performance (matching OpenSim master)
CREATE INDEX IF NOT EXISTS GridUser_UserID ON GridUser(UserID);
CREATE INDEX IF NOT EXISTS GridUser_LastRegionID ON GridUser(LastRegionID);
CREATE INDEX IF NOT EXISTS GridUser_HomeRegionID ON GridUser(HomeRegionID);

-- The table structure already matches OpenSim master v2 requirements:
-- UserID: VARCHAR(255) - matches
-- HomeRegionID, LastRegionID: CHAR(36) - matches (SQLite compatible with UUID)
-- Position/LookAt fields: VARCHAR(64) - matches
-- Online: CHAR(5) - matches
-- Login/Logout: CHAR(16) - matches