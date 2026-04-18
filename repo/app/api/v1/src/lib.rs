mod auth;
mod users;
mod candidates;
mod rooms;
mod sessions;
mod assets;
mod reports;
mod outputs;
mod exports;
mod templates;
mod scans;
mod errors;
mod shared;
mod pagination;
mod validators;
mod template_validation;

pub use auth::login;
pub use users::{create_user, delete_user, list_users, update_user};
pub use candidates::{create_candidate, delete_candidate, get_candidate, list_candidates, update_candidate, create_merge_candidate};
pub use rooms::{create_room, delete_room, list_rooms, update_room};
pub use sessions::{assign_session, create_session, delete_session, list_sessions, update_session};
pub use assets::{create_asset, delete_asset, list_assets, update_asset};
pub use reports::{dashboard_summary, incident_rates, incident_rates_fallback, materials_inventory, near_expiry_alerts, operations_alerts, reports_dashboard, return_rates, seat_utilization};
pub use outputs::{print_admit_cards, print_door_signs, print_seating_charts, print_proctor_packet, print_summary_report, create_message_draft, upload_attachment, list_attachments, get_attachment, generate_output, list_outputs, list_outputs_fallback};
pub use exports::{export_csv, export_excel, export_pdf};
pub use templates::{create_template, delete_template, list_templates, lock_template, update_template};
pub use scans::lookup_scan;
pub use template_validation::{validate_against_template, validate_against_template_partial};

// Narrow re-exports for downstream test targets and other crates that need to
// reference request-guard or validator types without depending on this crate's
// internal module layout. Keep this list tight — do not expose internal modules
// wholesale.
pub use errors::{ApiError, ApiErrorBody, ApiResult};
pub use shared::{parse_prompt_datetime, ApiContext};
pub use validators::{validate_room_capacity, validate_session_duration};

use rocket::{routes, Route};

pub fn routes_v1() -> Vec<Route> {
    routes![
        login,
        create_user,
        list_users,
        update_user,
        delete_user,
        create_candidate,
        list_candidates,
        get_candidate,
        update_candidate,
        delete_candidate,
        create_merge_candidate,
        create_room,
        list_rooms,
        update_room,
        delete_room,
        create_session,
        assign_session,
        list_sessions,
        update_session,
        delete_session,
        create_asset,
        list_assets,
        update_asset,
        delete_asset,
        reports_dashboard,
        dashboard_summary,
        seat_utilization,
        near_expiry_alerts,
        incident_rates,
        incident_rates_fallback,
        return_rates,
        materials_inventory,
        operations_alerts,
        print_admit_cards,
        print_seating_charts,
        print_door_signs,
        print_proctor_packet,
        print_summary_report,
        generate_output,
        list_outputs,
        list_outputs_fallback,
        upload_attachment,
        list_attachments,
        get_attachment,
        export_csv,
        export_excel,
        export_pdf,
        create_message_draft,
        create_template,
        list_templates,
        update_template,
        delete_template,
        lock_template,
        lookup_scan
    ]
}
