-- AgentPrefs Migration v1: User Agent Preferences
-- This migration creates the agent preferences system for user settings

-- Agent preferences table
CREATE TABLE IF NOT EXISTS AgentPrefs (
    PrincipalID CHAR(36) NOT NULL PRIMARY KEY,
    AccessPrefs CHAR(2) NOT NULL DEFAULT 'M',
    HoverHeight REAL NOT NULL DEFAULT 0.0,
    Language CHAR(5) NOT NULL DEFAULT 'en-us',
    LanguageIsPublic INTEGER NOT NULL DEFAULT 1,
    PermEveryone INTEGER NOT NULL DEFAULT 0,
    PermGroup INTEGER NOT NULL DEFAULT 0,
    PermNextOwner INTEGER NOT NULL DEFAULT 532480
);

-- Create index for performance
CREATE INDEX IF NOT EXISTS AgentPrefs_PrincipalID ON AgentPrefs(PrincipalID);