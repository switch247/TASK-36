# Questions & Clarifications – ProctorOps Exam Administration Platform

This document records ambiguities, assumptions, and design decisions made during development of the ProctorOps Exam Administration platform. Each entry describes a question, our interpretation, and the implemented solution.

---

## 1. Reusable Form Templates – Dynamic UI vs. Backend Storage

**Question:** The prompt requires “reusable form templates for registering candidates, proctors, rooms, and exam sessions” with “required‑field checks, data type and range rules.” Should these templates be stored in the database as JSON schemas, or implemented as hardcoded forms? How are they applied to the UI?

**Understanding:** To be truly reusable and configurable by administrators, templates must be stored in the database and rendered dynamically by the Dioxus frontend. This also aligns with the “offline” requirement – no external services needed.

**Solution:**
- Template table: `form_templates` (id, name, entity_type, schema_json, created_at, version).
- Frontend fetches template by entity type and renders fields dynamically based on schema (type, required, min/max, regex pattern).
- Backend validates submitted data against the stored schema before persistence.

---

## 2. Barcode/QR Scan Input – Hardware Assumptions

**Question:** “Supporting barcode/QR scan input for candidate IDs and booklet/asset tracking” – should the web UI use the device’s camera, or assume a physical USB barcode scanner that acts like a keyboard? The environment is on‑prem, and both options exist.

**Understanding:** A physical scanner that emulates keyboard input is more reliable in high‑stakes exam environments (no permission prompts, faster). However, the UI should also provide a manual input fallback.

**Solution:**
- Input field that automatically captures scanner input (barcode value + Enter key).
- Also provide a “Scan with Camera” button for devices with a camera, using the browser’s `MediaDevices` API (optional, not required for all deployments).
- Metadata captured: scanner device label (if available), timestamp, operator ID.

---

## 3. Duplicate Detection – Threshold Configuration

**Question:** “Guided merge flow when potential duplicates are detected using configurable thresholds (exact match on ID, or 90% similarity on name plus same date of birth).” Should thresholds be configurable by administrators at runtime? Where are they stored?

**Understanding:** Thresholds should be configurable via the admin UI (e.g., similarity percentage, weight fields). Stored in a `settings` table.

**Solution:**
- `duplicate_rules` table: rule_name, field1, field2, similarity_threshold, is_active.
- Default rule: exact match on candidate ID (100%), or name similarity ≥ 90% AND same date of birth.
- When a new candidate is created, backend runs configured rules. If potential duplicate found, returns a `409 Conflict` with list of matching candidates and merge options.
- Frontend shows a modal: “Keep both”, “Merge”, “Cancel”.

---

## 4. Print Modes – “Test Print” Watermark and Final Print Lock

**Question:** “Printing must support a ‘test print’ watermark and a final print mode that locks the template version used.” Does locking the template version mean that subsequent changes to the template should not affect already‑printed outputs? How is the watermark applied?

**Understanding:** For audit compliance, final print mode should capture the exact template version and mark the output as “final”. Watermark on test prints distinguishes draft from official.

**Solution:**
- Template versions: each template has a version number and a snapshot of its schema.
- When generating a final print, the backend stores the template version ID with the output record.
- Watermark on test prints: overlay text “TEST PRINT – Not Official” at 45° angle on PDF/printed page.
- Final print outputs have no watermark but include a digital signature/hash for authenticity.

---

## 5. Offline Operations & Reporting Center – Data Freshness

**Question:** The Operations & Reporting Center shows “seat utilization, materials inventory, near‑expiry items, incident/return rates”. In an offline‑first system, how frequently are these aggregates recalculated? Real‑time or batch?

**Understanding:** Real‑time for critical metrics (seat utilization), but batch (nightly) for historical trends is acceptable to reduce load. Given the on‑prem environment, we can use a local scheduled job.

**Solution:**
- Seat utilization: recalculated on every booking change (real‑time via triggers or materialized view refresh).
- Near‑expiry and incident rates: recomputed hourly by a background worker.
- Dashboards fetch aggregated data from pre‑computed tables to ensure fast response.

---

## 6. Session‑Based Auth vs. JWT – Which One for the Frontend?

**Question:** “The system supports both session‑based auth (30‑minute idle timeout) and JWT (8‑hour expiration).” Should the Dioxus frontend use cookies (session) or Bearer tokens (JWT)? Both are mentioned.

