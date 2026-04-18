# Test Coverage Audit

## Scope and Method

- Static inspection only. No code, tests, scripts, containers, servers, or package managers were run.
- Project type declaration is present as `fullstack` at [README.md](C:\Users\kidus\OneDrive\Desktop\ProctorOps-exam-main\repo\README.md#L1).
- Backend route mount is `/api/v1` in `backend/src/main.rs:58`.
- Registered API handlers are enumerated in `app/api/v1/src/lib.rs:33-89`.
- Endpoint inventory below normalizes parameterized paths and de-duplicates duplicate Rocket route entries that share the same `METHOD + PATH`:
  - `GET /api/v1/outputs` appears twice in code (`app/api/v1/src/outputs.rs:127`, `app/api/v1/src/outputs.rs:174`) but is one endpoint.
  - `GET /api/v1/operations/incident-rates` appears twice in code (`app/api/v1/src/reports.rs:286`, `app/api/v1/src/reports.rs:487`) but is one endpoint.

## Backend Endpoint Inventory

Total unique endpoints: `52`

| Endpoint | Route evidence |
|---|---|
| `POST /api/v1/auth/login` | `app/api/v1/src/auth.rs:22` |
| `POST /api/v1/users` | `app/api/v1/src/users.rs:49` |
| `GET /api/v1/users` | `app/api/v1/src/users.rs:111` |
| `PUT /api/v1/users/{id}` | `app/api/v1/src/users.rs:130` |
| `DELETE /api/v1/users/{id}` | `app/api/v1/src/users.rs:206` |
| `POST /api/v1/candidates` | `app/api/v1/src/candidates.rs:189` |
| `GET /api/v1/candidates` | `app/api/v1/src/candidates.rs:304` |
| `GET /api/v1/candidates/{id}` | `app/api/v1/src/candidates.rs:384` |
| `PUT /api/v1/candidates/{id}` | `app/api/v1/src/candidates.rs:418` |
| `DELETE /api/v1/candidates/{id}` | `app/api/v1/src/candidates.rs:531` |
| `POST /api/v1/candidates/merge` | `app/api/v1/src/candidates.rs:568` |
| `POST /api/v1/rooms` | `app/api/v1/src/rooms.rs:33` |
| `GET /api/v1/rooms` | `app/api/v1/src/rooms.rs:111` |
| `PUT /api/v1/rooms/{id}` | `app/api/v1/src/rooms.rs:182` |
| `DELETE /api/v1/rooms/{id}` | `app/api/v1/src/rooms.rs:276` |
| `POST /api/v1/sessions` | `app/api/v1/src/sessions.rs:57` |
| `GET /api/v1/sessions` | `app/api/v1/src/sessions.rs:141` |
| `POST /api/v1/sessions/{id}/assignments` | `app/api/v1/src/sessions.rs:233` |
| `PUT /api/v1/sessions/{id}` | `app/api/v1/src/sessions.rs:307` |
| `DELETE /api/v1/sessions/{id}` | `app/api/v1/src/sessions.rs:412` |
| `POST /api/v1/assets` | `app/api/v1/src/assets.rs:36` |
| `GET /api/v1/assets` | `app/api/v1/src/assets.rs:85` |
| `PUT /api/v1/assets/{id}` | `app/api/v1/src/assets.rs:166` |
| `DELETE /api/v1/assets/{id}` | `app/api/v1/src/assets.rs:235` |
| `GET /api/v1/reports/dashboard` | `app/api/v1/src/reports.rs:56` |
| `GET /api/v1/dashboard/summary` | `app/api/v1/src/reports.rs:101` |
| `GET /api/v1/operations/seat-utilization` | `app/api/v1/src/reports.rs:177` |
| `GET /api/v1/operations/near-expiry-alerts` | `app/api/v1/src/reports.rs:229` |
| `GET /api/v1/operations/incident-rates` | `app/api/v1/src/reports.rs:286`, `app/api/v1/src/reports.rs:487` |
| `GET /api/v1/operations/return-rates` | `app/api/v1/src/reports.rs:336` |
| `GET /api/v1/operations/materials-inventory` | `app/api/v1/src/reports.rs:388` |
| `GET /api/v1/operations/alerts` | `app/api/v1/src/reports.rs:450` |
| `POST /api/v1/outputs` | `app/api/v1/src/outputs.rs:87` |
| `GET /api/v1/outputs` | `app/api/v1/src/outputs.rs:127`, `app/api/v1/src/outputs.rs:174` |
| `POST /api/v1/outputs/admit-cards` | `app/api/v1/src/outputs.rs:194` |
| `POST /api/v1/outputs/seating-charts` | `app/api/v1/src/outputs.rs:204` |
| `POST /api/v1/outputs/door-signs` | `app/api/v1/src/outputs.rs:214` |
| `POST /api/v1/outputs/proctor-packet` | `app/api/v1/src/outputs.rs:224` |
| `POST /api/v1/outputs/summary-report` | `app/api/v1/src/outputs.rs:234` |
| `POST /api/v1/messages/drafts` | `app/api/v1/src/outputs.rs:244` |
| `POST /api/v1/attachments` | `app/api/v1/src/outputs.rs:269` |
| `GET /api/v1/attachments` | `app/api/v1/src/outputs.rs:355` |
| `GET /api/v1/attachments/{id}` | `app/api/v1/src/outputs.rs:402` |
| `POST /api/v1/exports/csv` | `app/api/v1/src/exports.rs:26` |
| `POST /api/v1/exports/excel` | `app/api/v1/src/exports.rs:48` |
| `POST /api/v1/exports/pdf` | `app/api/v1/src/exports.rs:70` |
| `POST /api/v1/templates` | `app/api/v1/src/templates.rs:85` |
| `GET /api/v1/templates` | `app/api/v1/src/templates.rs:122` |
| `PUT /api/v1/templates/{template_id}/{version_no}` | `app/api/v1/src/templates.rs:142` |
| `DELETE /api/v1/templates/{template_id}/{version_no}` | `app/api/v1/src/templates.rs:192` |
| `POST /api/v1/templates/{template_id}/lock` | `app/api/v1/src/templates.rs:230` |
| `POST /api/v1/scans/lookup` | `app/api/v1/src/scans.rs:45` |

## API Test Classification

### True No-Mock HTTP

- All endpoint-covering tests under `API_tests/*.rs` fall in this class.
- Evidence:
  - Real app bootstrap with mounted `/api/v1` routes: `API_tests/common.rs:60-133`.
  - Real Rocket HTTP client: `API_tests/common.rs:133`.
  - Real MySQL schema creation and seed setup: `API_tests/common.rs:84-108`, `API_tests/common.rs:370-574`.
  - Requests dispatched through Rocket local HTTP layer: repeated `.dispatch()` calls across `API_tests/*.rs`.

### HTTP With Mocking

- None found by static inspection.

### Non-HTTP (unit/integration without HTTP)

- Backend unit/service tests under `unit_tests/*.rs`.
- Frontend unit/component tests under `frontend/tests/*.rs`, `frontend/src/component_tests.rs`, `frontend/src/additional_tests.rs`, and inline tests in `frontend/src/main.rs:1324-1379`.
- Special case: `unit_tests/shared_guard_tests.rs` uses Rocket HTTP against a synthetic test-only route (`/whoami`), not a real application endpoint. It is useful for guard behavior, but it does not count toward endpoint coverage.

## Mock Detection

- `rg` scan for `jest.mock`, `vi.mock`, `sinon.stub`, `mockall`, `mockito`, `override`, `Mock` across `API_tests`, `unit_tests`, `frontend`, `app`, and `backend` returned no mocking constructs.
- No DI overrides or substituted controller/service states were found in endpoint-covering tests.
- Important qualification:
  - API tests do seed data directly through SQL helper functions in `API_tests/common.rs` and related helpers. That is setup convenience, not request-path mocking.

## API Test Mapping Table

| Endpoint | Covered | Test type | Test files | Evidence |
|---|---|---|---|---|
| `POST /api/v1/auth/login` | yes | true no-mock HTTP | `API_tests/auth_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `login_success_returns_tokens` (`auth_tests.rs:10`), `frontend_models_deserialize_from_live_backend_http_responses` (`fullstack_contract_tests.rs:10`) |
| `POST /api/v1/users` | yes | true no-mock HTTP | `API_tests/users_tests.rs` | `admin_can_create_user_and_record_is_persisted` (`users_tests.rs:9`) |
| `GET /api/v1/users` | yes | true no-mock HTTP | `API_tests/users_tests.rs` | `list_users_requires_auth` (`users_tests.rs:111`), `list_users_returns_seeded_users_for_admin` (`users_tests.rs:128`) |
| `PUT /api/v1/users/{id}` | yes | true no-mock HTTP | `API_tests/users_tests.rs` | `update_user_role_by_admin_persists_change` (`users_tests.rs:161`) |
| `DELETE /api/v1/users/{id}` | yes | true no-mock HTTP | `API_tests/users_tests.rs` | `delete_user_by_admin_removes_row` (`users_tests.rs:253`) |
| `POST /api/v1/candidates` | yes | true no-mock HTTP | `API_tests/crud_tests.rs`, `API_tests/workflow_tests.rs`, `API_tests/error_tests.rs` | `create_read_update_delete_candidate_with_auth` (`crud_tests.rs:12`) |
| `GET /api/v1/candidates` | yes | true no-mock HTTP | `API_tests/auth_tests.rs`, `API_tests/workflow_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `protected_without_token_returns_401` (`auth_tests.rs:64`), `reports_pagination_filtering_and_sorting_boundaries` (`workflow_tests.rs:88`) |
| `GET /api/v1/candidates/{id}` | yes | true no-mock HTTP | `API_tests/crud_tests.rs`, `API_tests/error_tests.rs` | `create_read_update_delete_candidate_with_auth` (`crud_tests.rs:12`), `not_found_resource_returns_404` (`error_tests.rs:55`) |
| `PUT /api/v1/candidates/{id}` | yes | true no-mock HTTP | `API_tests/crud_tests.rs` | `create_read_update_delete_candidate_with_auth` (`crud_tests.rs:12`) |
| `DELETE /api/v1/candidates/{id}` | yes | true no-mock HTTP | `API_tests/crud_tests.rs` | `create_read_update_delete_candidate_with_auth` (`crud_tests.rs:12`) |
| `POST /api/v1/candidates/merge` | yes | true no-mock HTTP | `API_tests/messages_merge_tests.rs` | `create_merge_candidate_persists_row_and_audits` (`messages_merge_tests.rs:62`) |
| `POST /api/v1/rooms` | yes | true no-mock HTTP | `API_tests/rooms_tests.rs` | `create_room_as_coordinator_persists_and_audits` (`rooms_tests.rs:17`) |
| `GET /api/v1/rooms` | yes | true no-mock HTTP | `API_tests/rooms_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `list_rooms_unauthenticated_returns_401` (`rooms_tests.rs:114`) |
| `PUT /api/v1/rooms/{id}` | yes | true no-mock HTTP | `API_tests/rooms_tests.rs` | `update_room_by_owner_persists_and_returns_200` (`rooms_tests.rs:186`) |
| `DELETE /api/v1/rooms/{id}` | yes | true no-mock HTTP | `API_tests/rooms_tests.rs` | `delete_room_by_owner_returns_204_and_removes_row` (`rooms_tests.rs:265`) |
| `POST /api/v1/sessions` | yes | true no-mock HTTP | `API_tests/sessions_tests.rs`, `API_tests/workflow_tests.rs` | `create_session_invalid_duration_returns_400` (`sessions_tests.rs:22`), `workflow_create_candidate_session_output_export` (`workflow_tests.rs:10`) |
| `GET /api/v1/sessions` | yes | true no-mock HTTP | `API_tests/sessions_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `list_sessions_coordinator_sees_only_own` (`sessions_tests.rs:81`) |
| `POST /api/v1/sessions/{id}/assignments` | yes | true no-mock HTTP | `API_tests/sessions_tests.rs`, `API_tests/output_tests.rs` | `assign_session_requires_proctor_user_as_assignee` (`sessions_tests.rs:240`) |
| `PUT /api/v1/sessions/{id}` | yes | true no-mock HTTP | `API_tests/sessions_tests.rs` | `update_session_by_coordinator_persists` (`sessions_tests.rs:146`) |
| `DELETE /api/v1/sessions/{id}` | yes | true no-mock HTTP | `API_tests/sessions_tests.rs` | `delete_session_by_owner_returns_204` (`sessions_tests.rs:190`) |
| `POST /api/v1/assets` | yes | true no-mock HTTP | `API_tests/assets_tests.rs` | `create_asset_by_coordinator_persists` (`assets_tests.rs:23`) |
| `GET /api/v1/assets` | yes | true no-mock HTTP | `API_tests/assets_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `list_assets_restricted_by_ownership` (`assets_tests.rs:121`) |
| `PUT /api/v1/assets/{id}` | yes | true no-mock HTTP | `API_tests/assets_tests.rs` | `update_asset_by_owner_changes_tracking_status` (`assets_tests.rs:167`) |
| `DELETE /api/v1/assets/{id}` | yes | true no-mock HTTP | `API_tests/assets_tests.rs` | `delete_asset_by_owner_returns_204` (`assets_tests.rs:221`) |
| `GET /api/v1/reports/dashboard` | yes | true no-mock HTTP | `API_tests/reports_tests.rs`, `API_tests/error_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `reports_dashboard_success_returns_counts_for_coordinator` (`reports_tests.rs:27`) |
| `GET /api/v1/dashboard/summary` | yes | true no-mock HTTP | `API_tests/reports_tests.rs` | `dashboard_summary_success_for_admin` (`reports_tests.rs:45`) |
| `GET /api/v1/operations/seat-utilization` | yes | true no-mock HTTP | `API_tests/workflow_tests.rs` | `reports_pagination_filtering_and_sorting_boundaries` (`workflow_tests.rs:88`) |
| `GET /api/v1/operations/near-expiry-alerts` | yes | true no-mock HTTP | `API_tests/reports_tests.rs` | `near_expiry_alerts_returns_assets_close_to_expiration` (`reports_tests.rs:76`) |
| `GET /api/v1/operations/incident-rates` | yes | true no-mock HTTP | `API_tests/reports_tests.rs`, `API_tests/error_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `incident_rates_returns_data_for_reporting_roles` (`reports_tests.rs:108`), `incident_rates_fallback_returns_array` (`reports_tests.rs:126`) |
| `GET /api/v1/operations/return-rates` | yes | true no-mock HTTP | `API_tests/reports_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `return_rates_success_for_coordinator` (`reports_tests.rs:181`) |
| `GET /api/v1/operations/materials-inventory` | yes | true no-mock HTTP | `API_tests/reports_tests.rs`, `API_tests/workflow_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `materials_inventory_returns_asset_rows` (`reports_tests.rs:220`) |
| `GET /api/v1/operations/alerts` | yes | true no-mock HTTP | `API_tests/reports_tests.rs`, `API_tests/workflow_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `operations_alerts_returns_ok_with_within_days` (`reports_tests.rs:239`) |
| `POST /api/v1/outputs` | yes | true no-mock HTTP | `API_tests/output_tests.rs`, `API_tests/workflow_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `final_print_allowed_for_proctor_and_coordinator` (`output_tests.rs:9`) |
| `GET /api/v1/outputs` | yes | true no-mock HTTP | `API_tests/outputs_list_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `list_outputs_returns_recent_outputs` (`outputs_list_tests.rs:144`), `list_outputs_fallback_without_query_still_returns_rows` (`outputs_list_tests.rs:172`) |
| `POST /api/v1/outputs/admit-cards` | yes | true no-mock HTTP | `API_tests/outputs_list_tests.rs` | `print_admit_cards_endpoint_produces_admit_card_output` (`outputs_list_tests.rs:18`) |
| `POST /api/v1/outputs/seating-charts` | yes | true no-mock HTTP | `API_tests/outputs_list_tests.rs` | `print_seating_charts_endpoint_returns_seating_chart` (`outputs_list_tests.rs:40`) |
| `POST /api/v1/outputs/door-signs` | yes | true no-mock HTTP | `API_tests/outputs_list_tests.rs` | `print_door_signs_endpoint_returns_door_sign` (`outputs_list_tests.rs:61`) |
| `POST /api/v1/outputs/proctor-packet` | yes | true no-mock HTTP | `API_tests/outputs_list_tests.rs` | `print_proctor_packet_endpoint_returns_proctor_packet` (`outputs_list_tests.rs:82`) |
| `POST /api/v1/outputs/summary-report` | yes | true no-mock HTTP | `API_tests/outputs_list_tests.rs` | `print_summary_report_endpoint_returns_summary_report` (`outputs_list_tests.rs:103`) |
| `POST /api/v1/messages/drafts` | yes | true no-mock HTTP | `API_tests/messages_merge_tests.rs` | `create_message_draft_persists_for_coordinator` (`messages_merge_tests.rs:9`) |
| `POST /api/v1/attachments` | yes | true no-mock HTTP | `API_tests/workflow_tests.rs`, `API_tests/error_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `attachment_upload_and_retrieval_roundtrip` (`workflow_tests.rs:389`) |
| `GET /api/v1/attachments` | yes | true no-mock HTTP | `API_tests/workflow_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `attachment_upload_and_retrieval_roundtrip` (`workflow_tests.rs:389`) |
| `GET /api/v1/attachments/{id}` | yes | true no-mock HTTP | `API_tests/workflow_tests.rs` | `attachment_upload_and_retrieval_roundtrip` (`workflow_tests.rs:389`) |
| `POST /api/v1/exports/csv` | yes | true no-mock HTTP | `API_tests/exports_tests.rs`, `API_tests/output_tests.rs`, `API_tests/workflow_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `export_csv_unsupported_report_returns_400` (`exports_tests.rs:61`) |
| `POST /api/v1/exports/excel` | yes | true no-mock HTTP | `API_tests/exports_tests.rs` | `export_excel_returns_tsv_with_expected_header` (`exports_tests.rs:9`) |
| `POST /api/v1/exports/pdf` | yes | true no-mock HTTP | `API_tests/exports_tests.rs` | `export_pdf_returns_placeholder_with_title` (`exports_tests.rs:37`) |
| `POST /api/v1/templates` | yes | true no-mock HTTP | `API_tests/templates_tests.rs` | `create_template_new_version_persists` (`templates_tests.rs:44`) |
| `GET /api/v1/templates` | yes | true no-mock HTTP | `API_tests/templates_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `list_templates_returns_seeded_and_requires_admin_or_coord` (`templates_tests.rs:9`) |
| `PUT /api/v1/templates/{template_id}/{version_no}` | yes | true no-mock HTTP | `API_tests/templates_tests.rs`, `API_tests/output_tests.rs` | `update_template_success_path_mutates_unlocked_version` (`templates_tests.rs:156`) |
| `DELETE /api/v1/templates/{template_id}/{version_no}` | yes | true no-mock HTTP | `API_tests/templates_tests.rs`, `API_tests/output_tests.rs` | `delete_template_success_removes_unlocked_version` (`templates_tests.rs:217`) |
| `POST /api/v1/templates/{template_id}/lock` | yes | true no-mock HTTP | `API_tests/templates_tests.rs` | `lock_template_creates_a_version` (`templates_tests.rs:110`) |
| `POST /api/v1/scans/lookup` | yes | true no-mock HTTP | `API_tests/workflow_tests.rs`, `API_tests/fullstack_contract_tests.rs` | `scan_asset_lookup_returns_asset_match` (`workflow_tests.rs:200`) |

## Coverage Summary

- Total unique endpoints: `52`
- Endpoints with HTTP tests: `52`
- Endpoints with true no-mock HTTP tests: `52`
- Endpoints with only mocked HTTP tests: `0`
- Uncovered endpoints: `0`
- HTTP coverage: `100%`
- True API coverage: `100%`

## Unit Test Summary

### Backend Unit Tests

Files found:

- `unit_tests/api_support_tests.rs`
- `unit_tests/audit_service_tests.rs`
- `unit_tests/auth_policy_tests.rs`
- `unit_tests/auth_service_tests.rs`
- `unit_tests/backend_bootstrap_tests.rs`
- `unit_tests/candidate_service_tests.rs`
- `unit_tests/cleansing_tests.rs`
- `unit_tests/cleansing_service_tests.rs`
- `unit_tests/dedupe_tests.rs`
- `unit_tests/dedupe_service_tests.rs`
- `unit_tests/encryption_tests.rs`
- `unit_tests/file_handling_service_tests.rs`
- `unit_tests/file_policy_tests.rs`
- `unit_tests/messaging_service_tests.rs`
- `unit_tests/output_service_tests.rs`
- `unit_tests/rbac_service_tests.rs`
- `unit_tests/reporting_service_tests.rs`
- `unit_tests/repository_tests.rs`
- `unit_tests/session_core_tests.rs`
- `unit_tests/shared_guard_tests.rs`
- `unit_tests/template_service_tests.rs`
- `unit_tests/template_validation_tests.rs`

Modules covered:

- Controllers/API support:
  - validators, API error construction, actor shaping in `unit_tests/api_support_tests.rs`
  - auth guard behavior in `unit_tests/shared_guard_tests.rs`
- Services:
  - `AuthService`, `CandidateService`, `CleansingService`, `MessagingService`, `OutputService`, `ReportingService`
- Security/auth:
  - password policy, RBAC matrix, session core helpers, encryption
- Domain/core:
  - dedupe logic, cleansing logic, template validation, template service
- Repository boundary:
  - only a minimal pool wrapper check in `unit_tests/repository_tests.rs`

Important backend modules not meaningfully unit-tested:

- API handlers in `app/api/v1/src/users.rs`, `candidates.rs`, `rooms.rs`, `sessions.rs`, `assets.rs`, `reports.rs`, `outputs.rs`, `templates.rs`, `exports.rs`, `scans.rs`
  - These are exercised through HTTP tests, but there is little handler-level unit isolation.
- File handling at persistence/integration boundary beyond pure validation paths.
- Export/report/output cross-service orchestration at deeper branch level outside endpoint tests.
- Database repository queries beyond the minimal cloneable-pool assertion.

### Frontend Unit Tests

Frontend unit tests: `PRESENT`

Files found:

- `frontend/tests/components_spec.rs`
- `frontend/tests/auth_spec.rs`
- `frontend/tests/helpers_spec.rs`
- `frontend/src/component_tests.rs`
- `frontend/src/additional_tests.rs`
- inline tests in `frontend/src/main.rs:1324-1379`

Frameworks/tools detected:

- Rust built-in test framework via `#[test]`
- Dioxus frontend modules imported/rendered from the real frontend crate (`frontend/Cargo.toml`, `frontend/tests/components_spec.rs`, `frontend/src/component_tests.rs`)

Components/modules covered:

- Dashboard rendering helpers: `dashboard_view`, `metric`, `spinner`
- Domain tables: candidates, rooms, sessions, assets, users, outputs, attachments, alerts, materials inventory, incidents, return rates, templates
- Auth/session helpers: `jwt_role`, `load_session`/storage-adjacent helpers, request/response models
- Routing/formatting helpers: route enum, date/time formatters, API base fallback, ID generation

Important frontend components/modules not tested:

- Page-level async flows in `frontend/src/main.rs`: `Login`, `Dashboard`, `candidates_page`, `rooms_page`, `proctors_page`, `exams_page`, `reports_page`, `templates_page`, `outputs_page`, `admin_page`
- Actual browser interaction and fetch behavior inside `get_json`, `post_empty`, `post_json`
- Router navigation behavior and form submission state transitions

Cross-layer observation:

- Testing is relatively balanced for a fullstack repo:
  - backend HTTP coverage is very strong
  - frontend unit coverage exists
  - browser E2E coverage also exists in `e2e/fullstack_e2e.js` and `e2e/role_matrix_e2e.js`
- The frontend tests are still helper/component heavy and do not cover the main async page flows with the same depth as backend HTTP coverage.

## API Observability Check

Verdict: `mostly strong`

Evidence:

- Requests are generally explicit and readable:
  - HTTP method and endpoint are visible in test bodies, for example `API_tests/users_tests.rs`, `API_tests/rooms_tests.rs`, `API_tests/workflow_tests.rs`.
- Request inputs are usually explicit JSON payloads, for example:
  - `admin_can_create_user_and_record_is_persisted` in `API_tests/users_tests.rs:9`
  - `create_room_as_coordinator_persists_and_audits` in `API_tests/rooms_tests.rs:17`
- Response content is usually asserted structurally, not only by status code:
  - examples across `API_tests/reports_tests.rs`, `API_tests/fullstack_contract_tests.rs`, `API_tests/workflow_tests.rs`

Weaknesses:

- Some tests emphasize status/result presence more than full response contract detail, especially permission and negative-path cases.
- A few workflow tests validate discoverability through follow-up queries rather than directly asserting the full creation response payload.

## Tests Check

- Success paths: strong
- Failure paths: strong
- Edge cases: moderate to strong
- Validation: strong
- Auth/permissions: strong
- Integration boundaries: strong for backend HTTP; moderate for frontend unit tests
- Over-mocking risk: low
- Superficial/autogenerated pattern risk: low to moderate
  - The tests are broad and purposeful, but several assertions are contract-shape checks rather than deep business invariant checks.

`run_tests.sh` assessment:

- Docker-based test orchestration: `OK`
  - `run_tests.sh:24-33` runs app stack, backend tests, frontend tests, and browser E2E in Docker/containers.
- Local dependency requirement: `not flagged`
  - The script requires Docker in PATH only (`run_tests.sh:7-9`), which is acceptable under the stated rule.

End-to-end expectation for fullstack:

- Present.
- Evidence:
  - browser E2E files `e2e/fullstack_e2e.js`, `e2e/role_matrix_e2e.js`
  - compose service `e2e-test` in `docker-compose.yml:86-108`

## Test Coverage Score

Score: `93/100`

## Score Rationale

- `+` Full endpoint inventory is covered by real HTTP tests.
- `+` No mocking was found in endpoint-covering API tests.
- `+` Backend unit coverage is broad across core logic and service layers.
- `+` Frontend unit tests are present with direct file-level evidence, which prevents a fullstack critical-gap finding.
- `+` Fullstack browser E2E coverage exists.
- `-` Frontend tests are mostly helper/component render checks; page-level async flows are under-tested.
- `-` Some tests assert response presence/status rather than deeper semantic invariants.
- `-` Repository/data-access unit coverage is thin outside HTTP suites.

## Key Gaps

- Frontend async page flows and request helpers in `frontend/src/main.rs` are not directly unit-tested.
- Handler-level unit tests are sparse; confidence relies heavily on HTTP suites.
- Repository/query-layer unit coverage is minimal.

## Confidence and Assumptions

- Confidence: `high`
- Assumptions:
  - Static inspection is sufficient to classify Rocket local client tests as HTTP tests reaching real handlers.
  - Duplicate Rocket route declarations with the same method and path are counted once per strict endpoint definition.
  - Coverage is based on visible tests only; no hidden/generated tests were assumed.

# README Audit

## README Location

- Present at [README.md](C:\Users\kidus\OneDrive\Desktop\ProctorOps-exam-main\repo\README.md).

## Hard Gate Check

| Gate | Result | Evidence |
|---|---|---|
| README exists at repo root | PASS | `README.md` present |
| Project type declared at top | PASS | `README.md:1` is `fullstack` |
| Clean markdown / readable structure | PARTIAL | structure is readable, but there are encoding artifacts at `README.md:51`, `README.md:82`, `README.md:89` |
| Backend/fullstack startup includes `docker-compose up` | PASS | `README.md:18-20` |
| Access method includes URL + port | PASS | `README.md:32-36` |
| Verification method explained | PASS | API verification at `README.md:51-80`; frontend flow at `README.md:82-91` |
| Docker-contained environment guidance | PASS | `README.md:9-12`, `README.md:95-101`; compose services for db/app/frontend/tests in `docker-compose.yml:1-108` |
| Demo credentials for all auth roles | PASS | `README.md:42-47`; role set aligns with seeded roles in `API_tests/common.rs:20-27`, `API_tests/common.rs:452-476` |

## High Priority Issues

- README test instructions are incomplete for a `fullstack` repo.
  - `README.md:97-101` documents only `docker-compose run --rm backend cargo test -p backend --tests`.
  - The repository defines separate Dockerized frontend and browser E2E test services in `docker-compose.yml:78-108`, and `run_tests.sh:26-33` executes all three layers.
  - Current wording overstates completeness: it claims the command executes "the full HTTP integration suite" but omits frontend and E2E coverage from the documented path.

- The MySQL access row is inaccurate.
  - README says `localhost:3306 (internal only)` at `README.md:36`.
  - Compose publishes MySQL to the host at `docker-compose.yml:11-12`.
  - This is a factual documentation defect.

## Medium Priority Issues

- README does not explain the architecture beyond a one-line stack summary.
  - There is no clear explanation of crate boundaries (`backend`, `frontend`, `app/core`, `app/api/v1`, `app/services`, `app/models`) even though the workspace is multi-crate.

- README does not describe role-based behavior beyond credentials.
  - The repo has explicit role-matrix coverage in tests (`unit_tests/rbac_service_tests.rs:6-23`, `e2e/role_matrix_e2e.js`), but the README does not summarize what each role can actually do.

- Verification instructions are useful but narrow.
  - The API smoke test verifies login plus `GET /users` only.
  - For a fullstack exam-operations platform, the verification section could better tie the documented UI flow to expected persisted backend state or reporting outcomes.

## Low Priority Issues

- Text encoding is broken in visible headings and arrows:
  - `README.md:51` uses `Option A â€”`
  - `README.md:82` uses `Option B â€”`
  - `README.md:89` uses `Reports â†’ Dashboard`

- README uses `docker-compose` spelling while `run_tests.sh` uses `docker compose`.
  - This is usually tolerable, but the docs would be cleaner if they were consistent.

## Engineering Quality

- Tech stack clarity: `moderate`
  - The top summary is clear.
  - Service/crate boundaries are not.

- Architecture explanation: `weak`
  - Minimal structure explanation.

- Testing instructions: `partial`
  - Docker-only is good.
  - Fullstack test surface is under-documented.

- Security/roles: `partial`
  - Demo credentials are present for all roles.
  - Role capabilities and security expectations are not explained.

- Workflows: `moderate`
  - Startup, access, verification, and stopping flows are present.

- Presentation quality: `moderate`
  - Overall readable, but encoding defects reduce polish.

## Hard Gate Failures

- None.

## README Verdict

Verdict: `PARTIAL PASS`

Rationale:

- All mandatory hard gates pass.
- The README is operationally usable.
- It does not meet a strong engineering-documentation standard for a fullstack repo because the test instructions are incomplete, the MySQL access statement is inaccurate, and the architecture/role explanations are thin.

# Final Verdicts

- Test Coverage Audit verdict: `PASS`
  - Strong result under strict static rules. All `52` unique API endpoints have true no-mock HTTP coverage, frontend unit tests are present, and fullstack E2E coverage exists.

- README Audit verdict: `PARTIAL PASS`
  - Hard gates pass, but accuracy and completeness defects remain.
