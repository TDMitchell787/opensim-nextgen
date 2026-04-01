-- FriendsStore Migration v2-4: Update Friends table for OpenSim master compatibility
-- This migration ensures Friends table matches OpenSim master v4 specifications

-- Update Friends table structure if needed (SQLite compatible)
-- Note: Current table already matches v4 structure, this migration ensures compatibility

-- Create indexes for performance (matching OpenSim master)
CREATE INDEX IF NOT EXISTS Friends_PrincipalID ON Friends(PrincipalID);
CREATE INDEX IF NOT EXISTS Friends_Friend ON Friends(Friend);

-- Verify table structure matches OpenSim master v4 requirements:
-- PrincipalID: VARCHAR(255) or CHAR(36) - both supported
-- Friend: VARCHAR(255) - matches
-- Flags: VARCHAR(16) or INT - both supported  
-- Offered: VARCHAR(32) - matches
-- PRIMARY KEY(PrincipalID, Friend) - matches