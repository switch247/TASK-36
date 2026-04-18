use base64::Engine;
use dioxus::prelude::*;
use dioxus_router::{use_navigator, Link, Routable, Router};
use gloo_net::http::Request;
use gloo_timers::future::sleep;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use wasm_bindgen_futures::spawn_local;

const API_BASE_DEFAULT: &str = "http://localhost:8000/api/v1";
const SESSION_STORAGE_KEY: &str = "proctorops_auth_session";

fn main() {
    dioxus::launch(App);
}

#[derive(Routable, Clone, PartialEq, Debug)]
enum Route {
    #[route("/")]
    Login {},
    #[route("/dashboard")]
    Dashboard {},
    #[route("/candidates")]
    Candidates {},
    #[route("/rooms")]
    Rooms {},
    #[route("/proctors")]
    Proctors {},
    #[route("/exams")]
    Exams {},
    #[route("/sessions")]
    Sessions {},
    #[route("/assets")]
    Assets {},
    #[route("/reports")]
    Reports {},
    #[route("/templates")]
    Templates {},
    #[route("/outputs")]
    Outputs {},
    #[route("/admin")]
    Admin {},
    #[route("/:.._route")]
    NotFound { _route: Vec<String> },
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct LoginResponse {
    pub session_id: String,
    pub jwt: String,
    pub session_expires_at: String,
    pub jwt_expires_at: String,
}
#[derive(Debug, Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}
#[derive(Clone, Copy)]
struct AuthCtx {
    session: Signal<Option<LoginResponse>>,
}
#[derive(Clone, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
    Info,
}
#[derive(Clone, PartialEq)]
struct Toast {
    id: i64,
    text: String,
    kind: ToastKind,
}
#[derive(Clone, Copy)]
struct ToastCtx {
    toast: Signal<Option<Toast>>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct DashboardSummary {
    pub total_candidates: i64,
    pub total_rooms: i64,
    pub total_sessions_this_week: i64,
    pub seat_utilization_count: usize,
    pub near_expiry_count: usize,
    pub incident_rate_count: usize,
    pub upcoming_sessions: Vec<UpcomingSession>,
    pub recent_outputs: Vec<RecentOutput>,
}
#[derive(Debug, Deserialize, Clone, Default)]
struct LegacyDashboardSummary {
    seat_utilization_count: usize,
    near_expiry_count: usize,
    incident_rate_count: usize,
}
#[derive(Debug, Deserialize, Clone)]
pub struct UpcomingSession {
    pub id: String,
    pub template_name: String,
    pub status: String,
    pub starts_at: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct RecentOutput {
    pub id: String,
    pub output_type: String,
    pub mode: String,
    pub created_at: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct OutputRow {
    pub id: String,
    pub session_id: String,
    pub output_type: String,
    pub mode: String,
    pub created_at: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct CandidateRow {
    pub id: String,
    pub scanned_barcode: String,
    pub national_id: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct RoomRow {
    pub id: String,
    pub capacity: i32,
    pub location: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct SessionRow {
    pub id: String,
    pub template_name: String,
    pub status: String,
    pub duration_minutes: i32,
    pub starts_at: Option<String>,
}
#[derive(Debug, Deserialize, Clone)]
pub struct AssetRow {
    pub id: String,
    pub booklet_code: String,
    pub tracking_status: String,
    pub incident_count: i32,
}
#[derive(Debug, Deserialize, Clone)]
pub struct IncidentRow {
    pub session_id: String,
    pub avg_incidents: f64,
}
#[derive(Debug, Deserialize, Clone)]
pub struct ReturnRateRow {
    pub session_id: String,
    pub total_assets: i64,
    pub returned_assets: i64,
    pub return_rate_pct: f64,
}
#[derive(Debug, Deserialize, Clone)]
pub struct MaterialInventoryRow {
    pub asset_id: String,
    pub booklet_code: String,
    pub tracking_status: String,
    pub session_id: String,
    pub expires_on: Option<String>,
    pub incident_count: i32,
}
#[derive(Debug, Deserialize, Clone)]
pub struct AlertRow {
    pub alert_type: String,
    pub severity: String,
    pub session_id: Option<String>,
    pub asset_id: Option<String>,
    pub message: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct UserRow {
    pub username: String,
    pub role: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct TemplateRow {
    pub template_id: String,
    pub version_no: i32,
    pub locked_for_final_print: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
struct CreateCandidateRequest {
    candidate_id: String,
    date_of_birth: String,
    national_id: String,
    scanned_barcode: String,
    metadata_json: String,
    template_id: Option<String>,
}
#[derive(Debug, Serialize)]
struct CreateRoomRequest {
    id: String,
    capacity: i32,
    location: String,
    template_id: Option<String>,
}
#[derive(Debug, Serialize)]
struct ScanReq {
    code: String,
    intent: String,
}
#[derive(Debug, Deserialize)]
pub struct ScanResp {
    pub code: String,
    pub found: bool,
    pub candidate_id: Option<String>,
    pub asset_id: Option<String>,
    pub asset_status: Option<String>,
    pub message: String,
}
#[derive(Debug, Serialize)]
struct CreateUserRequest {
    username: String,
    password: String,
    role: String,
    template_id: Option<String>,
}
#[derive(Debug, Serialize)]
struct CreateSessionRequest {
    id: String,
    template_name: String,
    duration_minutes: i32,
    status: String,
    starts_at: String,
    ends_at: String,
}
#[derive(Debug, Serialize)]
struct OutputReq {
    session_id: String,
    mode: String,
    output_type: String,
}
#[derive(Debug, Serialize)]
struct TemplateReq {
    template_id: String,
    version_no: i32,
    snapshot: serde_json::Value,
    lock_for_final_print: bool,
}
#[derive(Debug, Serialize)]
struct AttachmentUploadReq {
    record_type: String,
    record_id: String,
    file_name: String,
    extension: String,
    bytes_base64: String,
    operator_label: String,
    device_label: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct AttachmentRow {
    pub id: String,
    pub record_type: String,
    pub record_id: String,
    pub file_name: String,
    pub extension: String,
    pub size_bytes: i64,
    pub captured_at: String,
}
#[derive(Debug, Deserialize)]
struct AttachmentFileResp {
    file_name: String,
    bytes_base64: String,
}
#[derive(Debug, Serialize)]
struct ExportReportRequest {
    report: String,
    within_days: Option<i64>,
    filter: Option<String>,
    limit: Option<u32>,
}
#[derive(Debug, Deserialize)]
pub struct ExportResponse {
    pub content: String,
}
#[derive(Debug, Serialize)]
struct MessageDraftReq {
    channel: String,
    recipient: String,
    subject: Option<String>,
    body: String,
}

fn api_base() -> &'static str {
    option_env!("API_BASE").unwrap_or(API_BASE_DEFAULT)
}

fn gen_id(prefix: &str) -> String {
    format!("{prefix}-{}", uuid::Uuid::new_v4())
}

#[component]
fn App() -> Element {
    let session = use_signal(load_session);
    let toast = use_signal(|| None::<Toast>);
    use_context_provider(|| AuthCtx { session });
    use_context_provider(|| ToastCtx { toast });

    use_effect(move || {
        if let Some(t) = toast() {
            let mut sig = toast;
            spawn_local(async move {
                sleep(Duration::from_secs(5)).await;
                if sig().as_ref().map(|x| x.id) == Some(t.id) {
                    sig.set(None);
                }
            });
        }
    });

    rsx! {
        document::Stylesheet { href: "/tailwind.css" }
        Router::<Route> {}
        if let Some(t) = toast() {
            div { class: "fixed top-4 right-4 z-50 text-white px-4 py-2 rounded shadow-lg {toast_bg(&t.kind)}", "{t.text}" }
        }
    }
}

#[component]
fn Login() -> Element {
    let auth = use_context::<AuthCtx>();
    let toast_ctx = use_context::<ToastCtx>();
    let nav = use_navigator();
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    use_effect(move || {
        if (auth.session)().is_some() {
            nav.replace(Route::Dashboard {});
        }
    });
    rsx! {
        div { class: "min-h-screen bg-slate-100 flex items-center justify-center",
            div { class: "bg-white border rounded-xl p-6 w-full max-w-sm",
                h1 { class: "text-xl font-bold", "Login" }
                input { class: "mt-3 w-full border rounded px-3 py-2", placeholder: "Username", value: "{username}", oninput: move |e| username.set(e.value()) }
                input { r#type: "password", class: "mt-2 w-full border rounded px-3 py-2", placeholder: "Password", value: "{password}", oninput: move |e| password.set(e.value()) }
                button { class: "mt-3 w-full bg-blue-600 text-white rounded py-2", onclick: move |_| {
                    let payload = LoginRequest { username: username(), password: password() };
                    let mut ss = auth.session; let mut u = username; let mut p = password;
                    spawn_local(async move {
                        match post_json::<LoginRequest, LoginResponse>("/auth/login", &payload, None).await {
                            Ok(v) => { ss.set(Some(v.clone())); save_session(&v); push_toast(toast_ctx, "Login successful", ToastKind::Success); u.set(String::new()); p.set(String::new()); }
                            Err(e) => push_toast(toast_ctx, e, ToastKind::Error),
                        }
                    });
                }, "Sign In" }
            }
        }
    }
}
fn require_auth() -> Option<LoginResponse> {
    let auth = use_context::<AuthCtx>();
    let nav = use_navigator();
    use_effect(move || {
        if (auth.session)().is_none() {
            nav.push(Route::Login {});
        }
    });
    (auth.session)()
}

#[component]
fn Dashboard() -> Element {
    let Some(sess) = require_auth() else {
        return rsx! { div { class: "p-6", "Redirecting..." } };
    };
    let toast_ctx = use_context::<ToastCtx>();
    let mut loading = use_signal(|| true);
    let mut loaded = use_signal(|| false);
    let mut summary = use_signal(DashboardSummary::default);
    if !loaded() {
        loaded.set(true);
        let fetch_sess = sess.clone();
        let mut l = loading;
        let mut s = summary;
        spawn_local(async move {
            match get_json::<LegacyDashboardSummary>("/reports/dashboard", Some(&fetch_sess)).await
            {
                Ok(legacy) => {
                    let candidates = get_json::<Vec<CandidateRow>>(
                        "/candidates?page=1&limit=200",
                        Some(&fetch_sess),
                    )
                    .await
                    .unwrap_or_default();
                    let rooms =
                        get_json::<Vec<RoomRow>>("/rooms?page=1&limit=200", Some(&fetch_sess))
                            .await
                            .unwrap_or_default();
                    let sessions = get_json::<Vec<SessionRow>>(
                        "/sessions?page=1&limit=200",
                        Some(&fetch_sess),
                    )
                    .await
                    .unwrap_or_default();
                    let outputs =
                        get_json::<Vec<OutputRow>>("/outputs?page=1&limit=200", Some(&fetch_sess))
                            .await
                            .unwrap_or_default();

                    s.set(DashboardSummary {
                        total_candidates: candidates.len() as i64,
                        total_rooms: rooms.len() as i64,
                        total_sessions_this_week: sessions.len() as i64,
                        seat_utilization_count: legacy.seat_utilization_count,
                        near_expiry_count: legacy.near_expiry_count,
                        incident_rate_count: legacy.incident_rate_count,
                        upcoming_sessions: sessions
                            .iter()
                            .take(3)
                            .map(|x| UpcomingSession {
                                id: x.id.clone(),
                                template_name: x.template_name.clone(),
                                status: x.status.clone(),
                                starts_at: x.starts_at.clone().unwrap_or_else(|| "N/A".to_string()),
                            })
                            .collect(),
                        recent_outputs: outputs
                            .iter()
                            .take(5)
                            .map(|x| RecentOutput {
                                id: x.id.clone(),
                                output_type: x.output_type.clone(),
                                mode: x.mode.clone(),
                                created_at: x.created_at.clone(),
                            })
                            .collect(),
                    });
                }
                Err(_) => {
                    s.set(DashboardSummary::default());
                    push_toast(
                        toast_ctx,
                        "Dashboard has no report data yet",
                        ToastKind::Info,
                    );
                }
            }
            l.set(false);
        });
    }
    let content = if loading() {
        spinner()
    } else {
        dashboard_view(summary())
    };
    rsx! { Shell { title: "Dashboard", active: Route::Dashboard {}, {content} } }
}

pub fn dashboard_view(s: DashboardSummary) -> Element {
    let upcoming_sessions = s.upcoming_sessions.clone();
    let recent_outputs = s.recent_outputs.clone();
    let trend_line_points = trend_points(&s);
    rsx! {
        div { class: "grid sm:grid-cols-3 gap-3",
            {metric("Candidates", s.total_candidates.to_string())}
            {metric("Rooms", s.total_rooms.to_string())}
            {metric("Sessions This Week", s.total_sessions_this_week.to_string())}
        }
        div { class: "mt-3 grid sm:grid-cols-3 gap-3",
            {metric("Seat Utilization", s.seat_utilization_count.to_string())}
            {metric("Near Expiry", s.near_expiry_count.to_string())}
            {metric("Incident Rates", s.incident_rate_count.to_string())}
        }
        div { class: "mt-3 grid lg:grid-cols-2 gap-3",
            div { class: "bg-white border rounded p-3", h3 { class: "font-bold", "Upcoming Sessions" }, if upcoming_sessions.is_empty() { p { class: "text-slate-500 text-sm mt-2", "No data" } } else { for r in upcoming_sessions { p { class: "text-sm mt-1", "{r.id} | {r.template_name} | {r.status} | {format_display_datetime(&r.starts_at)}" } } } }
            div { class: "bg-white border rounded p-3", h3 { class: "font-bold", "Recent Outputs" }, if recent_outputs.is_empty() { p { class: "text-slate-500 text-sm mt-2", "No data" } } else { for r in recent_outputs { p { class: "text-sm mt-1", "{r.id} | {r.output_type} | {r.mode} | {format_display_datetime(&r.created_at)}" } } } }
        }
        div { class: "mt-3 bg-white border rounded p-3", h3 { class: "font-bold", "Seat Utilization Trend" }, svg { class: "w-full h-24", view_box: "0 0 400 100", polyline { points: "{trend_line_points}", fill: "none", stroke: "#2563eb", stroke_width: "3" } } }
    }
}

pub fn trend_points(s: &DashboardSummary) -> String {
    let values = vec![
        s.total_candidates.max(0) as f64,
        s.total_rooms.max(0) as f64,
        s.total_sessions_this_week.max(0) as f64,
        s.seat_utilization_count as f64,
        s.near_expiry_count as f64,
        s.incident_rate_count as f64,
    ];
    let max = values
        .iter()
        .cloned()
        .fold(1.0_f64, |acc, x| if x > acc { x } else { acc });
    let mut pts = Vec::new();
    for (idx, v) in values.iter().enumerate() {
        let x = (idx as f64) * (400.0 / (values.len().saturating_sub(1).max(1) as f64));
        let y = 90.0 - ((v / max) * 70.0);
        pts.push(format!("{:.0},{:.0}", x, y));
    }
    pts.join(" ")
}

#[component]
fn Candidates() -> Element {
    candidates_page()
}
#[component]
fn Rooms() -> Element {
    rooms_page()
}
#[component]
fn Proctors() -> Element {
    proctors_page()
}
#[component]
fn Exams() -> Element {
    exams_page()
}
#[component]
fn Sessions() -> Element {
    list_page_sessions()
}
#[component]
fn Assets() -> Element {
    list_page_assets()
}
#[component]
fn Reports() -> Element {
    reports_page()
}
#[component]
fn Templates() -> Element {
    templates_page()
}
#[component]
fn Outputs() -> Element {
    outputs_page()
}
#[component]
fn Admin() -> Element {
    admin_page()
}
#[component]
fn NotFound(_route: Vec<String>) -> Element {
    rsx! { div { class: "p-8", Link { to: Route::Login {}, "Back" } } }
}

fn candidates_page() -> Element {
    let Some(sess) = require_auth() else {
        return spinner();
    };
    let toast_ctx = use_context::<ToastCtx>();
    let mut loading = use_signal(|| true);
    let mut loaded = use_signal(|| false);
    let mut rows = use_signal(Vec::<CandidateRow>::new);
    let mut room_options = use_signal(Vec::<RoomRow>::new);
    let mut dob = use_signal(String::new);
    let mut nid = use_signal(String::new);
    let mut bar = use_signal(String::new);
    let mut name = use_signal(String::new);
    let mut room_id = use_signal(String::new);
    let mut template_id = use_signal(|| "candidate-registration".to_string());
    if !loaded() {
        loaded.set(true);
        let fetch_sess = sess.clone();
        let mut l = loading;
        let mut r = rows;
        let mut ro = room_options;
        spawn_local(async move {
            match get_json::<Vec<CandidateRow>>("/candidates?page=1&limit=50", Some(&fetch_sess))
                .await
            {
                Ok(v) => r.set(v),
                Err(e) => push_toast(toast_ctx, e, ToastKind::Error),
            }
            if let Ok(v) =
                get_json::<Vec<RoomRow>>("/rooms?page=1&limit=200", Some(&fetch_sess)).await
            {
                ro.set(v);
            }
            l.set(false);
        });
    }
    let list_content = if loading() {
        spinner()
    } else {
        table_candidates(rows())
    };
    rsx! { Shell { title: "Candidates", active: Route::Candidates {},
        div { class: "bg-white border rounded p-3",
            h3 { class: "font-bold", "Create Candidate" }
            input { class: "mt-2 border rounded px-3 py-2 mr-2", placeholder: "DOB (MM/DD/YYYY)", value: "{dob}", oninput: move |e| dob.set(e.value()) }
            input { class: "mt-2 border rounded px-3 py-2 mr-2", placeholder: "National ID", value: "{nid}", oninput: move |e| nid.set(e.value()) }
            input { class: "mt-2 border rounded px-3 py-2 mr-2", placeholder: "Barcode", value: "{bar}", oninput: move |e| bar.set(e.value()) }
            input { class: "mt-2 border rounded px-3 py-2 mr-2", placeholder: "Candidate Name", value: "{name}", oninput: move |e| name.set(e.value()) }
            input { class: "mt-2 border rounded px-3 py-2 mr-2", placeholder: "Template ID", value: "{template_id}", oninput: move |e| template_id.set(e.value()) }
            select { class: "mt-2 border rounded px-3 py-2 mr-2", value: "{room_id}", onchange: move |e| room_id.set(e.value()),
                option { value: "", "Select Room" }
                for r in room_options() {
                    option { value: "{r.id}", "{r.id} ({r.location})" }
                }
            }
            button { class: "mt-2 bg-blue-600 text-white rounded px-4 py-2", onclick: move |_| {
                let p = CreateCandidateRequest {
                    candidate_id: gen_id("cand"),
                    date_of_birth:dob(),
                    national_id:nid(),
                    scanned_barcode:bar(),
                    metadata_json: serde_json::json!({
                        "name": name(),
                        "room_id": room_id()
                    }).to_string(),
                    template_id: Some(template_id()),
                };
                if let Some(existing) = rows().into_iter().find(|r| r.national_id == p.national_id || r.scanned_barcode == p.scanned_barcode) {
                    let should_merge = web_sys::window()
                        .and_then(|w| w.confirm_with_message("Potential duplicate found. Merge with existing record?").ok())
                        .unwrap_or(false);
                    if should_merge {
                        let merge_payload = serde_json::json!({
                            "left_candidate_id": existing.id,
                            "right_candidate_id": p.candidate_id,
                            "similarity_score": 0.95
                        });
                        let s = sess.clone();
                        spawn_local(async move {
                            match post_empty("/candidates/merge", &merge_payload, Some(&s)).await {
                                Ok(_) => push_toast(toast_ctx, "Duplicate merge flow submitted", ToastKind::Success),
                                Err(e) => push_toast(toast_ctx, e, ToastKind::Error),
                            }
                        });
                        return;
                    }
                }
                let s = sess.clone(); let mut b=dob; let mut c=nid; let mut d=bar; let mut e=name; let mut f=room_id; let mut t=template_id; let mut rws = rows;
                spawn_local(async move {
                    match post_empty("/candidates", &p, Some(&s)).await {
                        Ok(_)=>{
                            push_toast(toast_ctx, "Candidate created", ToastKind::Success);
                            b.set(String::new()); c.set(String::new()); d.set(String::new()); e.set(String::new()); f.set(String::new()); t.set("candidate-registration".to_string());
                            if let Ok(updated) = get_json::<Vec<CandidateRow>>("/candidates?page=1&limit=50", Some(&s)).await {
                                rws.set(updated);
                            }
                        },
                        Err(err)=>push_toast(toast_ctx, err, ToastKind::Error)
                    }
                });
            }, "Create" }
        }
        {list_content}
    } }
}

fn rooms_page() -> Element {
    let Some(sess) = require_auth() else {
        return spinner();
    };
    let toast_ctx = use_context::<ToastCtx>();
    let mut loading = use_signal(|| true);
    let mut loaded = use_signal(|| false);
    let mut rows = use_signal(Vec::<RoomRow>::new);
    let mut cap = use_signal(String::new);
    let mut loc = use_signal(String::new);
    let mut template_id = use_signal(|| "room-config".to_string());
    if !loaded() {
        loaded.set(true);
        let fetch_sess = sess.clone();
        let mut l = loading;
        let mut r = rows;
        spawn_local(async move {
            match get_json::<Vec<RoomRow>>("/rooms?page=1&limit=50", Some(&fetch_sess)).await {
                Ok(v) => r.set(v),
                Err(e) => push_toast(toast_ctx, e, ToastKind::Error),
            }
            l.set(false);
        });
    }
    let list_content = if loading() {
        spinner()
    } else {
        table_rooms(rows())
    };
    rsx! { Shell { title: "Rooms", active: Route::Rooms {},
        div { class: "bg-white border rounded p-3",
            h3 { class: "font-bold", "Create Room" }
            input { class: "mt-2 border rounded px-3 py-2 mr-2", placeholder: "Capacity", value: "{cap}", oninput: move |e| cap.set(e.value()) }
            input { class: "mt-2 border rounded px-3 py-2 mr-2", placeholder: "Location", value: "{loc}", oninput: move |e| loc.set(e.value()) }
            input { class: "mt-2 border rounded px-3 py-2 mr-2", placeholder: "Template ID", value: "{template_id}", oninput: move |e| template_id.set(e.value()) }
            button { class: "mt-2 bg-blue-600 text-white rounded px-4 py-2", onclick: move |_| {
                let p = CreateRoomRequest { id:gen_id("room"), capacity:cap().parse().unwrap_or(0), location:loc(), template_id: Some(template_id()) }; let s = sess.clone(); let mut b=cap; let mut c=loc; let mut t=template_id; let mut rws=rows;
                spawn_local(async move {
                    match post_empty("/rooms", &p, Some(&s)).await {
                        Ok(_)=>{
                            push_toast(toast_ctx, "Room created", ToastKind::Success);
                            b.set(String::new()); c.set(String::new()); t.set("room-config".to_string());
                            if let Ok(updated) = get_json::<Vec<RoomRow>>("/rooms?page=1&limit=50", Some(&s)).await {
                                rws.set(updated);
                            }
                        },
                        Err(err)=>push_toast(toast_ctx, err, ToastKind::Error)
                    }
                });
            }, "Create" }
        }
        {list_content}
    } }
}

fn proctors_page() -> Element {
    let Some(sess) = require_auth() else {
        return spinner();
    };
    let toast_ctx = use_context::<ToastCtx>();
    let mut loading = use_signal(|| true);
    let mut loaded = use_signal(|| false);
    let mut users = use_signal(Vec::<UserRow>::new);
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    if !loaded() {
        loaded.set(true);
        let fetch_sess = sess.clone();
        let mut l = loading;
        let mut u = users;
        spawn_local(async move {
            match get_json::<Vec<UserRow>>("/users", Some(&fetch_sess)).await {
                Ok(v) => u.set(v.into_iter().filter(|x| x.role == "Proctor").collect()),
                Err(e) => push_toast(toast_ctx, e, ToastKind::Error),
            }
            l.set(false);
        });
    }
    let list_content = if loading() {
        spinner()
    } else {
        table_users(users())
    };
    rsx! { Shell { title: "Proctors", active: Route::Proctors {},
        div { class: "bg-white border rounded p-3",
            h3 { class: "font-bold", "Create Proctor (Template: proctor-profile)" }
            input { class: "mt-2 border rounded px-3 py-2 mr-2", placeholder: "Username", value: "{username}", oninput: move |e| username.set(e.value()) }
            input { r#type: "password", class: "mt-2 border rounded px-3 py-2 mr-2", placeholder: "Password", value: "{password}", oninput: move |e| password.set(e.value()) }
            button { class: "mt-2 bg-blue-600 text-white rounded px-4 py-2", onclick: move |_| {
                let req = CreateUserRequest { username: username(), password: password(), role: "Proctor".to_string(), template_id: Some("proctor-profile".to_string()) };
                let mut un = username;
                let mut pw = password;
                let mut rows_sig = users;
                let s = sess.clone();
                spawn_local(async move {
                    match post_empty("/users", &req, Some(&s)).await {
                        Ok(_) => {
                            push_toast(toast_ctx, "Proctor created", ToastKind::Success);
                            un.set(String::new());
                            pw.set(String::new());
                            if let Ok(v) = get_json::<Vec<UserRow>>("/users", Some(&s)).await {
                                rows_sig.set(v.into_iter().filter(|x| x.role == "Proctor").collect());
                            }
                        }
                        Err(e) => push_toast(toast_ctx, e, ToastKind::Error),
                    }
                });
            }, "Create Proctor" }
        }
        {list_content}
    } }
}

fn exams_page() -> Element {
    let Some(sess) = require_auth() else {
        return spinner();
    };
    let toast_ctx = use_context::<ToastCtx>();
    let mut loading = use_signal(|| true);
    let mut loaded = use_signal(|| false);
    let mut rows = use_signal(Vec::<SessionRow>::new);
    let mut template_name = use_signal(|| "base-template".to_string());
    let mut duration = use_signal(|| "90".to_string());
    let mut starts_at = use_signal(|| "04/10/2026 09:00 AM".to_string());
    let mut ends_at = use_signal(|| "04/10/2026 10:30 AM".to_string());
    if !loaded() {
        loaded.set(true);
        let fetch_sess = sess.clone();
        let mut l = loading;
        let mut r = rows;
        spawn_local(async move {
            match get_json::<Vec<SessionRow>>("/sessions?page=1&limit=50", Some(&fetch_sess)).await
            {
                Ok(v) => r.set(v),
                Err(e) => push_toast(toast_ctx, e, ToastKind::Error),
            }
            l.set(false);
        });
    }
    let list_content = if loading() {
        spinner()
    } else {
        table_sessions(rows())
    };
    rsx! { Shell { title: "Exams", active: Route::Exams {},
        div { class: "bg-white border rounded p-3",
            h3 { class: "font-bold", "Create Exam Session (Template-driven)" }
            input { class: "mt-2 border rounded px-3 py-2 mr-2", placeholder: "Template Name", value: "{template_name}", oninput: move |e| template_name.set(e.value()) }
            input { class: "mt-2 border rounded px-3 py-2 mr-2", placeholder: "Duration Minutes", value: "{duration}", oninput: move |e| duration.set(e.value()) }
            input { class: "mt-2 border rounded px-3 py-2 mr-2", placeholder: "Starts (MM/DD/YYYY hh:mm AM/PM)", value: "{starts_at}", oninput: move |e| starts_at.set(e.value()) }
            input { class: "mt-2 border rounded px-3 py-2 mr-2", placeholder: "Ends (MM/DD/YYYY hh:mm AM/PM)", value: "{ends_at}", oninput: move |e| ends_at.set(e.value()) }
            button { class: "mt-2 bg-emerald-600 text-white rounded px-4 py-2", onclick: move |_| {
                let req = CreateSessionRequest {
                    id: gen_id("exam"),
                    template_name: template_name(),
                    duration_minutes: duration().parse().unwrap_or(90),
                    status: "Scheduled".to_string(),
                    starts_at: starts_at(),
                    ends_at: ends_at(),
                };
                let s = sess.clone();
                let mut rows_sig = rows;
                spawn_local(async move {
                    match post_empty("/sessions", &req, Some(&s)).await {
                        Ok(_) => {
                            push_toast(toast_ctx, "Exam session created", ToastKind::Success);
                            if let Ok(v) = get_json::<Vec<SessionRow>>("/sessions?page=1&limit=50", Some(&s)).await {
                                rows_sig.set(v);
                            }
                        }
                        Err(e) => push_toast(toast_ctx, e, ToastKind::Error),
                    }
                });
            }, "Create Exam Session" }
        }
        {list_content}
    } }
}
fn list_page_sessions() -> Element {
    let Some(sess) = require_auth() else {
        return spinner();
    };
    let toast_ctx = use_context::<ToastCtx>();
    let mut loading = use_signal(|| true);
    let mut loaded = use_signal(|| false);
    let mut rows = use_signal(Vec::<SessionRow>::new);
    let mut code = use_signal(String::new);
    let mut scan_intent = use_signal(|| "candidate_lookup".to_string());
    if !loaded() {
        loaded.set(true);
        let fetch_sess = sess.clone();
        let mut l = loading;
        let mut r = rows;
        spawn_local(async move {
            match get_json::<Vec<SessionRow>>("/sessions?page=1&limit=50", Some(&fetch_sess)).await
            {
                Ok(v) => r.set(v),
                Err(e) => push_toast(toast_ctx, e, ToastKind::Error),
            }
            l.set(false);
        });
    }
    let list_content = if loading() {
        spinner()
    } else {
        table_sessions(rows())
    };
    rsx! { Shell { title: "Sessions", active: Route::Sessions {}, {list_content}
        div { class: "mt-3 bg-white border rounded p-3", h3 { class: "font-bold", "QR / Barcode Capture" }
            input { class: "mt-2 border rounded px-3 py-2 mr-2", placeholder: "Scan or enter code", value: "{code}", oninput: move |e| code.set(e.value()) }
            select { class: "mt-2 border rounded px-3 py-2 mr-2", value: "{scan_intent}", onchange: move |e| scan_intent.set(e.value()),
                option { value: "candidate_lookup", "Candidate lookup" }
                option { value: "asset_lookup", "Asset / booklet lookup" }
            }
            button { class: "bg-slate-700 text-white rounded px-4 py-2 mr-2", onclick: move |_| code.set(format!("SIM-{}", js_sys::Date::now() as i64)), "Simulate" }
            button { class: "bg-emerald-600 text-white rounded px-4 py-2", onclick: move |_| { let p = ScanReq { code: code(), intent: scan_intent() }; let s = sess.clone(); spawn_local(async move { match post_json::<ScanReq, ScanResp>("/scans/lookup", &p, Some(&s)).await { Ok(v)=>{ if v.found { if let Some(cid) = v.candidate_id { push_toast(toast_ctx, format!("Valid candidate: {}", cid), ToastKind::Success) } else if let Some(aid) = v.asset_id { push_toast(toast_ctx, format!("Valid asset: {} ({})", aid, v.asset_status.unwrap_or_else(|| "unknown".to_string())), ToastKind::Success) } else { push_toast(toast_ctx, v.message, ToastKind::Info) } } else { push_toast(toast_ctx, v.message, ToastKind::Error) } }, Err(e)=>push_toast(toast_ctx, e, ToastKind::Error)} }); }, "Submit" }
        }
    } }
}

fn list_page_assets() -> Element {
    let Some(sess) = require_auth() else {
        return spinner();
    };
    let toast_ctx = use_context::<ToastCtx>();
    let mut loading = use_signal(|| true);
    let mut loaded = use_signal(|| false);
    let mut rows = use_signal(Vec::<AssetRow>::new);
    if !loaded() {
        loaded.set(true);
        let fetch_sess = sess.clone();
        let mut l = loading;
        let mut r = rows;
        spawn_local(async move {
            match get_json::<Vec<AssetRow>>("/assets?page=1&limit=50", Some(&fetch_sess)).await {
                Ok(v) => r.set(v),
                Err(e) => push_toast(toast_ctx, e, ToastKind::Error),
            }
            l.set(false);
        });
    }
    let list_content = if loading() {
        spinner()
    } else {
        table_assets(rows())
    };
    rsx! { Shell { title: "Assets", active: Route::Assets {}, {list_content} } }
}

fn reports_page() -> Element {
    let Some(sess) = require_auth() else {
        return spinner();
    };
    let toast_ctx = use_context::<ToastCtx>();
    let mut loading = use_signal(|| true);
    let mut loaded = use_signal(|| false);
    let mut incident_rows = use_signal(Vec::<IncidentRow>::new);
    let mut return_rows = use_signal(Vec::<ReturnRateRow>::new);
    let mut inventory_rows = use_signal(Vec::<MaterialInventoryRow>::new);
    let mut alert_rows = use_signal(Vec::<AlertRow>::new);
    let mut draft_recipient = use_signal(String::new);
    let mut draft_body = use_signal(String::new);
    if !loaded() {
        loaded.set(true);
        let fetch_sess = sess.clone();
        let mut l = loading;
        let mut ir = incident_rows;
        let mut rr = return_rows;
        let mut mr = inventory_rows;
        let mut ar = alert_rows;
        spawn_local(async move {
            match get_json::<Vec<IncidentRow>>("/operations/incident-rates", Some(&fetch_sess))
                .await
            {
                Ok(v) => ir.set(v),
                Err(_) => ir.set(Vec::new()),
            }
            match get_json::<Vec<ReturnRateRow>>("/operations/return-rates", Some(&fetch_sess))
                .await
            {
                Ok(v) => rr.set(v),
                Err(_) => rr.set(Vec::new()),
            }
            match get_json::<Vec<MaterialInventoryRow>>(
                "/operations/materials-inventory?page=1&limit=200",
                Some(&fetch_sess),
            )
            .await
            {
                Ok(v) => mr.set(v),
                Err(_) => mr.set(Vec::new()),
            }
            match get_json::<Vec<AlertRow>>(
                "/operations/alerts?within_days=30&page=1&limit=200",
                Some(&fetch_sess),
            )
            .await
            {
                Ok(v) => ar.set(v),
                Err(_) => ar.set(Vec::new()),
            }
            l.set(false);
        });
    }
    let export_csv_sess = sess.clone();
    let export_inventory_sess = sess.clone();
    let export_excel_sess = sess.clone();
    let export_pdf_sess = sess.clone();
    let draft_sess = sess.clone();
    let content = if loading() {
        spinner()
    } else {
        rsx! {
            div { class: "bg-white border rounded p-3",
                h3 { class: "font-bold", "Server-Generated Exports" }
                p { class: "text-sm text-slate-600 mt-1", "Exports are generated from backend report queries (not client row payloads)." }
                button { class: "mt-2 bg-blue-600 text-white rounded px-4 py-2 mr-2", onclick: move |_| {
                    let s = export_csv_sess.clone();
                    spawn_local(async move {
                        let payload = ExportReportRequest {
                            report: "incident_rates".to_string(),
                            within_days: None,
                            filter: None,
                            limit: Some(200),
                        };
                        match post_json::<ExportReportRequest, ExportResponse>("/exports/csv", &payload, Some(&s)).await {
                            Ok(res) => {
                                let preview = res.content.lines().next().unwrap_or("CSV generated");
                                push_toast(toast_ctx, format!("CSV ready: {preview}"), ToastKind::Success);
                            }
                            Err(e) => push_toast(toast_ctx, e, ToastKind::Error),
                        }
                    });
                }, "Export Incident CSV" }
                button { class: "mt-2 bg-emerald-600 text-white rounded px-4 py-2 mr-2", onclick: move |_| {
                    let s = export_excel_sess.clone();
                    spawn_local(async move {
                        let payload = ExportReportRequest {
                            report: "return_rates".to_string(),
                            within_days: None,
                            filter: None,
                            limit: Some(200),
                        };
                        match post_json::<ExportReportRequest, ExportResponse>("/exports/excel", &payload, Some(&s)).await {
                            Ok(_) => push_toast(toast_ctx, "Excel-style export generated", ToastKind::Success),
                            Err(e) => push_toast(toast_ctx, e, ToastKind::Error),
                        }
                    });
                }, "Export Return Excel" }
                button { class: "mt-2 bg-slate-700 text-white rounded px-4 py-2", onclick: move |_| {
                    let s = export_pdf_sess.clone();
                    spawn_local(async move {
                        let payload = ExportReportRequest {
                            report: "near_expiry_assets".to_string(),
                            within_days: Some(30),
                            filter: None,
                            limit: Some(200),
                        };
                        match post_json::<ExportReportRequest, ExportResponse>("/exports/pdf", &payload, Some(&s)).await {
                            Ok(_) => push_toast(toast_ctx, "PDF export generated", ToastKind::Success),
                            Err(e) => push_toast(toast_ctx, e, ToastKind::Error),
                        }
                    });
                }, "Export Near-Expiry PDF" }
                button { class: "mt-2 bg-amber-600 text-white rounded px-4 py-2 ml-2", onclick: move |_| {
                    let s = export_inventory_sess.clone();
                    spawn_local(async move {
                        let payload = ExportReportRequest {
                            report: "materials_inventory".to_string(),
                            within_days: None,
                            filter: None,
                            limit: Some(200),
                        };
                        match post_json::<ExportReportRequest, ExportResponse>("/exports/csv", &payload, Some(&s)).await {
                            Ok(_) => push_toast(toast_ctx, "Materials inventory CSV generated", ToastKind::Success),
                            Err(e) => push_toast(toast_ctx, e, ToastKind::Error),
                        }
                    });
                }, "Export Inventory CSV" }
            }
            div { class: "mt-3 bg-white border rounded p-3",
                h3 { class: "font-bold", "Message Drafts" }
                input { class: "mt-2 border rounded px-3 py-2 mr-2", placeholder: "Recipient", value: "{draft_recipient}", oninput: move |e| draft_recipient.set(e.value()) }
                input { class: "mt-2 border rounded px-3 py-2 mr-2", placeholder: "Message body", value: "{draft_body}", oninput: move |e| draft_body.set(e.value()) }
                button { class: "mt-2 bg-indigo-600 text-white rounded px-4 py-2", onclick: move |_| {
                    let payload = MessageDraftReq {
                        channel: "Email".to_string(),
                        recipient: draft_recipient(),
                        subject: Some("Operations Alert".to_string()),
                        body: draft_body(),
                    };
                    let mut recipient = draft_recipient;
                    let mut body = draft_body;
                    let s = draft_sess.clone();
                    spawn_local(async move {
                        match post_empty("/messages/drafts", &payload, Some(&s)).await {
                            Ok(_) => {
                                push_toast(toast_ctx, "Message draft created", ToastKind::Success);
                                recipient.set(String::new());
                                body.set(String::new());
                            }
                            Err(e) => push_toast(toast_ctx, e, ToastKind::Error),
                        }
                    });
                }, "Create Draft" }
            }
            {table_incidents(incident_rows())}
            {table_return_rates(return_rows())}
            {table_materials_inventory(inventory_rows())}
            {table_alerts(alert_rows())}
        }
    };
    rsx! { Shell { title: "Reports", active: Route::Reports {}, {content} } }
}

fn templates_page() -> Element {
    let Some(sess) = require_auth() else {
        return spinner();
    };
    let toast_ctx = use_context::<ToastCtx>();
    let mut loading = use_signal(|| true);
    let mut loaded = use_signal(|| false);
    let mut rows = use_signal(Vec::<TemplateRow>::new);
    let mut template_id = use_signal(|| "candidate-registration".to_string());
    let mut ver = use_signal(String::new);
    let mut snapshot_json = use_signal(|| {
        "{\"rules\":{\"date_of_birth\":[\"Required\"],\"national_id\":[\"Required\"],\"scanned_barcode\":[\"Required\"],\"name\":[\"Required\"]}}".to_string()
    });
    if !loaded() {
        loaded.set(true);
        let fetch_sess = sess.clone();
        let mut l = loading;
        let mut r = rows;
        spawn_local(async move {
            match get_json::<Vec<TemplateRow>>("/templates", Some(&fetch_sess)).await {
                Ok(v) => r.set(v),
                Err(e) => push_toast(toast_ctx, e, ToastKind::Error),
            }
            l.set(false);
        });
    }
    let list_content = if loading() {
        spinner()
    } else {
        table_templates(rows())
    };
    rsx! { Shell { title: "Templates", active: Route::Templates {},
        div { class: "bg-white border rounded p-3",
            input { class: "border rounded px-3 py-2 mr-2", placeholder: "Template ID", value: "{template_id}", oninput: move |e| template_id.set(e.value()) }
            input { class: "border rounded px-3 py-2 mr-2", placeholder: "Version", value: "{ver}", oninput: move |e| ver.set(e.value()) }
            textarea { class: "mt-2 w-full border rounded px-3 py-2", rows: "4", placeholder: "Snapshot JSON", value: "{snapshot_json}", oninput: move |e| snapshot_json.set(e.value()) }
            button { class: "bg-blue-600 text-white rounded px-4 py-2", onclick: move |_| {
                let snapshot = match serde_json::from_str::<serde_json::Value>(&snapshot_json()) {
                    Ok(v) => v,
                    Err(_) => {
                        push_toast(toast_ctx, "Snapshot JSON is invalid", ToastKind::Error);
                        return;
                    }
                };
                let p = TemplateReq { template_id: template_id(), version_no: ver().parse().unwrap_or(1), snapshot, lock_for_final_print: false };
                let s = sess.clone(); let mut a=template_id; let mut b=ver; let mut c=snapshot_json; let mut rws=rows;
                spawn_local(async move {
                    match post_empty("/templates", &p, Some(&s)).await {
                        Ok(_)=>{
                            push_toast(toast_ctx, "Template saved", ToastKind::Success);
                            a.set("candidate-registration".to_string());
                            b.set(String::new());
                            c.set("{\"rules\":{\"date_of_birth\":[\"Required\"],\"national_id\":[\"Required\"],\"scanned_barcode\":[\"Required\"],\"name\":[\"Required\"]}}".to_string());
                            if let Ok(updated) = get_json::<Vec<TemplateRow>>("/templates", Some(&s)).await {
                                rws.set(updated);
                            }
                        },
                        Err(e)=>push_toast(toast_ctx, e, ToastKind::Error)
                    }
                });
            }, "Save" }
        }
        {list_content}
    } }
}

fn outputs_page() -> Element {
    let Some(sess) = require_auth() else {
        return spinner();
    };
    let toast_ctx = use_context::<ToastCtx>();
    let mut loading = use_signal(|| true);
    let mut loaded = use_signal(|| false);
    let mut rows = use_signal(Vec::<OutputRow>::new);
    let mut attachments = use_signal(Vec::<AttachmentRow>::new);
    let mut session_options = use_signal(Vec::<SessionRow>::new);
    let mut sid = use_signal(String::new);
    let mut otype = use_signal(String::new);
    let mut mode = use_signal(String::new);
    let mut rec_type = use_signal(|| "candidate".to_string());
    let mut rec_id = use_signal(String::new);
    let mut file_name = use_signal(String::new);
    let mut extension = use_signal(|| "pdf".to_string());
    let mut bytes_b64 = use_signal(String::new);
    if !loaded() {
        loaded.set(true);
        let fetch_sess = sess.clone();
        let mut l = loading;
        let mut r = rows;
        let mut at = attachments;
        let mut so = session_options;
        spawn_local(async move {
            match get_json::<Vec<OutputRow>>("/outputs", Some(&fetch_sess)).await {
                Ok(v) => r.set(v),
                Err(_) => r.set(Vec::new()),
            }
            if let Ok(v) = get_json::<Vec<AttachmentRow>>("/attachments", Some(&fetch_sess)).await {
                at.set(v);
            }
            if let Ok(v) =
                get_json::<Vec<SessionRow>>("/sessions?page=1&limit=200", Some(&fetch_sess)).await
            {
                so.set(v);
            }
            l.set(false);
        });
    }
    let list_content = if loading() {
        spinner()
    } else {
        table_outputs(rows())
    };
    let sess_for_output = sess.clone();
    let sess_for_attachment = sess.clone();
    rsx! { Shell { title: "Outputs", active: Route::Outputs {},
        div { class: "bg-white border rounded p-3",
            select { class: "border rounded px-3 py-2 mr-2", value: "{sid}", onchange: move |e| sid.set(e.value()),
                option { value: "", "Select Session ID" }
                for srow in session_options() {
                    option { value: "{srow.id}", "{srow.id}" }
                }
            }
            select { class: "border rounded px-3 py-2 mr-2", value: "{otype}", onchange: move |e| otype.set(e.value()),
                option { value: "", "Select Output Type" }
                option { value: "AdmitCard", "Admit Card" }
                option { value: "SeatingChart", "Seating Chart" }
                option { value: "DoorSign", "Door Sign" }
                option { value: "ProctorPacket", "Proctor Packet" }
                option { value: "SummaryReport", "Summary Report" }
            }
            select { class: "border rounded px-3 py-2 mr-2", value: "{mode}", onchange: move |e| mode.set(e.value()),
                option { value: "", "Select Mode" }
                option { value: "TestPrint", "TestPrint" }
                option { value: "FinalPrint", "FinalPrint" }
            }
            button { class: "bg-blue-600 text-white rounded px-4 py-2", onclick: move |_| {
                let p = OutputReq { session_id: sid(), output_type: otype(), mode: mode() }; let s = sess_for_output.clone(); let mut a=sid; let mut b=otype; let mut c=mode; let mut rws=rows;
                spawn_local(async move {
                    match post_json::<OutputReq, serde_json::Value>("/outputs", &p, Some(&s)).await {
                        Ok(_)=>{
                            push_toast(toast_ctx, "Output generated", ToastKind::Success);
                            a.set(String::new()); b.set(String::new()); c.set(String::new());
                            if let Ok(updated) = get_json::<Vec<OutputRow>>("/outputs", Some(&s)).await {
                                rws.set(updated);
                            }
                        },
                        Err(e)=>push_toast(toast_ctx, e, ToastKind::Error)
                    }
                });
            }, "Generate Output" }
        }
        div { class: "mt-3 bg-white border rounded p-3",
            h3 { class: "font-bold mb-2", "Attachment Capture" }
            select { class: "border rounded px-3 py-2 mr-2", value: "{rec_type}", onchange: move |e| rec_type.set(e.value()),
                option { value: "candidate", "candidate" }
                option { value: "room", "room" }
                option { value: "session", "session" }
                option { value: "asset", "asset" }
            }
            input { class: "border rounded px-3 py-2 mr-2", placeholder: "Record ID", value: "{rec_id}", oninput: move |e| rec_id.set(e.value()) }
            input { class: "border rounded px-3 py-2 mr-2", placeholder: "File name", value: "{file_name}", oninput: move |e| file_name.set(e.value()) }
            input { class: "border rounded px-3 py-2 mr-2", placeholder: "Ext (pdf/jpg...)", value: "{extension}", oninput: move |e| extension.set(e.value()) }
            textarea { class: "mt-2 w-full border rounded px-3 py-2", rows: "3", placeholder: "Paste file bytes as base64", value: "{bytes_b64}", oninput: move |e| bytes_b64.set(e.value()) }
            button { class: "mt-2 bg-emerald-600 text-white rounded px-4 py-2 mr-2", onclick: move |_| {
                let p = AttachmentUploadReq {
                    record_type: rec_type(),
                    record_id: rec_id(),
                    file_name: file_name(),
                    extension: extension(),
                    bytes_base64: bytes_b64(),
                    operator_label: "web-operator".to_string(),
                    device_label: "manual-upload".to_string(),
                };
                let s = sess_for_attachment.clone(); let mut rid = rec_id; let mut fnm = file_name; let mut ext = extension; let mut b64 = bytes_b64; let mut at = attachments;
                spawn_local(async move {
                    match post_empty("/attachments", &p, Some(&s)).await {
                        Ok(_) => {
                            push_toast(toast_ctx, "Attachment uploaded", ToastKind::Success);
                            rid.set(String::new()); fnm.set(String::new()); ext.set("pdf".to_string()); b64.set(String::new());
                            if let Ok(v) = get_json::<Vec<AttachmentRow>>("/attachments", Some(&s)).await {
                                at.set(v);
                            }
                        }
                        Err(e) => push_toast(toast_ctx, e, ToastKind::Error),
                    }
                });
            }, "Upload Attachment" }
            {table_attachments(attachments())}
        }
        {list_content}
    } }
}

fn admin_page() -> Element {
    let Some(sess) = require_auth() else {
        return spinner();
    };
    let toast_ctx = use_context::<ToastCtx>();
    let mut loading = use_signal(|| true);
    let mut loaded = use_signal(|| false);
    let mut users = use_signal(Vec::<UserRow>::new);
    let mut user = use_signal(String::new);
    let mut pass = use_signal(String::new);
    let mut role = use_signal(String::new);
    if !loaded() {
        loaded.set(true);
        let fetch_sess = sess.clone();
        let mut l = loading;
        let mut u = users;
        spawn_local(async move {
            match get_json::<Vec<UserRow>>("/users", Some(&fetch_sess)).await {
                Ok(v) => u.set(v),
                Err(e) => push_toast(toast_ctx, e, ToastKind::Error),
            }
            l.set(false);
        });
    }
    let list_content = if loading() {
        spinner()
    } else {
        table_users(users())
    };
    rsx! { Shell { title: "Admin", active: Route::Admin {},
        div { class: "bg-white border rounded p-3",
            input { class: "border rounded px-3 py-2 mr-2", placeholder: "Username", value: "{user}", oninput: move |e| user.set(e.value()) }
            input { r#type: "password", class: "border rounded px-3 py-2 mr-2", placeholder: "Password", value: "{pass}", oninput: move |e| pass.set(e.value()) }
            select { class: "border rounded px-3 py-2 mr-2", value: "{role}", onchange: move |e| role.set(e.value()),
                option { value: "", "Select Role" }
                option { value: "Admin", "Admin" }
                option { value: "Coordinator", "Coordinator" }
                option { value: "Proctor", "Proctor" }
                option { value: "Auditor", "Auditor" }
            }
            button { class: "bg-emerald-600 text-white rounded px-4 py-2", onclick: move |_| {
                let p = CreateUserRequest { username: user(), password: pass(), role: role(), template_id: None }; let s = sess.clone(); let mut a=user; let mut b=pass; let mut c=role; let mut urows=users;
                spawn_local(async move {
                    match post_empty("/users", &p, Some(&s)).await {
                        Ok(_)=>{
                            push_toast(toast_ctx, "User created successfully", ToastKind::Success);
                            a.set(String::new()); b.set(String::new()); c.set(String::new());
                            if let Ok(updated) = get_json::<Vec<UserRow>>("/users", Some(&s)).await {
                                urows.set(updated);
                            }
                        },
                        Err(e)=>push_toast(toast_ctx, e, ToastKind::Error)
                    }
                });
            }, "Create User" }
        }
        {list_content}
    } }
}
#[component]
fn Shell(title: &'static str, active: Route, children: Element) -> Element {
    let mut auth = use_context::<AuthCtx>();
    let toast_ctx = use_context::<ToastCtx>();
    let nav = use_navigator();
    let role = (auth.session)()
        .as_ref()
        .map(jwt_role)
        .unwrap_or_else(|| "Auditor".to_string());
    let is_admin = role == "Admin";
    let is_coordinator = role == "Coordinator";
    rsx! {
        div { class: "min-h-screen bg-slate-100 flex",
            aside { class: "hidden md:block w-64 bg-slate-900 text-white p-4", h2 { class: "font-bold text-xl mb-3", "ProctorOps" }
                p { class: "text-xs text-slate-300 mb-2", "Role: {role}" }
                {menu(Route::Dashboard {}, active == Route::Dashboard {}, "Dashboard")}
                if role != "Auditor" { {menu(Route::Candidates {}, active == Route::Candidates {}, "Candidates")} }
                if role != "Auditor" { {menu(Route::Rooms {}, active == Route::Rooms {}, "Rooms")} }
                if is_admin || is_coordinator { {menu(Route::Proctors {}, active == Route::Proctors {}, "Proctors")} }
                if is_admin || is_coordinator { {menu(Route::Exams {}, active == Route::Exams {}, "Exams")} }
                if role != "Auditor" { {menu(Route::Sessions {}, active == Route::Sessions {}, "Sessions")} }
                if role != "Auditor" { {menu(Route::Assets {}, active == Route::Assets {}, "Assets")} }
                {menu(Route::Reports {}, active == Route::Reports {}, "Reports")}
                if is_admin || is_coordinator { {menu(Route::Templates {}, active == Route::Templates {}, "Templates")} }
                if role != "Auditor" { {menu(Route::Outputs {}, active == Route::Outputs {}, "Outputs")} }
                if is_admin { {menu(Route::Admin {}, active == Route::Admin {}, "Admin")} }
                button { class: "w-full mt-3 rounded bg-slate-700 py-2", onclick: move |_| { clear_session(); auth.session.set(None); nav.push(Route::Login {}); push_toast(toast_ctx, "Logged out", ToastKind::Info); }, "Logout" }
            }
            main { class: "flex-1 p-6", h1 { class: "text-2xl font-extrabold mb-3", "{title}" }, {children} }
        }
    }
}

pub fn jwt_role(session: &LoginResponse) -> String {
    let parts: Vec<&str> = session.jwt.split('.').collect();
    if parts.len() < 2 {
        return "Auditor".to_string();
    }
    let payload = parts[1].replace('-', "+").replace('_', "/");
    let padded = match payload.len() % 4 {
        2 => format!("{payload}=="),
        3 => format!("{payload}="),
        _ => payload,
    };
    let decoded = base64::engine::general_purpose::STANDARD.decode(padded);
    let Ok(bytes) = decoded else {
        return "Auditor".to_string();
    };
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
    parsed
        .get("role")
        .and_then(|v| v.as_str())
        .unwrap_or("Auditor")
        .to_string()
}

fn menu(route: Route, active: bool, label: &'static str) -> Element {
    let cls = if active {
        "block rounded bg-blue-600 px-3 py-2 mb-1"
    } else {
        "block rounded hover:bg-slate-800 px-3 py-2 mb-1"
    };
    rsx! { Link { class: "{cls}", to: route, "{label}" } }
}
pub fn metric(title: &'static str, value: String) -> Element {
    rsx! { div { class: "bg-white border rounded p-3", p { class: "text-sm text-slate-500", "{title}" } p { class: "text-xl font-bold", "{value}" } } }
}
pub fn spinner() -> Element {
    rsx! { div { class: "bg-white border rounded p-3 text-slate-600", "Loading..." } }
}
pub fn toast_bg(k: &ToastKind) -> &'static str {
    match k {
        ToastKind::Success => "bg-emerald-600",
        ToastKind::Error => "bg-rose-600",
        ToastKind::Info => "bg-slate-700",
    }
}

fn format_display_datetime(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("N/A") {
        return "N/A".to_string();
    }

    let mut candidate = trimmed.to_string();
    if candidate.contains(' ') && !candidate.contains('T') {
        candidate = candidate.replace(' ', "T");
    }
    if candidate.ends_with(" UTC") {
        candidate = candidate.trim_end_matches(" UTC").to_string() + "Z";
    }

    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(&candidate));
    if date.get_time().is_nan() {
        if let Some((year, month, day, hour24, minute)) = parse_datetime_components(trimmed) {
            return format_mmddyyyy_hhmm_ampm(year, month, day, hour24, minute);
        }
        return raw.to_string();
    }

    let month = date.get_month() + 1;
    let day = date.get_date() as u32;
    let year = date.get_full_year() as i32;
    let hour24 = date.get_hours() as u32;
    let minute = date.get_minutes() as u32;
    let ampm = if hour24 >= 12 { "PM" } else { "AM" };
    let mut hour12 = hour24 % 12;
    if hour12 == 0 {
        hour12 = 12;
    }
    format!("{month:02}/{day:02}/{year:04} {hour12:02}:{minute:02} {ampm}")
}

fn format_display_date_opt(raw: Option<&str>) -> String {
    let Some(value) = raw else {
        return "N/A".to_string();
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return "N/A".to_string();
    }

    let candidate = if trimmed.contains('T') {
        trimmed.to_string()
    } else {
        format!("{trimmed}T00:00:00")
    };
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(&candidate));
    if date.get_time().is_nan() {
        if let Some((year, month, day, _, _)) = parse_datetime_components(trimmed) {
            return format!("{month:02}/{day:02}/{year:04}");
        }
        return trimmed.to_string();
    }
    let month = date.get_month() + 1;
    let day = date.get_date() as u32;
    let year = date.get_full_year() as i32;
    format!("{month:02}/{day:02}/{year:04}")
}

fn parse_datetime_components(raw: &str) -> Option<(i32, u32, u32, u32, u32)> {
    let cleaned = raw
        .trim()
        .trim_end_matches('Z')
        .trim_end_matches(" UTC")
        .replace('T', " ");
    let mut parts = cleaned.split_whitespace();
    let date = parts.next()?;
    let time = parts.next().unwrap_or("00:00:00");

    let mut d = date.split('-');
    let year = d.next()?.parse::<i32>().ok()?;
    let month = d.next()?.parse::<u32>().ok()?;
    let day = d.next()?.parse::<u32>().ok()?;

    let mut t = time.split(':');
    let hour24 = t.next()?.parse::<u32>().ok()?;
    let minute = t.next()?.parse::<u32>().ok()?;
    Some((year, month, day, hour24, minute))
}

fn format_mmddyyyy_hhmm_ampm(year: i32, month: u32, day: u32, hour24: u32, minute: u32) -> String {
    let ampm = if hour24 >= 12 { "PM" } else { "AM" };
    let mut hour12 = hour24 % 12;
    if hour12 == 0 {
        hour12 = 12;
    }
    format!("{month:02}/{day:02}/{year:04} {hour12:02}:{minute:02} {ampm}")
}

fn push_toast(mut ctx: ToastCtx, msg: impl Into<String>, kind: ToastKind) {
    ctx.toast.set(Some(Toast {
        id: js_sys::Date::now() as i64,
        text: msg.into(),
        kind,
    }));
}

async fn get_json<T: for<'de> Deserialize<'de>>(
    path: &str,
    auth: Option<&LoginResponse>,
) -> Result<T, String> {
    let mut req = Request::get(&format!("{}{path}", api_base()));
    if let Some(a) = auth {
        req = req
            .header("Authorization", &format!("Bearer {}", a.jwt))
            .header("x-session-id", &a.session_id);
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json::<T>().await.map_err(|e| e.to_string())
}

async fn post_empty<T: Serialize>(
    path: &str,
    payload: &T,
    auth: Option<&LoginResponse>,
) -> Result<(), String> {
    let mut req =
        Request::post(&format!("{}{path}", api_base())).header("Content-Type", "application/json");
    if let Some(a) = auth {
        req = req
            .header("Authorization", &format!("Bearer {}", a.jwt))
            .header("x-session-id", &a.session_id);
    }
    let body = serde_json::to_string(payload).map_err(|e| e.to_string())?;
    let req = req.body(body).map_err(|e| e.to_string())?;
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if resp.ok() {
        Ok(())
    } else {
        Err(format!("HTTP {}", resp.status()))
    }
}

async fn post_json<TReq: Serialize, TResp: for<'de> Deserialize<'de>>(
    path: &str,
    payload: &TReq,
    auth: Option<&LoginResponse>,
) -> Result<TResp, String> {
    let mut req =
        Request::post(&format!("{}{path}", api_base())).header("Content-Type", "application/json");
    if let Some(a) = auth {
        req = req
            .header("Authorization", &format!("Bearer {}", a.jwt))
            .header("x-session-id", &a.session_id);
    }
    let body = serde_json::to_string(payload).map_err(|e| e.to_string())?;
    let req = req.body(body).map_err(|e| e.to_string())?;
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json::<TResp>().await.map_err(|e| e.to_string())
}

fn load_session() -> Option<LoginResponse> {
    let storage = web_sys::window()?.local_storage().ok().flatten()?;
    let raw = storage.get_item(SESSION_STORAGE_KEY).ok().flatten()?;
    serde_json::from_str::<LoginResponse>(&raw).ok()
}
fn save_session(s: &LoginResponse) {
    if let Some(st) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = st.set_item(
            SESSION_STORAGE_KEY,
            &serde_json::to_string(s).unwrap_or_default(),
        );
    }
}
fn clear_session() {
    if let Some(st) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = st.remove_item(SESSION_STORAGE_KEY);
    }
}

pub fn table_candidates(rows: Vec<CandidateRow>) -> Element {
    rsx! { div { class: "mt-3 bg-white border rounded p-3 overflow-auto",
        table { class: "w-full text-sm",
            thead { class: "bg-slate-50 text-left", tr { th { class: "p-2", "Candidate ID" } th { class: "p-2", "Barcode" } th { class: "p-2", "National ID" } } }
            tbody { for r in rows { tr { class: "border-t", td { class: "p-2", "{r.id}" } td { class: "p-2", "{r.scanned_barcode}" } td { class: "p-2", "{r.national_id}" } } } }
        }
    } }
}
pub fn table_rooms(rows: Vec<RoomRow>) -> Element {
    rsx! { div { class: "mt-3 bg-white border rounded p-3 overflow-auto",
        table { class: "w-full text-sm",
            thead { class: "bg-slate-50 text-left", tr { th { class: "p-2", "Room ID" } th { class: "p-2", "Capacity" } th { class: "p-2", "Location" } } }
            tbody { for r in rows { tr { class: "border-t", td { class: "p-2", "{r.id}" } td { class: "p-2", "{r.capacity}" } td { class: "p-2", "{r.location}" } } } }
        }
    } }
}
pub fn table_sessions(rows: Vec<SessionRow>) -> Element {
    rsx! { div { class: "mt-3 bg-white border rounded p-3 overflow-auto",
        table { class: "w-full text-sm",
            thead { class: "bg-slate-50 text-left", tr { th { class: "p-2", "Session ID" } th { class: "p-2", "Template" } th { class: "p-2", "Status" } th { class: "p-2", "Duration (min)" } } }
            tbody { for r in rows { tr { class: "border-t", td { class: "p-2", "{r.id}" } td { class: "p-2", "{r.template_name}" } td { class: "p-2", "{r.status}" } td { class: "p-2", "{r.duration_minutes}" } } } }
        }
    } }
}
pub fn table_assets(rows: Vec<AssetRow>) -> Element {
    rsx! { div { class: "mt-3 bg-white border rounded p-3 overflow-auto",
        table { class: "w-full text-sm",
            thead { class: "bg-slate-50 text-left", tr { th { class: "p-2", "Asset ID" } th { class: "p-2", "Booklet Code" } th { class: "p-2", "Status" } th { class: "p-2", "Incidents" } } }
            tbody { for r in rows { tr { class: "border-t", td { class: "p-2", "{r.id}" } td { class: "p-2", "{r.booklet_code}" } td { class: "p-2", "{r.tracking_status}" } td { class: "p-2", "{r.incident_count}" } } } }
        }
    } }
}
fn table_incidents(rows: Vec<IncidentRow>) -> Element {
    rsx! { div { class: "mt-3 bg-white border rounded p-3 overflow-auto",
        table { class: "w-full text-sm",
            thead { class: "bg-slate-50 text-left", tr { th { class: "p-2", "Session ID" } th { class: "p-2", "Avg Incidents" } } }
            tbody { for r in rows { tr { class: "border-t", td { class: "p-2", "{r.session_id}" } td { class: "p-2", "{r.avg_incidents}" } } } }
        }
    } }
}
fn table_return_rates(rows: Vec<ReturnRateRow>) -> Element {
    rsx! { div { class: "mt-3 bg-white border rounded p-3 overflow-auto",
        table { class: "w-full text-sm",
            thead { class: "bg-slate-50 text-left", tr {
                th { class: "p-2", "Session ID" }
                th { class: "p-2", "Total Assets" }
                th { class: "p-2", "Returned Assets" }
                th { class: "p-2", "Return Rate %" }
            } }
            tbody { for r in rows {
                tr { class: "border-t",
                    td { class: "p-2", "{r.session_id}" }
                    td { class: "p-2", "{r.total_assets}" }
                    td { class: "p-2", "{r.returned_assets}" }
                    td { class: "p-2", "{r.return_rate_pct}" }
                }
            } }
        }
    } }
}
pub fn table_materials_inventory(rows: Vec<MaterialInventoryRow>) -> Element {
    rsx! { div { class: "mt-3 bg-white border rounded p-3 overflow-auto",
        h3 { class: "font-bold mb-2", "Materials Inventory" }
        table { class: "w-full text-sm",
            thead { class: "bg-slate-50 text-left", tr {
                th { class: "p-2", "Asset ID" }
                th { class: "p-2", "Booklet Code" }
                th { class: "p-2", "Status" }
                th { class: "p-2", "Session" }
                th { class: "p-2", "Expires" }
                th { class: "p-2", "Incidents" }
            } }
            tbody { for r in rows {
                tr { class: "border-t",
                    td { class: "p-2", "{r.asset_id}" }
                    td { class: "p-2", "{r.booklet_code}" }
                    td { class: "p-2", "{r.tracking_status}" }
                    td { class: "p-2", "{r.session_id}" }
                    td { class: "p-2", "{format_display_date_opt(r.expires_on.as_deref())}" }
                    td { class: "p-2", "{r.incident_count}" }
                }
            } }
        }
    } }
}
pub fn table_alerts(rows: Vec<AlertRow>) -> Element {
    rsx! { div { class: "mt-3 bg-white border rounded p-3 overflow-auto",
        h3 { class: "font-bold mb-2", "Operations Alerts" }
        table { class: "w-full text-sm",
            thead { class: "bg-slate-50 text-left", tr {
                th { class: "p-2", "Type" }
                th { class: "p-2", "Severity" }
                th { class: "p-2", "Session" }
                th { class: "p-2", "Asset" }
                th { class: "p-2", "Message" }
            } }
            tbody { for r in rows {
                tr { class: "border-t",
                    td { class: "p-2", "{r.alert_type}" }
                    td { class: "p-2", "{r.severity}" }
                    td { class: "p-2", "{r.session_id.as_deref().unwrap_or(\"N/A\")}" }
                    td { class: "p-2", "{r.asset_id.as_deref().unwrap_or(\"N/A\")}" }
                    td { class: "p-2", "{r.message}" }
                }
            } }
        }
    } }
}
fn table_templates(rows: Vec<TemplateRow>) -> Element {
    rsx! { div { class: "mt-3 bg-white border rounded p-3 overflow-auto",
        table { class: "w-full text-sm",
            thead { class: "bg-slate-50 text-left", tr { th { class: "p-2", "Template ID" } th { class: "p-2", "Version" } th { class: "p-2", "Locked" } th { class: "p-2", "Created At" } } }
            tbody { for r in rows { tr { class: "border-t", td { class: "p-2", "{r.template_id}" } td { class: "p-2", "{r.version_no}" } td { class: "p-2", "{r.locked_for_final_print}" } td { class: "p-2", "{format_display_datetime(&r.created_at)}" } } } }
        }
    } }
}
pub fn table_users(rows: Vec<UserRow>) -> Element {
    rsx! { div { class: "mt-3 bg-white border rounded p-3 overflow-auto",
        table { class: "w-full text-sm",
            thead { class: "bg-slate-50 text-left", tr { th { class: "p-2", "Username" } th { class: "p-2", "Role" } } }
            tbody { for r in rows { tr { class: "border-t", td { class: "p-2", "{r.username}" } td { class: "p-2", "{r.role}" } } } }
        }
    } }
}
pub fn table_outputs(rows: Vec<OutputRow>) -> Element {
    rsx! { div { class: "mt-3 bg-white border rounded p-3 overflow-auto",
        table { class: "w-full text-sm",
            thead { class: "bg-slate-50 text-left", tr { th { class: "p-2", "Output ID" } th { class: "p-2", "Session ID" } th { class: "p-2", "Type" } th { class: "p-2", "Mode" } th { class: "p-2", "Created At" } } }
            tbody { for r in rows { tr { class: "border-t", td { class: "p-2", "{r.id}" } td { class: "p-2", "{r.session_id}" } td { class: "p-2", "{r.output_type}" } td { class: "p-2", "{r.mode}" } td { class: "p-2", "{format_display_datetime(&r.created_at)}" } } } }
        }
    } }
}

pub fn table_attachments(rows: Vec<AttachmentRow>) -> Element {
    rsx! { div { class: "mt-3 bg-white border rounded p-3 overflow-auto",
        table { class: "w-full text-sm",
            thead { class: "bg-slate-50 text-left", tr {
                th { class: "p-2", "Attachment ID" }
                th { class: "p-2", "Record" }
                th { class: "p-2", "File" }
                th { class: "p-2", "Size" }
                th { class: "p-2", "Captured At" }
            } }
            tbody {
                for r in rows {
                    tr { class: "border-t",
                        td { class: "p-2", "{r.id}" }
                        td { class: "p-2", "{r.record_type}:{r.record_id}" }
                        td { class: "p-2", "{r.file_name}.{r.extension}" }
                        td { class: "p-2", "{r.size_bytes}" }
                        td { class: "p-2", "{format_display_datetime(&r.captured_at)}" }
                    }
                }
            }
        }
    } }
}

#[cfg(test)]
mod component_tests;

#[cfg(test)]
mod additional_tests;

// ---------------------------------------------------------------------------
// Unit tests — exercise pure frontend helpers that do not depend on wasm/JS.
// Run with: `cargo test -p frontend --bin frontend`.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_datetime_components_parses_iso_with_t_separator() {
        let got = parse_datetime_components("2026-03-27T09:45:00").expect("parsed");
        assert_eq!(got, (2026, 3, 27, 9, 45));
    }

    #[test]
    fn parse_datetime_components_parses_space_separator_and_trailing_utc() {
        let got = parse_datetime_components("2026-03-27 13:30:00 UTC").expect("parsed");
        assert_eq!(got, (2026, 3, 27, 13, 30));
    }

    #[test]
    fn parse_datetime_components_rejects_garbage() {
        assert!(parse_datetime_components("not-a-datetime").is_none());
    }

    #[test]
    fn format_mmddyyyy_hhmm_ampm_handles_midnight_and_noon() {
        assert_eq!(
            format_mmddyyyy_hhmm_ampm(2026, 3, 27, 0, 5),
            "03/27/2026 12:05 AM"
        );
        assert_eq!(
            format_mmddyyyy_hhmm_ampm(2026, 3, 27, 12, 0),
            "03/27/2026 12:00 PM"
        );
        assert_eq!(
            format_mmddyyyy_hhmm_ampm(2026, 3, 27, 13, 45),
            "03/27/2026 01:45 PM"
        );
    }

    #[test]
    fn api_base_falls_back_to_default_when_env_unset() {
        // compile-time option_env! means this always returns the default unless built with API_BASE set.
        // The default points at the rocket backend.
        let base = api_base();
        assert!(
            base.ends_with("/api/v1"),
            "api_base must end with /api/v1, got {base}"
        );
    }

    #[test]
    fn login_response_roundtrips_through_json() {
        let original = LoginResponse {
            session_id: "11111111-1111-1111-1111-111111111111".into(),
            jwt: "header.payload.signature".into(),
            jwt_expires_at: "2030-01-01T00:00:00+00:00".into(),
            session_expires_at: "2030-01-01T00:00:00+00:00".into(),
        };
        let s = serde_json::to_string(&original).expect("serialize");
        let round: LoginResponse = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(round.session_id, original.session_id);
        assert_eq!(round.jwt, original.jwt);
    }

    #[test]
    fn gen_id_contains_prefix_and_uuid_length() {
        let id = gen_id("cand");
        assert!(id.starts_with("cand-"));
        // UUID-v4 has 36 chars; total = "cand-" (5) + 36 = 41.
        assert_eq!(id.len(), 41);
    }

    #[test]
    fn route_enum_has_login_and_dashboard_variants() {
        // Construct variants to confirm the Route enum — which drives top-level
        // client-side routing — exposes the expected routes the auth flow relies on.
        let _login: Route = Route::Login {};
        let _dashboard: Route = Route::Dashboard {};
        let _reports: Route = Route::Reports {};
        let _outputs: Route = Route::Outputs {};
    }
}
