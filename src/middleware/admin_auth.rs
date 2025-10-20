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

use crate::admin::{AdminAuthBackend, AdminUserAuth};
use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_login::AuthSession;

pub type AdminAuthSession = AuthSession<AdminAuthBackend>;

pub async fn require_admin_auth(
    auth_session: AdminAuthSession,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    tracing::debug!(
        "require_admin_auth middleware called for: {}",
        request.uri()
    );
    tracing::debug!("Auth session user present: {}", auth_session.user.is_some());

    if let Some(user) = auth_session.user {
        tracing::debug!("User authenticated: {}", user.email);
        request.extensions_mut().insert(user);
        let response = next.run(request).await;
        tracing::debug!("Handler completed with status: {}", response.status());
        response
    } else {
        tracing::warn!("Authentication required but user not present");
        (StatusCode::UNAUTHORIZED, "Not authenticated").into_response()
    }
}

/// Extension type for accessing authenticated admin user in handlers
/// Usage in handlers:
/// ```ignore
/// async fn handler(Extension(user): Extension<AdminUserAuth>) -> impl IntoResponse {
///     // user is guaranteed to be authenticated here
/// }
/// ```
pub type AuthenticatedUser = axum::extract::Extension<AdminUserAuth>;
