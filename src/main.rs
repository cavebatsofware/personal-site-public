/*  This file is part of a personal website project codename personal-site
 *  Copyright (C) 2025  Grant DeFayette
 *
 *  personal-site is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  personal-site is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with personal-site.  If not, see <https://www.gnu.org/licenses/>.
 */

use axum::{
    extract::Path,
    http::{header, StatusCode},
    middleware::{from_fn, from_fn_with_state},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use axum_login::AuthManagerLayerBuilder;
use std::{env, sync::Arc};
use time::Duration as TimeDuration;
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;

mod admin;
mod app;
mod database;
mod email;
mod entities;
mod errors;
mod middleware;
mod migration;
mod s3;
mod security;
mod settings;

use self::middleware::{
    access_log_middleware, rate_limit_middleware, require_admin_auth, security_middleware,
};
use app::AppState;
use errors::{AppError, AppResult};

#[cfg(test)]
mod tests;

async fn serve_access(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(code): Path<String>,
) -> AppResult<Html<String>> {
    if !state.is_valid_code(&code).await.unwrap_or(false) {
        return Err(AppError::InvalidAccess);
    }

    tracing::info!("Valid access code used: {}", code);

    let html_bytes =
        state.s3.get_file(&code, "index.html").await.map_err(|e| {
            AppError::FileSystem(std::io::Error::new(std::io::ErrorKind::NotFound, e))
        })?;

    let html_content = String::from_utf8(html_bytes).map_err(|e| {
        AppError::FileSystem(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    })?;

    Ok(Html(html_content))
}

async fn download_access(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(code): Path<String>,
) -> AppResult<impl IntoResponse> {
    if !state.is_valid_code(&code).await.unwrap_or(false) {
        return Err(AppError::InvalidAccess);
    }

    tracing::info!("Valid access code used for download: {}", code);

    let pdf_content =
        state.s3.get_file(&code, "Resume.pdf").await.map_err(|e| {
            AppError::FileSystem(std::io::Error::new(std::io::ErrorKind::NotFound, e))
        })?;

    let response = (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/pdf"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"Grant_DeFayette_Resume.pdf\"",
            ),
        ],
        pdf_content,
    );

    Ok(response)
}

async fn health_check() -> &'static str {
    "OK"
}

async fn serve_landing() -> AppResult<Html<String>> {
    let html_content = tokio::fs::read_to_string("landing.html")
        .await
        .map_err(AppError::FileSystem)?;

    let site_domain = env::var("SITE_DOMAIN").map_err(|_| {
        AppError::Configuration("SITE_DOMAIN environment variable is required".to_string())
    })?;
    let site_url = env::var("SITE_URL").map_err(|_| {
        AppError::Configuration("SITE_URL environment variable is required".to_string())
    })?;

    let templated_content = html_content
        .replace("{{SITE_DOMAIN}}", &site_domain)
        .replace("{{SITE_URL}}", &site_url);

    Ok(Html(templated_content))
}

async fn handle_404() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not Found")
}

async fn serve_favicon() -> AppResult<impl IntoResponse> {
    // return android-chrome-192x192.png as favicon
    let content = tokio::fs::read("assets/icons/android-chrome-192x192.png")
        .await
        .map_err(AppError::FileSystem)?;

    let response = (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/png")],
        content,
    );

    Ok(response)
}

