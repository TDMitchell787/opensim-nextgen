-- LogStore Migration v1: Logging System
-- This migration creates the logging system for OpenSim operations

-- Logs table
CREATE TABLE IF NOT EXISTS logs (
    logID INTEGER PRIMARY KEY AUTOINCREMENT,
    target VARCHAR(36) DEFAULT NULL,
    server VARCHAR(64) DEFAULT NULL,
    method VARCHAR(64) DEFAULT NULL,
    arguments VARCHAR(255) DEFAULT NULL,
    priority INTEGER DEFAULT NULL,
    message TEXT
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS logs_target ON logs(target);
CREATE INDEX IF NOT EXISTS logs_server ON logs(server);
CREATE INDEX IF NOT EXISTS logs_method ON logs(method);
CREATE INDEX IF NOT EXISTS logs_priority ON logs(priority);