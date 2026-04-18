# API Integration Tests

This directory contains executable backend integration tests:
- `auth_tests.rs`: auth header/session checks
- `crud_tests.rs`: CRUD contract smoke
- `workflow_tests.rs`: end-to-end workflow smoke
- `error_tests.rs`: error-path smoke

Run via:
- `cargo test -p backend --tests`
