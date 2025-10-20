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

use crate::{admin::AdminUserAuth, app::AppState, middleware::security::SecurityContext};
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

/// Access logging middleware that logs requests after they complete
/// Uses the SecurityContext and response status to determine success/failure
pub async fn access_log_middleware(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    // Get security context from request extensions
    let security_context = match request.extensions().get::<SecurityContext>() {
        Some(ctx) => ctx.clone(),
        None => {
            tracing::error!("SecurityContext not found in request extensions. security_middleware must run before access_log_middleware.");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Extract request information for logging
    let path = request.uri().path().to_string();
    let method = request.method().to_string();

    // Check if this is an authenticated admin request
    let is_admin = request.extensions().get::<AdminUserAuth>().is_some();

    // Continue to the next middleware/handler
    let response = next.run(request).await;

    // Determine if the request was successful based on status code
    let status = response.status();
    let success = status.is_success();

    // Determine action type based on path for filtering
    let action_type = determine_action_type(&path);

    // Only log if logging is enabled and meets criteria
    if should_log(&action_type, success, &state) {
        // Use special action prefix for admin-authenticated requests
        let action = if is_admin {
            format!("admin:{}", method)
        } else {
            method.clone()
        };

        // Log the access attempt
        if let Err(e) = state
            .security
            .log_access_attempt(
                Some(security_context.ip_address),
                security_context.user_agent,
                &format!("{}:{}", method, path),
                &action,
                success,
            )
            .await
        {
            tracing::error!("Failed to log access attempt: {}", e);
        }
    }

    response
}

/// Determine the action type based on the request path for filtering purposes
fn determine_action_type(path: &str) -> String {
    if path.starts_with("/assets/") {
        "asset".to_string()
    } else if path.starts_with("/admin/assets") {
        "asset".to_string()
    } else if path == "/health" {
        "health".to_string()
    } else if path == "/favicon.ico" {
        "favicon".to_string()
    } else {
        "request".to_string()
    }
}

/// Determine if we should log this request based on configuration and context
fn should_log(action: &str, success: bool, state: &AppState) -> bool {
    // Don't log noisy requests that aren't meaningful for tracking
    // - health checks (monitoring pings)
    // - favicon requests (browser automatic requests)
    // - asset requests (CSS, JS, images - these are just page dependencies)
    if matches!(action, "health" | "favicon" | "asset") {
        return false;
    }

    // Check if logging is enabled
    if !state.security.config.enable_logging {
        return false;
    }

    // Check if we should log successful attempts
    if success && !state.security.config.log_successful_attempts {
        return false;
    }

    // Always log failed attempts if logging is enabled
    true
}