**Understanding:** For an on‑prem web UI, session cookies are more secure (HttpOnly, not accessible to JS) and align with “session‑based auth”. JWT is provided for API clients (e.g., mobile or script access).

**Solution:**
- Frontend uses session cookies (Rocket’s cookie‑based sessions). Idle timeout implemented by tracking user activity in JS and refreshing session via ping endpoint.
- JWT is optional for external integrations; not used by the main Dioxus app.
- Logout destroys server‑side session and clears cookie.

---

## 7. Encrypted at Rest – Which Fields Are Considered Sensitive?

**Question:** “Sensitive fields such as date of birth are encrypted at rest.” Does this include candidate names, addresses, or contact information? The prompt mentions only DOB, but staff contact info is also sensitive.

**Understanding:** To be safe, any personally identifiable information (PII) should be encrypted at rest. This includes candidate name, DOB, national ID, address, phone, email, and staff contact info.

**Solution:**
- Use application‑layer encryption (AES‑256‑GCM) with a key derived from a master secret (stored in environment variable).
- Encrypt: `candidate.full_name`, `candidate.dob`, `candidate.national_id`, `candidate.contact_phone`, `staff.phone`, `staff.email`.
- Decrypt only when needed for reporting/display, with masking for non‑authorized roles.

---

## 8. Guided Merge Flow – Who Can Perform the Merge?

**Question:** “Guided merge flow when potential duplicates are detected.” Should only administrators perform merges, or can exam coordinators also do it? How is the merge decision audited?

**Understanding:** To maintain data integrity, merge operations should be restricted to users with the “Administrator” or “Exam Coordinator” role. All merges are logged in the audit trail.

**Solution:**
- API `POST /candidates/merge` requires role `Admin` or `Coordinator`.
- Request body: `primary_candidate_id`, `secondary_candidate_id`, `fields_to_keep` (mapping).
- Backend creates a new record with merged data, marks secondary as `merged_into`, and logs the action.
- The original records remain (soft‑deleted or flagged) for audit purposes.

---

## 9. Place‑Name Standardization – ZIP/City Reference Table

**Question:** “Performs place‑name standardization using a locally stored ZIP/city reference table rather than any online geocoding.” How is the reference table populated? Must it be pre‑loaded by administrators?

**Understanding:** The reference table must be provided as a seed file during installation (e.g., CSV of known ZIP codes and cities). Administrators can update it via import.

**Solution:**
- Seed migration: `INSERT INTO zip_city (zip_code, city, state) VALUES ...` from a static CSV (e.g., US ZIP data).
- API endpoint `POST /admin/zip-city/import` allows admin to upload new CSV for updates.
- During candidate registration, the system validates the entered city against the ZIP code (soft warning if mismatch).

---

## 10. Output Generation – Admit Cards, Seating Charts, Proctor Packets

**Question:** The prompt lists multiple output types (admit cards, seating charts, door signs, proctor packets, summary reports). Should these be generated as PDFs on‑the‑fly or pre‑rendered? How are they stored?

**Understanding:** For offline printing, outputs should be generated on‑demand as PDF files and optionally saved to a local folder for re‑printing. The UI should allow download or direct print.

**Solution:**
- Endpoint `POST /outputs/generate` accepts `session_id` and `output_type`.
- Backend uses a PDF generation library (e.g., `printpdf` in Rust) to render the template with session data.
- Generated PDF is stored in `./outputs/` with a unique filename and returns a download URL.
- Final print mode locks the template version by storing the template snapshot with the output record.

---

## Summary of Assumptions

| Area | Assumption |
|------|------------|
| Form templates | Stored as JSON schema in DB, rendered dynamically |
| Barcode/QR | Physical scanner + optional camera fallback |
| Duplicate thresholds | Admin‑configurable, stored in DB |
| Print modes | Test watermark, final locks template version |
| Dashboard aggregates | Real‑time for seat utilization, hourly for others |
| Auth for frontend | Session cookies, JWT for external APIs |
| Encrypted fields | All PII (name, DOB, national ID, phone, email) |
| Merge permissions | Admin or Coordinator only, audited |
| ZIP/city table | Seeded from CSV, admin can update |
| PDF generation | On‑demand, stored locally |

All decisions were made to meet the prompt’s offline, on‑prem requirements while ensuring security, auditability, and usability.