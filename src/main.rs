use axum::{
    routing::{get, post, delete},
    Router,
    http::Method,
};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use tokio::signal;

mod models;
mod services;
mod repository;
mod handlers;
mod utils;

use handlers::{ScanHandler, CoverageHandler, HealthHandler};
use services::{FileSystemScanner, DefaultCoverageCalculator};
use repository::InMemoryScanRepository;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::scan_handler::create_scan,
        handlers::scan_handler::get_scan,
        handlers::scan_handler::list_scans,
        handlers::scan_handler::delete_scan,
        handlers::coverage_handler::get_coverage_metrics,
        handlers::coverage_handler::get_coverage_report,
        handlers::health_handler::health_check,
    ),
    components(
        schemas(
            models::ScanRequest,
            models::ScanResult,
            models::ScanStatus,
            models::RequirementAnnotation,
            models::CoverageMetrics,
            models::CoverageReport,
            models::FileCoverage,
            handlers::health_handler::HealthResponse,
        )
    ),
    tags(
        (name = "scans", description = "Scan management endpoints"),
        (name = "coverage", description = "Coverage analysis endpoints"),
        (name = "health", description = "Health check endpoints"),
    ),
    info(
        title = "SDD Navigator Service",
        description = "HTTP service for scanning codebase for @req annotations and calculating specification coverage metrics",
        version = "0.1.0",
        contact(
            name = "SDD Navigator Team",
            email = "team@sdd-navigator.com"
        )
    )
)]
struct ApiDoc;

#[derive(Clone)]
pub struct AppState {
    pub scan_handler: handlers::scan_handler::ScanHandler,
    pub coverage_handler: handlers::coverage_handler::CoverageHandler,
    pub health_handler: handlers::health_handler::HealthHandler,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sdd_navigator_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app_state = create_app_state().await?;

    let api_doc = ApiDoc::openapi();
    let swagger_ui = SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", api_doc);

    let app = Router::new()
        .route("/health", get(handlers::health_handler::health_check))
        .route("/api/v1/scans", post(handlers::scan_handler::create_scan))
        .route("/api/v1/scans", get(handlers::scan_handler::list_scans))
        .route("/api/v1/scans/:scan_id", get(handlers::scan_handler::get_scan))
        .route("/api/v1/scans/:scan_id", delete(handlers::scan_handler::delete_scan))
        .route(
            "/api/v1/scans/:scan_id/coverage",
            get(handlers::coverage_handler::get_coverage_metrics),
        )
        .route(
            "/api/v1/scans/:scan_id/coverage/report",
            get(handlers::coverage_handler::get_coverage_report),
        )
        .merge(swagger_ui)
        .with_state(app_state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods([Method::GET, Method::POST, Method::DELETE])
                        .allow_headers(Any),
                ),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("SDD Navigator Service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    let shutdown_signal = shutdown_signal();
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    tracing::info!("SDD Navigator Service shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        let _ = tokio::signal::ctrl_c().await;
        tracing::info!("Received terminate signal, shutting down...");
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, shutting down...");
        },
        _ = terminate => {
            tracing::info!("Received terminate signal, shutting down...");
        },
    }
}

async fn create_app_state() -> Result<AppState, Box<dyn std::error::Error>> {
    let scanner = FileSystemScanner::with_default_processor();
    let coverage_calculator = DefaultCoverageCalculator::new();
    let repository = InMemoryScanRepository::new();

    let scan_handler = ScanHandler::new(scanner, repository.clone());
    let coverage_handler = CoverageHandler::new(coverage_calculator, repository.clone());
    let health_handler = HealthHandler::new(repository);

    Ok(AppState {
        scan_handler,
        coverage_handler,
        health_handler,
    })
}
