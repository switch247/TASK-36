SET @precision_exists := (
    SELECT COUNT(*)
    FROM information_schema.COLUMNS
    WHERE TABLE_SCHEMA = DATABASE()
      AND TABLE_NAME = 'print_outputs'
      AND COLUMN_NAME = 'created_at'
      AND DATETIME_PRECISION = 6
);

SET @sql := IF(
    @precision_exists = 0,
    'ALTER TABLE print_outputs MODIFY COLUMN created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6)',
    'SELECT 1'
);
PREPARE stmt FROM @sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;
