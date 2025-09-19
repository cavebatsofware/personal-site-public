use axum::{
    extract::Path,
    http::{StatusCode, header},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::{collections::HashSet, env};
use tower::ServiceBuilder;
use tower_http::{trace::TraceLayer, services::ServeDir};
use tracing_subscriber;

#[derive(Debug)]
struct AppState {
    valid_codes: HashSet<String>,
}

impl AppState {
    fn new() -> anyhow::Result<Self> {
        let codes_env = env::var("ACCESS_CODES")
            .unwrap_or_else(|_| {
                tracing::warn!("ACCESS_CODES environment variable not set, will fail startup");
                "".to_string()
            });

        let valid_codes: HashSet<String> = codes_env
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if valid_codes.is_empty() {
            anyhow::bail!("No valid access codes found in ACCESS_CODES environment variable");
        }

        tracing::info!("Loaded {} valid access codes", valid_codes.len());

        Ok(AppState { valid_codes })
    }

    fn is_valid_code(&self, code: &str) -> bool {
        self.valid_codes.contains(code)
    }
}

async fn serve_document(Path(code): Path<String>) -> Result<Html<String>, StatusCode> {
    let state = AppState::new().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !state.is_valid_code(&code) {
        tracing::warn!("Invalid access code attempted: {}", code);
        return Err(StatusCode::NOT_FOUND);
    }

    tracing::info!("Valid access code used: {}", code);

    // Read the HTML file
    let html_content = tokio::fs::read_to_string("index.html")
        .await
        .map_err(|e| {
            tracing::error!("Failed to read index.html: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Html(html_content))
}

async fn download_document(Path(code): Path<String>) -> impl IntoResponse {
    let state = match AppState::new() {
        Ok(state) => state,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if !state.is_valid_code(&code) {
        tracing::warn!("Invalid access code attempted for download: {}", code);
        return StatusCode::NOT_FOUND.into_response();
    }

    tracing::info!("Valid access code used for download: {}", code);

    // Read the PDF file
    let pdf_content = match tokio::fs::read("Document.pdf").await {
        Ok(content) => content,
        Err(e) => {
            tracing::error!("Failed to read document PDF: {}", e);
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    // Create response with proper headers for PDF download
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/pdf"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"example-document.pdf\""),
        ],
        pdf_content,
    ).into_response()
}


async fn health_check() -> &'static str {
    "OK"
}

async fn serve_landing() -> Result<Html<String>, StatusCode> {
    // Read the landing page HTML file
    let html_content = tokio::fs::read_to_string("landing.html")
        .await
        .map_err(|e| {
            tracing::error!("Failed to read landing.html: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Html(html_content))
}

async fn handle_404() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not Found")
}

async fn serve_favicon() -> impl IntoResponse {
    // Serve application icon as favicon
    let favicon = tokio::fs::read("assets/icons/android-chrome-192x192.png").await;
    match favicon {
        Ok(content) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "image/png")],
            content,
        ).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Favicon not found").into_response(),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Validate environment setup
    let _state = AppState::new()?;

    // Build the application with routes
    let app = Router::new()
        .route("/", get(serve_landing))
        .route("/favicon.ico", get(serve_favicon))
        .route("/document", get(serve_landing))
        .route("/document/{code}", get(serve_document))
        .route("/document/{code}/download", get(download_document))
        .route("/health", get(health_check))
        .nest_service("/assets", ServeDir::new("./assets"))
        .fallback(handle_404)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
        );

    // Determine the bind address
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    let addr = format!("0.0.0.0:{}", port);

    tracing::info!("Document server starting on {}", addr);
    tracing::info!("Access document at: http://localhost:{}/document/{{your-code}}", port);

    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to {}: {}", addr, e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    Ok(())
}