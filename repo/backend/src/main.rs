use app_api_v1::{catchers_v1, routes_v1};
use app_services::audit_service::AuditService;
use app_services::auth_service::AuthService;
use app_services::candidate_service::CandidateService;
use app_services::cleansing_service::CleansingService;
use app_services::dedupe_service::DedupeService;
use app_services::file_handling_service::FileHandlingService;
use app_services::messaging_service::MessagingService;
use app_services::output_service::OutputService;
use app_services::reporting_service::ReportingService;
use app_services::template_service::TemplateService;
use base64::Engine;
use rocket::{Build, Rocket};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use tracing_subscriber::EnvFilter;

// Live in the backend binary; dead in `backend_bootstrap_tests` which
// `include!`s this file but only exercises `build_cors` / `init_tracing`.
#[allow(dead_code)]
async fn build_rocket() -> anyhow::Result<Rocket<Build>> {
    let database_url = env::var("DATABASE_URL")?;
    let jwt_secret = env::var("JWT_SECRET")?;
    let aes_key_b64 = env::var("AES_KEY_BASE64")?;

    let aes_raw = base64::engine::general_purpose::STANDARD.decode(aes_key_b64)?;
    let aes_key: [u8; 32] = aes_raw
        .try_into()
        .map_err(|_| anyhow::anyhow!("AES key must decode to exactly 32 bytes"))?;

    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    let auth_service = AuthService::new(pool.clone(), jwt_secret);
    let candidate_service = CandidateService::new(pool.clone(), aes_key);

    let cleansing_service = CleansingService::new(pool.clone());
    let _dedupe_service = DedupeService;
    let _file_service = FileHandlingService;
    let _template_service = TemplateService;

    let audit_service = AuditService::new(pool.clone());
    let reporting_service = ReportingService::new(pool.clone());
    let output_service = OutputService::new(pool.clone());
    let messaging_service = MessagingService::new(pool.clone());
    let cors = build_cors()?;

    Ok(rocket::build()
        .manage(pool)
        .manage(auth_service)
        .manage(candidate_service)
        .manage(cleansing_service)
        .manage(audit_service)
        .manage(reporting_service)
        .manage(output_service)
        .manage(messaging_service)
        .attach(cors)
        .mount("/api/v1", routes_v1())
        .register("/", catchers_v1()))
}

fn build_cors() -> anyhow::Result<rocket_cors::Cors> {
    let origins_env = env::var("ROCKET_CORS_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:8080,http://127.0.0.1:8080".to_string());
    let origins: Vec<String> = origins_env
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    let allowed_origins = if origins.is_empty() {
        AllowedOrigins::some_exact(&["http://localhost:8080"])
    } else {
        AllowedOrigins::some_exact(&origins)
    };

    CorsOptions {
        allowed_origins,
        allowed_methods: vec!["GET", "POST", "PUT", "DELETE", "OPTIONS", "HEAD"]
            .into_iter()
            .map(|m| m.parse().expect("valid CORS method"))
            .collect(),
        allowed_headers: AllowedHeaders::all(),
        allow_credentials: false, // Must be false when using origins: all()
        ..Default::default()
    }
    .to_cors()
    .map_err(|err| anyhow::anyhow!("failed to build CORS: {err}"))
}

#[rocket::main]
#[allow(dead_code)] // entrypoint in the binary; dead in `backend_bootstrap_tests`.
async fn main() {
    init_tracing();

    if let Err(err) = run().await {
        tracing::error!(error = %err, "backend startup failed");
        std::process::exit(1);
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_level(true)
        .without_time()
        .try_init();
}

#[allow(dead_code)] // called only from `main`; dead in `backend_bootstrap_tests`.
async fn run() -> anyhow::Result<()> {
    let rocket = build_rocket().await?;
    tracing::info!("launching backend");
    rocket.launch().await?;
    Ok(())
}
