fullstack

# ProctorOps Exam Platform

A fullstack exam operations platform (Rust + Rocket backend, Dioxus/WASM frontend, MySQL database) for managing candidates, rooms, sessions, assets, proctor assignments, templates, outputs, and reporting.

## Requirements

- Docker
- Docker Compose

No other local tooling (no Rust, no MySQL, no Node) is required.

## Startup

From the repository root:

```
docker-compose up
```

This single command builds and starts every service:

- MySQL database (schema + seed data applied automatically)
- Rust/Rocket backend API
- Dioxus frontend web app

Wait until the logs show the backend listening on port 8000 and the frontend serving on port 8080.

## Access

| Service     | URL                            |
|-------------|--------------------------------|
| Frontend    | http://localhost:8080          |
| Backend API | http://localhost:8000/api/v1   |
| MySQL       | localhost:3306 (internal only) |

## Demo Credentials

All four roles are pre-seeded and ready to use:

| Role        | Username         | Password              |
|-------------|------------------|-----------------------|
| Admin       | admin_local      | AdminPass#2026!       |
| Coordinator | coord_local      | CoordPass#2026!       |
| Proctor     | proctor_local    | ProctorPass#2026!     |
| Auditor     | auditor_local    | AuditorPass#2026!     |

## Verify the System Works

### Option A — API smoke test (curl)

1. Log in as admin and capture the JWT + session id:

   ```
   curl -s -X POST http://localhost:8000/api/v1/auth/login \
     -H "Content-Type: application/json" \
     -d '{"username":"admin_local","password":"AdminPass#2026!"}'
   ```

   Expected: HTTP 200 with a JSON body shaped like:

   ```
   {
     "session_id": "<uuid>",
     "session_expires_at": "<rfc3339>",
     "jwt": "<header.payload.signature>",
     "jwt_expires_at": "<rfc3339>"
   }
   ```

2. Call a protected endpoint using the returned credentials:

   ```
   curl -s http://localhost:8000/api/v1/users \
     -H "Authorization: Bearer <jwt>" \
     -H "x-session-id: <session_id>"
   ```

   Expected: HTTP 200 with a JSON array containing `admin_local`, `coord_local`, `proctor_local`, and `auditor_local`.

### Option B — Frontend login flow

1. Open http://localhost:8080.
2. Sign in as `coord_local` / `CoordPass#2026!`.
3. Navigate to **Candidates** and create a candidate (DOB `03/27/2001`, any national id/barcode, pick a room).
4. Navigate to **Sessions** and create a session using template `base-template`, duration `90`, starts `03/27/2026 09:00 AM`, ends `03/27/2026 10:30 AM`.
5. Navigate to **Outputs**, generate an **Admit Card** in **Draft** mode for that session.
6. Navigate to **Reports → Dashboard**; the counters update to reflect the new candidate, session, and output.

If every step above completes without an error toast, the fullstack is healthy.

## Running Tests

Tests run exclusively inside Docker. From the repo root:

```
docker-compose run --rm backend cargo test -p backend --tests
```

This executes unit tests and the full HTTP integration suite (auth, CRUD, users, rooms, assets, sessions, templates, reports, outputs, exports, messages, merge, workflow, errors) against an isolated MySQL test database spun up inside the compose network.

## Stopping

```
docker-compose down
```

Add `-v` to also drop the database volume for a clean reset.
