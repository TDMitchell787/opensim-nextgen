-- Presence Migration v3-4: Update Presence table for OpenSim master compatibility
-- This migration adds LastSeen column and required indexes

-- Add LastSeen column to Presence table (SQLite compatible)
ALTER TABLE Presence ADD COLUMN LastSeen DATETIME DEFAULT CURRENT_TIMESTAMP;

-- Create indexes for performance (matching OpenSim master v4)
CREATE INDEX IF NOT EXISTS Presence_UserID ON Presence(UserID);
CREATE INDEX IF NOT EXISTS Presence_RegionID ON Presence(RegionID);
CREATE INDEX IF NOT EXISTS Presence_SessionID ON Presence(SessionID);

-- Update existing records to have a valid LastSeen timestamp
UPDATE Presence SET LastSeen = datetime('now') WHERE LastSeen IS NULL;