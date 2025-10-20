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

use crate::{app::AppState, middleware::security::SecurityContext};
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

/// Rate limiting middleware that checks if the request should be blocked
/// Uses the SecurityContext injected by security_middleware
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    // Get security context from request extensions
    let security_context = match request.extensions().get::<SecurityContext>() {
        Some(ctx) => ctx.clone(),
        None => {
            tracing::error!("SecurityContext not found in request extensions. security_middleware should run before rate_limit_middleware.");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Extract path for rate limiting key (simple approach for now)
    let path = request.uri().path();
    let rate_limit_key = format!("{}:{}", security_context.ip_address, path);

    // Check if IP is blocked (returns true if allowed, false if blocked/newly blocked)
    match state
        .security
        .check_rate_limit(security_context.ip_address, &rate_limit_key)
        .await
    {
        Ok((false, newly_blocked)) => {
            // IP is blocked or just got blocked
            if newly_blocked {
                // First time being blocked - log this event only
                tracing::warn!(
                    "IP blocked for rate limiting: {} (path: {})",
                    security_context.ip_address,
                    path
                );

                let _ = state
                    .security
                    .log_access_attempt(
                        Some(security_context.ip_address),
                        security_context.user_agent.clone(),
                        &rate_limit_key,
                        "rate_limited_blocked",
                        false,
                    )
                    .await;
            } else {
                // Already blocked - don't log, prevents DB growth from DDoS
                tracing::debug!(
                    "Blocked IP attempted access (not logged): {}",
                    security_context.ip_address
                );
            }

            StatusCode::TOO_MANY_REQUESTS.into_response()
        }
        Err(e) => {
            tracing::error!("Rate limit check failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
        Ok((true, _)) => {
            // Not rate limited, continue to next middleware/handler
            next.run(request).await
        }
    }
}
