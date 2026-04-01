-- Fix NULL defaults for inventory permission columns
-- Ensures HG items and all future inserts get proper default values
-- Compatible with MySQL 5.7+ and MariaDB 10.2+

ALTER TABLE `inventoryitems`
    MODIFY `inventoryBasePermissions` INT NOT NULL DEFAULT 0,
    MODIFY `inventoryCurrentPermissions` INT DEFAULT 0,
    MODIFY `inventoryEveryOnePermissions` INT NOT NULL DEFAULT 0,
    MODIFY `inventoryNextPermissions` INT DEFAULT 0,
    MODIFY `inventoryGroupPermissions` INT DEFAULT 0,
    MODIFY `salePrice` INT NOT NULL DEFAULT 0,
    MODIFY `saleType` TINYINT NOT NULL DEFAULT 0,
    MODIFY `groupOwned` TINYINT NOT NULL DEFAULT 0,
    MODIFY `flags` INT NOT NULL DEFAULT 0,
    MODIFY `creationDate` INT NOT NULL DEFAULT 0;

-- Fix any NULL permission values from older inserts
UPDATE `inventoryitems` SET `inventoryBasePermissions` = 0 WHERE `inventoryBasePermissions` IS NULL;
UPDATE `inventoryitems` SET `inventoryCurrentPermissions` = 0 WHERE `inventoryCurrentPermissions` IS NULL;
UPDATE `inventoryitems` SET `inventoryEveryOnePermissions` = 0 WHERE `inventoryEveryOnePermissions` IS NULL;
UPDATE `inventoryitems` SET `inventoryNextPermissions` = 0 WHERE `inventoryNextPermissions` IS NULL;
UPDATE `inventoryitems` SET `inventoryGroupPermissions` = 0 WHERE `inventoryGroupPermissions` IS NULL;
UPDATE `inventoryitems` SET `salePrice` = 0 WHERE `salePrice` IS NULL;
UPDATE `inventoryitems` SET `saleType` = 0 WHERE `saleType` IS NULL;
UPDATE `inventoryitems` SET `groupOwned` = 0 WHERE `groupOwned` IS NULL;
UPDATE `inventoryitems` SET `flags` = 0 WHERE `flags` IS NULL;
