SET @blob_exists := (
    SELECT COUNT(*)
    FROM information_schema.COLUMNS
    WHERE TABLE_SCHEMA = DATABASE()
      AND TABLE_NAME = 'attachments'
      AND COLUMN_NAME = 'file_blob'
);

SET @sql := IF(
    @blob_exists = 0,
    'ALTER TABLE attachments ADD COLUMN file_blob LONGBLOB NULL',
    'SELECT 1'
);
PREPARE stmt FROM @sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;