async fn serve_admin_spa() -> AppResult<impl IntoResponse> {
    let html_content = tokio::fs::read_to_string("admin-assets/index.html")
        .await
        .map_err(AppError::FileSystem)?;

    let response = (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html_content,
    );

    Ok(response)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Check for migration command
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "migrate" {
        match run_migrations_sync().await {
            Ok(_) => {
                tracing::info!("Database migrations completed successfully");
                return Ok(());
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Database migration failed: {}", e));
            }
        }
    }

    // Create shared app state with database connection
    let state = AppState::new().await?;

    // Setup session store for admin authentication
    // Create a separate sqlx pool for tower-sessions
    let database_url = env::var("DATABASE_URL")?;
    let session_pool = sqlx::PgPool::connect(&database_url).await?;

    let session_store = PostgresStore::new(session_pool);
    session_store.migrate().await?;

    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(TimeDuration::days(7)));

    // Setup admin auth backend
    let admin_backend = admin::AdminAuthBackend::new(state.db.clone());
    let auth_layer =
        AuthManagerLayerBuilder::new(admin_backend.clone(), session_layer.clone()).build();

    // Setup email service
    let email_service = Arc::new(email::EmailService::new().await?);

    // Create admin state
    let admin_state = admin::routes::AdminState {
        auth_backend: admin_backend.clone(),
        email_service: email_service.clone(),
        settings: state.settings.clone(),
    };

    // Build admin routes
    let admin_routes = admin::routes::admin_api_routes()
        .with_state(admin_state)
        .layer(auth_layer.clone());

    // Build access code management routes
    let access_code_state = admin::access_codes::AccessCodeState {
        db: state.db.clone(),
    };
    let access_code_routes = admin::access_codes::access_code_routes()
        .with_state(access_code_state)
        .layer(from_fn(require_admin_auth))
        .layer(auth_layer.clone());

    // Build access log management routes
    let access_log_state = admin::access_logs::AccessLogState {
        db: state.db.clone(),
    };
    let access_log_routes = admin::access_logs::access_log_routes()
        .with_state(access_log_state)
        .layer(from_fn(require_admin_auth))
        .layer(auth_layer.clone());

    // Build settings management routes
    let settings_state = admin::settings::SettingsState {
        settings: state.settings.clone(),
    };
    let settings_routes = admin::settings::settings_routes()
        .with_state(settings_state)
        .layer(from_fn(require_admin_auth))
        .layer(auth_layer);

    // Build the application with routes and middleware stack
    let app = Router::new()
        .route("/", get(serve_landing))
        .route("/favicon.ico", get(serve_favicon))
        .route("/access", get(serve_landing))
        .route("/access/{code}", get(serve_access))
        .route("/access/{code}/download", get(download_access))
        // Alias routes for resume
        .route("/resume/{code}", get(serve_access))
        .route("/resume/{code}/download", get(download_access))
        .route("/health", get(health_check))
        .nest_service("/admin/assets", ServeDir::new("./admin-assets/assets"))
        .route("/admin", get(serve_admin_spa))
        .route("/admin/{*path}", get(serve_admin_spa))
        .nest_service("/assets", ServeDir::new("./assets"))
        .merge(admin_routes)
        .merge(access_code_routes)
        .merge(access_log_routes)
        .merge(settings_routes)
        .fallback(handle_404)
        .with_state(state.clone())
        .layer(
            ServiceBuilder::new()
                // Security middleware runs first to extract context
                .layer(from_fn(security_middleware))
                // Rate limiting uses security context
                .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
                // Access logging runs last to capture final response
                .layer(from_fn_with_state(state.clone(), access_log_middleware))
                // Standard HTTP tracing
                .layer(TraceLayer::new_for_http()),
        );

    // Start cleanup task for old entries
    // Runs every 5 minutes to prevent memory leaks in rate_limit_cache
    let cleanup_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // Every 5 minutes
        loop {
            interval.tick().await;
            if let Err(e) = cleanup_state.security.cleanup_old_entries().await {
                tracing::error!("Failed to cleanup old entries: {}", e);
            }
        }
    });

    // Determine the bind address
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    let addr = format!("0.0.0.0:{}", port);

    // Production environments will likely want to set RUST_LOG=warn
    // unless they want to see very verbose logs
    tracing::info!("Server starting on {}", addr);
    tracing::info!("Access at: http://localhost:{}/access/{{your-code}}", port);
    tracing::info!("RUST_LOG environment variable: {:?}", env::var("RUST_LOG"));

    // Start the server with connection info support
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to {}: {}", addr, e))?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    Ok(())
}

async fn run_migrations_sync() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Running database migrations...");

    let db = database::establish_connection().await?;
    tracing::info!("Database connection established for migrations");

    database::run_migrations(&db).await?;
    tracing::info!("Database migrations completed successfully");

    database::close_connection(db).await?;
    Ok(())
}
