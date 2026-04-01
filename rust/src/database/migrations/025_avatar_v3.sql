-- Avatar Migration v3: Update Avatars table for OpenSim master compatibility
-- This migration changes Value column from VARCHAR(255) to TEXT

-- SQLite doesn't support ALTER COLUMN, so we need to recreate the table
CREATE TABLE Avatars_new (
    PrincipalID CHAR(36) NOT NULL,
    Name VARCHAR(32) NOT NULL,
    Value TEXT NOT NULL DEFAULT '',
    PRIMARY KEY(PrincipalID, Name)
);

-- Copy data from old table
INSERT INTO Avatars_new (PrincipalID, Name, Value)
SELECT PrincipalID, Name, Value FROM Avatars;

-- Replace old table with new one
DROP TABLE Avatars;
ALTER TABLE Avatars_new RENAME TO Avatars;

-- Create index for performance
CREATE INDEX IF NOT EXISTS Avatars_PrincipalID ON Avatars(PrincipalID);