# Architecture Overview

## Workspace
- `backend`: Rocket API bootstrap and state wiring.
- `frontend`: Dioxus web shell.
- `app/core`: shared auth/session/crypto/types + governance validation primitives.
- `app/models`: DB entities and migrations.
- `app/api/v1`: modular API controllers.
- `app/services`: business logic for auth, candidate handling, cleansing, dedupe, file policy, templates, reporting, outputs, and messaging.

## Security Model
- Passwords hashed with bcrypt and complexity policy:
  - minimum 12 chars
  - uppercase, lowercase, digit, special char required
- Account lockout after 5 failures for 15 minutes.
- Dual token model:
  - JWT (8-hour expiry)
  - Session record with 30-minute idle timeout (`user_sessions`)
- Protected route authentication requires both:
  - `Authorization: Bearer <jwt>`
  - `x-session-id: <session_id>`
- Role is resolved only from verified JWT claims.
- Object-level authorization enforced with `created_by` filters for non-admin users.
- Candidate DOB encrypted at application layer using AES-256-GCM.
- `audit_logs` immutable via update/delete blocking triggers.

## Data Governance Pipeline
- Unit standardization, currency normalization to USD, and date normalization.
- ZIP/City validation against `zip_city_reference`.
- Room capacity outlier detection (>3x average).
- Guided merge logic with persistence in `merge_candidates`.

## Output/Compliance
- Output types: AdmitCard, SeatingChart, DoorSign, ProctorPacket, SummaryReport.
- Final print locks session record.
- Attachments persisted with SHA-256 fingerprints and capture metadata.
- Version and change history support via:
  - `template_versions`
  - `entity_change_history`

## Run
1. `cp .env.example .env`
2. Set secure secrets in `.env`.
3. `docker compose up --build`
4. `cargo test -p backend --tests`
