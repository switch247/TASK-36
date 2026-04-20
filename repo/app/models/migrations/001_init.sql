CREATE TABLE IF NOT EXISTS users (
    id CHAR(36) PRIMARY KEY,
    username VARCHAR(100) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role ENUM('Admin', 'Coordinator', 'Proctor', 'Auditor') NOT NULL,
    failed_login_attempts INT NOT NULL DEFAULT 0,
    lockout_until DATETIME NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS user_sessions (
    id CHAR(36) PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    last_activity DATETIME NOT NULL,
    expires_at DATETIME NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS candidates (
    id CHAR(36) PRIMARY KEY,
    encrypted_dob TEXT NOT NULL,
    national_id VARCHAR(64) NOT NULL,
    scanned_barcode VARCHAR(120) NOT NULL,
    metadata JSON NOT NULL,
    created_by CHAR(36) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS rooms (
    id CHAR(36) PRIMARY KEY,
    capacity INT NOT NULL CHECK (capacity BETWEEN 1 AND 500),
    location VARCHAR(255) NOT NULL,
    created_by CHAR(36) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS exam_sessions (
    id CHAR(36) PRIMARY KEY,
    template_name VARCHAR(255) NOT NULL,
    duration_minutes INT NOT NULL CHECK (duration_minutes BETWEEN 15 AND 360),
    status ENUM('Draft', 'Scheduled', 'Active', 'Completed', 'Cancelled', 'FinalPrinted') NOT NULL,
    starts_at DATETIME NOT NULL,
    ends_at DATETIME NOT NULL,
    locked_for_final_print BOOLEAN NOT NULL DEFAULT FALSE,
    created_by CHAR(36) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS assets (
    id CHAR(36) PRIMARY KEY,
    booklet_code VARCHAR(100) NOT NULL UNIQUE,
    tracking_status ENUM('Prepared', 'InTransit', 'Delivered', 'Collected', 'Archived') NOT NULL,
    session_id CHAR(36) NOT NULL,
    expires_on DATE NULL,
    incident_count INT NOT NULL DEFAULT 0,
    created_by CHAR(36) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES exam_sessions(id),
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS attachments (
    id CHAR(36) PRIMARY KEY,
    record_type VARCHAR(64) NOT NULL,
    record_id CHAR(36) NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    extension VARCHAR(16) NOT NULL,
    size_bytes BIGINT NOT NULL,
    fingerprint_sha256 CHAR(64) NOT NULL,
    operator_label VARCHAR(255) NOT NULL,
    device_label VARCHAR(255) NOT NULL,
    captured_at DATETIME NOT NULL,
    created_by CHAR(36) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS merge_candidates (
    id CHAR(36) PRIMARY KEY,
    left_candidate_id CHAR(36) NOT NULL,
    right_candidate_id CHAR(36) NOT NULL,
    similarity_score DOUBLE NOT NULL,
    status ENUM('Pending', 'Approved', 'Rejected') NOT NULL DEFAULT 'Pending',
    created_by CHAR(36) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS template_versions (
    id CHAR(36) PRIMARY KEY,
    template_id VARCHAR(100) NOT NULL,
    version_no INT NOT NULL,
    snapshot JSON NOT NULL,
    locked_for_final_print BOOLEAN NOT NULL DEFAULT FALSE,
    created_by CHAR(36) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY uq_template_version (template_id, version_no),
    FOREIGN KEY (created_by) REFERENCES users(id)
);

-- Audit tables intentionally omit FK constraints on the actor columns.
-- Audit/change history records must survive the deletion or renaming of
-- the originating user, and system-internal recorders may reference synthetic
-- actor ids (health checks, background jobs) that never have a matching
-- users row. `changed_by` / `actor_user_id` are still indexed for lookup.
CREATE TABLE IF NOT EXISTS entity_change_history (
    id CHAR(36) PRIMARY KEY,
    entity_name VARCHAR(64) NOT NULL,
    entity_id CHAR(36) NOT NULL,
    action ENUM('CREATE', 'UPDATE', 'DELETE') NOT NULL,
    previous_state JSON NULL,
    new_state JSON NULL,
    changed_by CHAR(36) NOT NULL,
    changed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_entity_change_history_changed_by (changed_by)
);

CREATE TABLE IF NOT EXISTS audit_logs (
    id CHAR(36) PRIMARY KEY,
    actor_user_id CHAR(36) NULL,
    action VARCHAR(100) NOT NULL,
    resource VARCHAR(255) NOT NULL,
    ip_address VARCHAR(45) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_audit_logs_actor_user_id (actor_user_id)
);

CREATE TABLE IF NOT EXISTS zip_city_reference (
    zip_code VARCHAR(12) PRIMARY KEY,
    city VARCHAR(128) NOT NULL,
    state VARCHAR(128) NULL,
    country VARCHAR(64) NOT NULL DEFAULT 'KE'
);

-- `created_by` is intentionally un-FK'd: message drafts may be produced by
-- system actors (schedulers, alert routers) whose ids don't exist in users,
-- and must outlive user deletion.
CREATE TABLE IF NOT EXISTS message_drafts (
    id CHAR(36) PRIMARY KEY,
    channel ENUM('SMS', 'Email') NOT NULL,
    recipient VARCHAR(255) NOT NULL,
    subject VARCHAR(255) NULL,
    body TEXT NOT NULL,
    created_by CHAR(36) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_message_drafts_created_by (created_by)
);

CREATE TABLE IF NOT EXISTS print_outputs (
    id CHAR(36) PRIMARY KEY,
    session_id CHAR(36) NOT NULL,
    output_type ENUM('AdmitCard', 'SeatingChart', 'DoorSign', 'ProctorPacket', 'SummaryReport') NOT NULL,
    mode ENUM('Draft', 'TestPrint', 'FinalPrint') NOT NULL,
    watermark VARCHAR(255) NULL,
    payload LONGTEXT NOT NULL,
    created_by CHAR(36) NOT NULL,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    FOREIGN KEY (session_id) REFERENCES exam_sessions(id),
    FOREIGN KEY (created_by) REFERENCES users(id)
);

DELIMITER $$
CREATE TRIGGER trg_audit_logs_no_update
BEFORE UPDATE ON audit_logs
FOR EACH ROW
BEGIN
    SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'audit_logs are immutable';
END$$
DELIMITER ;

DELIMITER $$
CREATE TRIGGER trg_audit_logs_no_delete
BEFORE DELETE ON audit_logs
FOR EACH ROW
BEGIN
    SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'audit_logs are immutable';
END$$
DELIMITER ;
