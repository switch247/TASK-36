# Test Coverage Audit

## Scope

- Audit mode: static inspection only. No code, tests, scripts, containers, servers, or package managers were run.
- Project type: `fullstack`, explicitly declared at the top of `README.md` (`README.md:1`).
- Backend route mount: `backend/src/main.rs` mounts `routes_v1()` at `/api/v1`.
- Route source of truth: `app/api/v1/src/lib.rs` plus per-route files in `app/api/v1/src/*.rs`.

## Backend Endpoint Inventory

1. `POST /api/v1/auth/login`
2. `POST /api/v1/users`
3. `GET /api/v1/users`
4. `PUT /api/v1/users/:id`
5. `DELETE /api/v1/users/:id`
6. `POST /api/v1/candidates`
7. `GET /api/v1/candidates`
8. `GET /api/v1/candidates/:id`
9. `PUT /api/v1/candidates/:id`
10. `DELETE /api/v1/candidates/:id`
11. `POST /api/v1/candidates/merge`
12. `POST /api/v1/rooms`
13. `GET /api/v1/rooms`
14. `PUT /api/v1/rooms/:id`
15. `DELETE /api/v1/rooms/:id`
16. `POST /api/v1/sessions`
17. `POST /api/v1/sessions/:id/assignments`
18. `GET /api/v1/sessions`
19. `PUT /api/v1/sessions/:id`
20. `DELETE /api/v1/sessions/:id`
21. `POST /api/v1/assets`
22. `GET /api/v1/assets`
23. `PUT /api/v1/assets/:id`
24. `DELETE /api/v1/assets/:id`
25. `GET /api/v1/reports/dashboard`
26. `GET /api/v1/dashboard/summary`
27. `GET /api/v1/operations/seat-utilization`
28. `GET /api/v1/operations/near-expiry-alerts`
29. `GET /api/v1/operations/incident-rates`
30. `GET /api/v1/operations/return-rates`
31. `GET /api/v1/operations/materials-inventory`
32. `GET /api/v1/operations/alerts`
33. `POST /api/v1/outputs/admit-cards`
34. `POST /api/v1/outputs/seating-charts`
35. `POST /api/v1/outputs/door-signs`
36. `POST /api/v1/outputs/proctor-packet`
37. `POST /api/v1/outputs/summary-report`
38. `POST /api/v1/outputs`
39. `GET /api/v1/outputs`
40. `POST /api/v1/attachments`
41. `GET /api/v1/attachments`
42. `GET /api/v1/attachments/:id`
43. `POST /api/v1/exports/csv`
44. `POST /api/v1/exports/excel`
45. `POST /api/v1/exports/pdf`
46. `POST /api/v1/messages/drafts`
47. `POST /api/v1/templates`
48. `GET /api/v1/templates`
49. `PUT /api/v1/templates/:template_id/:version_no`
50. `DELETE /api/v1/templates/:template_id/:version_no`
51. `POST /api/v1/templates/:template_id/lock`
52. `POST /api/v1/scans/lookup`

## API Test Mapping Table

