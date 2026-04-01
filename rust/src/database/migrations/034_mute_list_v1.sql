-- MuteListStore Migration v1: User Mute Lists
-- This migration creates the mute list system for user communication preferences

-- Mute list table
CREATE TABLE IF NOT EXISTS MuteList (
    AgentID CHAR(36) NOT NULL,
    MuteID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    MuteName VARCHAR(64) NOT NULL DEFAULT '',
    MuteType INTEGER NOT NULL DEFAULT 1,
    MuteFlags INTEGER NOT NULL DEFAULT 0,
    Stamp INTEGER NOT NULL,
    PRIMARY KEY (AgentID, MuteID, MuteName)
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS MuteList_AgentID ON MuteList(AgentID);
CREATE INDEX IF NOT EXISTS MuteList_MuteID ON MuteList(MuteID);
CREATE INDEX IF NOT EXISTS MuteList_MuteName ON MuteList(MuteName);