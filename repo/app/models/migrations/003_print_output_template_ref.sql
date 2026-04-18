SET @template_id_exists := (
    SELECT COUNT(*)
    FROM information_schema.COLUMNS
    WHERE TABLE_SCHEMA = DATABASE()
      AND TABLE_NAME = 'print_outputs'
      AND COLUMN_NAME = 'template_id'
);

SET @sql := IF(
    @template_id_exists = 0,
    'ALTER TABLE print_outputs ADD COLUMN template_id VARCHAR(100) NULL',
    'SELECT 1'
);
PREPARE stmt FROM @sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @template_version_exists := (
    SELECT COUNT(*)
    FROM information_schema.COLUMNS
    WHERE TABLE_SCHEMA = DATABASE()
      AND TABLE_NAME = 'print_outputs'
      AND COLUMN_NAME = 'template_version_no'
);

SET @sql := IF(
    @template_version_exists = 0,
    'ALTER TABLE print_outputs ADD COLUMN template_version_no INT NULL',
    'SELECT 1'
);
PREPARE stmt FROM @sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @index_exists := (
    SELECT COUNT(*)
    FROM information_schema.STATISTICS
    WHERE TABLE_SCHEMA = DATABASE()
      AND TABLE_NAME = 'print_outputs'
      AND INDEX_NAME = 'idx_print_outputs_template_ref'
);

SET @sql := IF(
    @index_exists = 0,
    'CREATE INDEX idx_print_outputs_template_ref ON print_outputs(template_id, template_version_no, mode)',
    'SELECT 1'
);
PREPARE stmt FROM @sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;