| Endpoint | Covered | Test type | Test files | Evidence |
|---|---|---|---|---|
| `POST /api/v1/auth/login` | yes | true no-mock HTTP | `API_tests/auth_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `login_success_returns_tokens`, `frontend_models_deserialize_from_live_backend_http_responses`; route `app/api/v1/src/auth.rs:22` |
| `POST /api/v1/users` | yes | true no-mock HTTP | `API_tests/users_tests.rs` | `admin_can_create_user_and_record_is_persisted`; route `app/api/v1/src/users.rs:49` |
| `GET /api/v1/users` | yes | true no-mock HTTP | `API_tests/users_tests.rs` | `list_users_returns_seeded_users_for_admin`; route `app/api/v1/src/users.rs:111` |
| `PUT /api/v1/users/:id` | yes | true no-mock HTTP | `API_tests/users_tests.rs` | `update_user_role_by_admin_persists_change`, `update_user_password_by_admin_allows_new_login`; route `app/api/v1/src/users.rs:130` |
| `DELETE /api/v1/users/:id` | yes | true no-mock HTTP | `API_tests/users_tests.rs` | `delete_user_by_admin_removes_row`; route `app/api/v1/src/users.rs:206` |
| `POST /api/v1/candidates` | yes | true no-mock HTTP | `API_tests/crud_tests.rs`, `API_tests/workflow_tests.rs`, `API_tests/error_tests.rs`, `API_tests/messages_merge_tests.rs` | `create_read_update_delete_candidate_with_auth`, `workflow_create_candidate_session_output_export`; route `app/api/v1/src/candidates.rs:189` |
| `GET /api/v1/candidates` | yes | true no-mock HTTP | `API_tests/auth_tests.rs`, `API_tests/workflow_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `token_can_access_protected_route`, `frontend_models_deserialize_from_live_backend_http_responses`; route `app/api/v1/src/candidates.rs:304` |
| `GET /api/v1/candidates/:id` | yes | true no-mock HTTP | `API_tests/crud_tests.rs`, `API_tests/error_tests.rs` | `create_read_update_delete_candidate_with_auth`, `not_found_resource_returns_404`; route `app/api/v1/src/candidates.rs:384` |
| `PUT /api/v1/candidates/:id` | yes | true no-mock HTTP | `API_tests/crud_tests.rs` | `create_read_update_delete_candidate_with_auth`; route `app/api/v1/src/candidates.rs:418` |
| `DELETE /api/v1/candidates/:id` | yes | true no-mock HTTP | `API_tests/crud_tests.rs` | `create_read_update_delete_candidate_with_auth`; route `app/api/v1/src/candidates.rs:531` |
| `POST /api/v1/candidates/merge` | yes | true no-mock HTTP | `API_tests/messages_merge_tests.rs` | `create_merge_candidate_persists_row_and_audits`; route `app/api/v1/src/candidates.rs:568` |
| `POST /api/v1/rooms` | yes | true no-mock HTTP | `API_tests/rooms_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `create_room_as_coordinator_persists_and_audits`, `frontend_models_deserialize_from_live_backend_http_responses`; route `app/api/v1/src/rooms.rs:33` |
| `GET /api/v1/rooms` | yes | true no-mock HTTP | `API_tests/rooms_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `list_rooms_scoped_to_coordinator`, `frontend_models_deserialize_from_live_backend_http_responses`; route `app/api/v1/src/rooms.rs:111` |
| `PUT /api/v1/rooms/:id` | yes | true no-mock HTTP | `API_tests/rooms_tests.rs` | `update_room_by_owner_persists_and_returns_200`; route `app/api/v1/src/rooms.rs:182` |
| `DELETE /api/v1/rooms/:id` | yes | true no-mock HTTP | `API_tests/rooms_tests.rs` | `delete_room_by_owner_returns_204_and_removes_row`; route `app/api/v1/src/rooms.rs:276` |
| `POST /api/v1/sessions` | yes | true no-mock HTTP | `API_tests/workflow_tests.rs`, `API_tests/sessions_tests.rs` | `workflow_create_candidate_session_output_export`, `create_session_invalid_duration_returns_400`; route `app/api/v1/src/sessions.rs:57` |
| `POST /api/v1/sessions/:id/assignments` | yes | true no-mock HTTP | `API_tests/sessions_tests.rs`, `API_tests/output_tests.rs` | `list_sessions_proctor_sees_assigned`, `final_print_allowed_for_proctor_and_coordinator`; route `app/api/v1/src/sessions.rs:233` |
| `GET /api/v1/sessions` | yes | true no-mock HTTP | `API_tests/sessions_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `list_sessions_coordinator_sees_only_own`, `frontend_models_deserialize_from_live_backend_http_responses`; route `app/api/v1/src/sessions.rs:141` |
| `PUT /api/v1/sessions/:id` | yes | true no-mock HTTP | `API_tests/sessions_tests.rs` | `update_session_by_coordinator_persists`; route `app/api/v1/src/sessions.rs:307` |
| `DELETE /api/v1/sessions/:id` | yes | true no-mock HTTP | `API_tests/sessions_tests.rs` | `delete_session_by_owner_returns_204`; route `app/api/v1/src/sessions.rs:412` |
| `POST /api/v1/assets` | yes | true no-mock HTTP | `API_tests/assets_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `create_asset_by_coordinator_persists`, `frontend_models_deserialize_from_live_backend_http_responses`; route `app/api/v1/src/assets.rs:36` |
| `GET /api/v1/assets` | yes | true no-mock HTTP | `API_tests/assets_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `list_assets_restricted_by_ownership`, `frontend_models_deserialize_from_live_backend_http_responses`; route `app/api/v1/src/assets.rs:85` |
| `PUT /api/v1/assets/:id` | yes | true no-mock HTTP | `API_tests/assets_tests.rs` | `update_asset_by_owner_changes_tracking_status`; route `app/api/v1/src/assets.rs:166` |
| `DELETE /api/v1/assets/:id` | yes | true no-mock HTTP | `API_tests/assets_tests.rs` | `delete_asset_by_owner_returns_204`; route `app/api/v1/src/assets.rs:235` |
| `GET /api/v1/reports/dashboard` | yes | true no-mock HTTP | `API_tests/reports_tests.rs`, `API_tests/error_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `reports_dashboard_success_returns_counts_for_coordinator`, `forbidden_role_returns_403`, `frontend_dashboard_summary_model_can_be_composed_from_live_endpoints`; route `app/api/v1/src/reports.rs:56` |
| `GET /api/v1/dashboard/summary` | yes | true no-mock HTTP | `API_tests/reports_tests.rs` | `dashboard_summary_success_for_admin`; route `app/api/v1/src/reports.rs:101` |
| `GET /api/v1/operations/seat-utilization` | yes | true no-mock HTTP | `API_tests/workflow_tests.rs` | `reports_pagination_filtering_and_sorting_boundaries`; route `app/api/v1/src/reports.rs:177` |
| `GET /api/v1/operations/near-expiry-alerts` | yes | true no-mock HTTP | `API_tests/reports_tests.rs` | `near_expiry_alerts_returns_assets_close_to_expiration`; route `app/api/v1/src/reports.rs:229` |
| `GET /api/v1/operations/incident-rates` | yes | true no-mock HTTP | `API_tests/reports_tests.rs`, `API_tests/error_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `incident_rates_returns_data_for_reporting_roles`, `missing_auth_returns_401`, `frontend_report_models_and_scan_model_match_backend_payloads`; routes `app/api/v1/src/reports.rs:286` and `app/api/v1/src/reports.rs:487` |
| `GET /api/v1/operations/return-rates` | yes | true no-mock HTTP | `API_tests/reports_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `return_rates_success_for_coordinator`, `frontend_report_models_and_scan_model_match_backend_payloads`; route `app/api/v1/src/reports.rs:336` |
| `GET /api/v1/operations/materials-inventory` | yes | true no-mock HTTP | `API_tests/reports_tests.rs`, `API_tests/workflow_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `materials_inventory_returns_asset_rows`, `reports_pagination_filtering_and_sorting_boundaries`; route `app/api/v1/src/reports.rs:388` |
| `GET /api/v1/operations/alerts` | yes | true no-mock HTTP | `API_tests/reports_tests.rs`, `API_tests/workflow_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `operations_alerts_returns_ok_with_within_days`, `reports_pagination_filtering_and_sorting_boundaries`; route `app/api/v1/src/reports.rs:450` |
| `POST /api/v1/outputs/admit-cards` | yes | true no-mock HTTP | `API_tests/outputs_list_tests.rs` | `print_admit_cards_endpoint_produces_admit_card_output`; route `app/api/v1/src/outputs.rs:194` |
| `POST /api/v1/outputs/seating-charts` | yes | true no-mock HTTP | `API_tests/outputs_list_tests.rs` | `print_seating_charts_endpoint_returns_seating_chart`; route `app/api/v1/src/outputs.rs:204` |
| `POST /api/v1/outputs/door-signs` | yes | true no-mock HTTP | `API_tests/outputs_list_tests.rs` | `print_door_signs_endpoint_returns_door_sign`; route `app/api/v1/src/outputs.rs:214` |
| `POST /api/v1/outputs/proctor-packet` | yes | true no-mock HTTP | `API_tests/outputs_list_tests.rs` | `print_proctor_packet_endpoint_returns_proctor_packet`; route `app/api/v1/src/outputs.rs:224` |
| `POST /api/v1/outputs/summary-report` | yes | true no-mock HTTP | `API_tests/outputs_list_tests.rs` | `print_summary_report_endpoint_returns_summary_report`; route `app/api/v1/src/outputs.rs:234` |
| `POST /api/v1/outputs` | yes | true no-mock HTTP | `API_tests/workflow_tests.rs`, `API_tests/output_tests.rs`, `API_tests/outputs_list_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `workflow_create_candidate_session_output_export`, `final_print_allowed_for_proctor_and_coordinator`; route `app/api/v1/src/outputs.rs:87` |
| `GET /api/v1/outputs` | yes | true no-mock HTTP | `API_tests/outputs_list_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `list_outputs_returns_recent_outputs`, `frontend_models_deserialize_from_live_backend_http_responses`; routes `app/api/v1/src/outputs.rs:127` and `app/api/v1/src/outputs.rs:174` |
| `POST /api/v1/attachments` | yes | true no-mock HTTP | `API_tests/workflow_tests.rs`, `API_tests/error_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `attachment_upload_and_retrieval_roundtrip`, `duplicate_attachment_fingerprint_returns_409`; route `app/api/v1/src/outputs.rs:269` |
| `GET /api/v1/attachments` | yes | true no-mock HTTP | `API_tests/workflow_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `attachment_upload_and_retrieval_roundtrip`, `frontend_models_deserialize_from_live_backend_http_responses`; route `app/api/v1/src/outputs.rs:355` |
| `GET /api/v1/attachments/:id` | yes | true no-mock HTTP | `API_tests/workflow_tests.rs` | `attachment_upload_and_retrieval_roundtrip`; route `app/api/v1/src/outputs.rs:402` |
| `POST /api/v1/exports/csv` | yes | true no-mock HTTP | `API_tests/workflow_tests.rs`, `API_tests/output_tests.rs`, `API_tests/exports_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `workflow_create_candidate_session_output_export`, `export_csv_writes_audit_log`; route `app/api/v1/src/exports.rs:26` |
| `POST /api/v1/exports/excel` | yes | true no-mock HTTP | `API_tests/exports_tests.rs` | `export_excel_returns_tsv_with_expected_header`; route `app/api/v1/src/exports.rs:48` |
| `POST /api/v1/exports/pdf` | yes | true no-mock HTTP | `API_tests/exports_tests.rs` | `export_pdf_returns_placeholder_with_title`; route `app/api/v1/src/exports.rs:70` |
| `POST /api/v1/messages/drafts` | yes | true no-mock HTTP | `API_tests/messages_merge_tests.rs` | `create_message_draft_persists_for_coordinator`; route `app/api/v1/src/outputs.rs:244` |
| `POST /api/v1/templates` | yes | true no-mock HTTP | `API_tests/templates_tests.rs` | `create_template_new_version_persists`; route `app/api/v1/src/templates.rs:85` |
| `GET /api/v1/templates` | yes | true no-mock HTTP | `API_tests/templates_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `list_templates_returns_seeded_and_requires_admin_or_coord`, `frontend_models_deserialize_from_live_backend_http_responses`; route `app/api/v1/src/templates.rs:122` |
| `PUT /api/v1/templates/:template_id/:version_no` | yes | true no-mock HTTP | `API_tests/templates_tests.rs`, `API_tests/output_tests.rs` | `update_template_success_path_mutates_unlocked_version`, `final_print_allowed_for_proctor_and_coordinator`; route `app/api/v1/src/templates.rs:142` |
| `DELETE /api/v1/templates/:template_id/:version_no` | yes | true no-mock HTTP | `API_tests/templates_tests.rs`, `API_tests/output_tests.rs` | `delete_template_success_removes_unlocked_version`, `final_print_allowed_for_proctor_and_coordinator`; route `app/api/v1/src/templates.rs:192` |
| `POST /api/v1/templates/:template_id/lock` | yes | true no-mock HTTP | `API_tests/templates_tests.rs` | `lock_template_creates_a_version`; route `app/api/v1/src/templates.rs:230` |
| `POST /api/v1/scans/lookup` | yes | true no-mock HTTP | `API_tests/workflow_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `scan_asset_lookup_returns_asset_match`, `frontend_report_models_and_scan_model_match_backend_payloads`; route `app/api/v1/src/scans.rs:45` |

## API Test Classification

### 1. True No-Mock HTTP

- All endpoint-hitting API tests in `API_tests/*.rs`.
- Evidence: `API_tests/common.rs` builds a Rocket app with real `AuthService`, `CandidateService`, `CleansingService`, `AuditService`, `ReportingService`, `OutputService`, and `MessagingService`, mounts `routes_v1()` at `/api/v1`, and uses `rocket::local::asynchronous::Client::tracked(...)`.
- Evidence: `API_tests/common.rs` provisions a real MySQL database, applies migrations, seeds data, and verifies DB side effects directly.

### 2. HTTP with Mocking

- None detected.

### 3. Non-HTTP (unit/integration without HTTP)

- `unit_tests/auth_service_tests.rs`
- `unit_tests/candidate_service_tests.rs`
- `unit_tests/reporting_service_tests.rs`
- `unit_tests/template_service_tests.rs`
- `unit_tests/output_service_tests.rs`
- `unit_tests/messaging_service_tests.rs`
- `unit_tests/audit_service_tests.rs`
- `unit_tests/file_handling_service_tests.rs`
- `unit_tests/dedupe_service_tests.rs`
- `unit_tests/cleansing_service_tests.rs`
- `unit_tests/rbac_service_tests.rs`
- `unit_tests/auth_policy_tests.rs`
- `unit_tests/file_policy_tests.rs`
- `unit_tests/session_core_tests.rs`
- `unit_tests/dedupe_tests.rs`
- `unit_tests/cleansing_tests.rs`
- `unit_tests/encryption_tests.rs`
- `unit_tests/api_support_tests.rs`
- `unit_tests/template_validation_tests.rs`

Note: `unit_tests/shared_guard_tests.rs` is HTTP-shaped, but it tests the auth guard on a synthetic `/whoami` route, not a production API endpoint, so it is excluded from endpoint coverage.

## Mock Detection

- Explicit mock/stub framework hits: none found in `API_tests`, `unit_tests`, or frontend tests.
- No `jest.mock`, `vi.mock`, `sinon.stub`, DI override pattern, or controller/service replacement was found in the audited test files.
- Direct evidence of real wiring:
  - `API_tests/common.rs`: mounts `routes_v1()` and manages real services.
  - `API_tests/common.rs`: creates and seeds a real SQLx MySQL pool and checks DB mutations after requests.

## Coverage Summary

- Total endpoints: `52`
- Endpoints with HTTP tests: `52`
- Endpoints with true no-mock HTTP tests: `52`
- HTTP coverage: `100.0%`
- True API coverage: `100.0%`

Strict note: `GET /api/v1/operations/incident-rates` and `GET /api/v1/outputs` each have two Rocket handlers that share the same method+path but differ by query shape. Under the stated endpoint definition, they count as one endpoint each. Tests exercise both path variants.

## Unit Test Summary

### Backend Unit Tests

- Test files registered in `backend/Cargo.toml`: `unit_tests/auth_policy_tests.rs`, `cleansing_tests.rs`, `dedupe_tests.rs`, `encryption_tests.rs`, `rbac_service_tests.rs`, `file_handling_service_tests.rs`, `template_service_tests.rs`, `output_service_tests.rs`, `file_policy_tests.rs`, `session_core_tests.rs`, `dedupe_service_tests.rs`, `api_support_tests.rs`, `auth_service_tests.rs`, `candidate_service_tests.rs`, `audit_service_tests.rs`, `messaging_service_tests.rs`, `reporting_service_tests.rs`, `cleansing_service_tests.rs`, `template_validation_tests.rs`, `shared_guard_tests.rs`.

- Controllers / API support covered:
  - `unit_tests/api_support_tests.rs`: validators, prompt datetime parsing, API error constructors.
  - `unit_tests/template_validation_tests.rs`: template-driven input validation.
  - `unit_tests/shared_guard_tests.rs`: `ApiContext` auth guard behavior.

- Services covered:
  - `unit_tests/auth_service_tests.rs`: password hashing, JWT validation, key generation.
  - `unit_tests/candidate_service_tests.rs`: DOB normalization and AES key wiring.
  - `unit_tests/reporting_service_tests.rs`: report aggregation methods.
  - Additional service files present by filename: `template_service_tests.rs`, `output_service_tests.rs`, `messaging_service_tests.rs`, `audit_service_tests.rs`, `file_handling_service_tests.rs`, `dedupe_service_tests.rs`, `cleansing_service_tests.rs`, `rbac_service_tests.rs`.

- Core/domain modules covered:
  - `unit_tests/auth_policy_tests.rs`
  - `unit_tests/file_policy_tests.rs`
  - `unit_tests/session_core_tests.rs`
  - `unit_tests/dedupe_tests.rs`
  - `unit_tests/cleansing_tests.rs`
  - `unit_tests/encryption_tests.rs`

- Repositories covered:
  - No direct repository-focused unit test file was found for `app/models/src/repository.rs`.

- Important backend modules not directly unit-tested:
  - `app/models/src/repository.rs`
  - `backend/src/main.rs` bootstrapping and CORS wiring
  - Route modules themselves as unit targets: `app/api/v1/src/users.rs`, `candidates.rs`, `rooms.rs`, `sessions.rs`, `assets.rs`, `reports.rs`, `outputs.rs`, `exports.rs`, `templates.rs`, `scans.rs`

### Frontend Unit Tests

- Frontend unit tests: PRESENT

- Frontend test files:
  - `frontend/tests/components_spec.rs`
  - `frontend/tests/auth_spec.rs`
  - `frontend/tests/helpers_spec.rs`
  - `frontend/src/component_tests.rs`
  - Inline unit module in `frontend/src/main.rs`

- Frameworks/tools detected:
  - Rust built-in test harness (`#[test]`)
  - Dioxus component rendering through exported component/helper functions

- Components/modules covered:
  - Render helpers and tables: `dashboard_view`, `metric`, `spinner`, `table_candidates`, `table_rooms`, `table_sessions`, `table_assets`, `table_users`, `table_outputs`, `table_attachments`, `table_alerts`, `table_materials_inventory`
  - Auth helper: `jwt_role`
  - Formatting/helpers: `toast_bg`, `trend_points`
  - Additional pure helpers in inline tests: `parse_datetime_components`, `format_mmddyyyy_hhmm_ampm`, `api_base`, `gen_id`, route enum construction

- Evidence:
  - `frontend/tests/components_spec.rs:9-101`
  - `frontend/tests/auth_spec.rs:4-30`
  - `frontend/tests/helpers_spec.rs`
  - `frontend/src/component_tests.rs`
  - `frontend/src/main.rs:1318+` inline tests

- Important frontend components/modules not unit-tested:
  - Page components: `Login`, `Dashboard`, `candidates_page`, `rooms_page`, `proctors_page`, `exams_page`, `list_page_sessions`, `list_page_assets`, `reports_page`, `templates_page`, `outputs_page`, `admin_page`
  - Shell/navigation and route gating: `Shell`, `menu`, `require_auth`
  - Network/storage helpers: `get_json`, `post_json`, `load_session`, `save_session`, `clear_session`
  - File retrieval behavior around `AttachmentFileResp`

- Frontend unit verdict:
  - Present, but substantially thinner than backend coverage.

### Cross-Layer Observation

- Backend testing is much stronger than frontend unit testing.
- This is partially compensated by a real browser E2E artifact at `e2e/fullstack_e2e.js`, which covers login, candidate creation, session assignment, outputs, exports, dashboard load, and role restrictions through the UI.
- Because frontend unit tests do exist and browser E2E is also present, this is not a strict frontend-test absence. It is still a balance gap.

## API Observability Check

- Verdict: strong
- Reason:
  - Tests usually show the exact method and path.
  - Request bodies are explicit JSON literals in the test.
  - Response status and response body fields are asserted directly.
  - Many tests also verify persistence side effects in MySQL.
- Representative evidence:
  - `API_tests/crud_tests.rs`: create/read/update/delete candidate payloads and response assertions.
  - `API_tests/users_tests.rs`: explicit error-body assertions for invalid role, weak password, duplicates, auth failures.
  - `API_tests/workflow_tests.rs`: full request/response chain across candidate, session, output, export.

## Tests Check

- Success paths: strong
  - CRUD happy paths, reporting reads, exports, output generation, attachments, scans.
- Failure cases: strong
  - 400, 401, 403, 404, 409 cases are widely covered.
- Edge cases: moderate to strong
  - Duplicate candidate detection, duplicate attachment fingerprint, attachment count limit, invalid date/time parsing, session assignment role validation, final-print lock behavior.
- Validation: strong
  - Candidate, room, session, template validation and helper-level validator tests exist.
- Auth/permissions: strong
  - Missing auth, invalid token/session, role restrictions, ownership scoping, coordinator/admin/proctor/auditor differences are tested.
- Integration boundaries: strong on backend; moderate on frontend
  - Backend HTTP tests hit real handlers and DB.
  - Frontend has browser E2E present and model contract tests, but unit coverage is mostly render/helper oriented.
- Assertion depth: generally meaningful
  - Most tests assert body fields and DB effects, not only status codes.
- Autogenerated/shallow signal: limited, but frontend component tests are shallow render checks rather than interaction-level unit tests.
- `run_tests.sh` check:
  - Docker-based: OK (`run_tests.sh:23-33`)
  - Local dependency requirement: only Docker in PATH (`run_tests.sh:7-10`), which is acceptable under the stated rule.

## End-to-End Expectations

- Fullstack expectation: real FE ↔ BE tests should exist.
- Evidence found:
  - `e2e/fullstack_e2e.js` uses Playwright against the browser UI and authenticated API calls.
  - `run_tests.sh:32-33` runs the browser E2E suite inside Docker.
- Verdict:
  - Present.
  - Strong API coverage means the platform is not relying on E2E alone.

## Test Coverage Score (0–100)

- Score: `90/100`

## Score Rationale

- High score justified by:
  - Full endpoint inventory covered by real HTTP tests.
  - No explicit mocking detected in the endpoint suite.
  - Wide auth, validation, error, and ownership coverage.
  - Backend unit suite spans core, service, and guard concerns.
  - Frontend unit tests exist and browser E2E is present.

- Score not higher because:
  - Frontend unit coverage is shallow relative to backend depth.
  - Key frontend page components and HTTP/storage helpers are not unit-tested.
  - Repository layer lacks direct unit coverage.
  - Some endpoints are covered mostly by one or two tests rather than a deep matrix.

## Key Gaps

- Frontend unit depth is materially weaker than backend unit/API depth.
- No direct unit coverage for `app/models/src/repository.rs`.
- Backend startup/CORS wiring in `backend/src/main.rs` is not directly tested.
- Many frontend page-level behaviors rely on E2E only, not focused unit tests.

## Confidence & Assumptions

- Confidence: high
- Assumptions:
  - Endpoint inventory is based on visible Rocket route macros and `/api/v1` mount wiring.
  - Coverage is counted by visible request paths in tests; runtime-only route behavior was not executed.
  - No hidden generated tests were assumed.

# README Audit

## README Location

- Found at required path: `README.md`

## Hard Gate Review

### Formatting

- Pass with a minor defect.
- Structure is readable and logically ordered.
- Minor defect: visible encoding corruption appears in section titles/arrows (`README.md:51`, `README.md:82`, `README.md:89`).

### Startup Instructions

- Pass.
- Required `docker-compose up` command is present (`README.md:18-20`).
- Scope of what starts is stated (`README.md:22-28`).

### Access Method

- Pass.
- Frontend URL and backend URL are explicitly documented (`README.md:30-36`).

### Verification Method

- Pass.
- API verification via `curl` is explicit (`README.md:49-80`).
- Web verification via UI flow is explicit (`README.md:82-91`).

### Environment Rules

- Pass.
- README explicitly forbids extra local tooling need (`README.md:7-12`).
- No prohibited runtime install instructions were found.
- Test instructions stay Docker-contained (`README.md:93-101`).

### Demo Credentials

- Pass.
- Auth exists and all four roles include username plus password (`README.md:38-47`).

## Engineering Quality

- Tech stack clarity: good (`README.md:5`)
- Architecture explanation: minimal
  - Stack is named, but no module/service topology or data-flow explanation is given.
- Testing instructions: partial
  - README documents only backend `cargo test -p backend --tests` inside Docker (`README.md:97-101`).
  - It does not mention the separate frontend and browser E2E stages that `run_tests.sh` actually runs (`run_tests.sh:26-33`).
- Security/roles: acceptable
  - Roles and credentials are documented.
  - Session/JWT expectations are shown in the API smoke test.
- Workflow coverage: acceptable
  - Startup, access, verification, tests, and shutdown are all present.
- Presentation quality: acceptable with one correctness issue
  - `localhost:3306 (internal only)` is contradictory wording (`README.md:36`). If it is internal only, `localhost` is the wrong description.

## High Priority Issues

- README test instructions understate the actual test surface. The documented command only covers backend tests (`README.md:97-101`), while the repository test runner also executes frontend tests and browser E2E (`run_tests.sh:26-33`). This is a documentation accuracy issue, not a hard-gate failure.

## Medium Priority Issues

- README lacks a concrete architecture section. The stack is named (`README.md:5`), but the boundary between Rocket API, Dioxus frontend, and MySQL-backed services is not explained.
- The MySQL access line is imprecise: `localhost:3306 (internal only)` is internally inconsistent (`README.md:36`).

## Low Priority Issues

- Text encoding artifacts reduce polish and readability in verification headings and arrow text (`README.md:51`, `README.md:82`, `README.md:89`).

## Hard Gate Failures

- None

## README Verdict

- PASS

## README Rationale

- All required hard gates are satisfied:
  - project type declared
  - Docker startup command present
  - access URLs present
  - verification steps present
  - no forbidden install/manual DB setup guidance
  - auth credentials for all roles present
- Remaining issues affect fidelity and quality, not baseline compliance.
