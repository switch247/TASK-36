CREATE TABLE IF NOT EXISTS exam_session_assignments (
    id CHAR(36) PRIMARY KEY,
    session_id CHAR(36) NOT NULL,
    user_id CHAR(36) NOT NULL,
    assigned_by CHAR(36) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY uq_session_assignee (session_id, user_id),
    FOREIGN KEY (session_id) REFERENCES exam_sessions(id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (assigned_by) REFERENCES users(id)
);

CREATE INDEX idx_session_assignments_user ON exam_session_assignments(user_id);
