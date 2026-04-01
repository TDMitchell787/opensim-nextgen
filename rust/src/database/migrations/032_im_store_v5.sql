-- IM_Store Migration v1-5: Instant Messaging and Offline Messages
-- This migration creates the instant messaging system for offline message storage

-- Offline instant messages table
CREATE TABLE IF NOT EXISTS im_offline (
    ID INTEGER PRIMARY KEY AUTOINCREMENT,
    PrincipalID CHAR(36) NOT NULL DEFAULT '',
    FromID CHAR(36) NOT NULL DEFAULT '',
    Message TEXT NOT NULL,
    TMStamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS im_offline_principalid ON im_offline(PrincipalID);
CREATE INDEX IF NOT EXISTS im_offline_fromid ON im_offline(FromID);
CREATE INDEX IF NOT EXISTS im_offline_tmstamp ON im_offline(TMStamp);