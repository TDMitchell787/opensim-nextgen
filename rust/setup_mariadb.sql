-- MariaDB Setup for OpenSim Next
-- Run with: sudo mysql -u root < setup_mariadb.sql

-- Create the opensim_mariadb database
CREATE DATABASE IF NOT EXISTS opensim_mariadb;

-- Create the opensim user with all privileges
CREATE USER IF NOT EXISTS 'opensim'@'localhost' IDENTIFIED BY 'opensim_secure';
GRANT ALL PRIVILEGES ON opensim_mariadb.* TO 'opensim'@'localhost';

-- Also allow connections from 127.0.0.1
CREATE USER IF NOT EXISTS 'opensim'@'127.0.0.1' IDENTIFIED BY 'opensim_secure';
GRANT ALL PRIVILEGES ON opensim_mariadb.* TO 'opensim'@'127.0.0.1';

-- Refresh privileges
FLUSH PRIVILEGES;

-- Verify the setup
USE opensim_mariadb;
SHOW TABLES;

-- Display success message
SELECT 'MariaDB setup completed successfully for OpenSim Next!' AS Status;