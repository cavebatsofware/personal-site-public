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

use super::{AdminAuthBackend, Credentials};
use crate::email::EmailService;
use crate::errors::{AppError, AppResult};
use crate::settings::SettingsService;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use axum_login::AuthSession;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub type AdminAuthSession = AuthSession<AdminAuthBackend>;

#[derive(Clone)]
pub struct AdminState {
    pub auth_backend: AdminAuthBackend,
    pub email_service: Arc<EmailService>,
    pub settings: SettingsService,
}

pub fn admin_api_routes() -> Router<AdminState> {
    Router::new()
        .route("/api/admin/register", post(register))
        .route("/api/admin/login", post(login))
        .route("/api/admin/logout", post(logout))
        .route("/api/admin/verify-email", get(verify_email))
        .route("/api/admin/me", get(me))
}

#[derive(Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct RegisterResponse {
    message: String,
    email: String,
}

async fn register(
    State(state): State<AdminState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<RegisterResponse>> {
    // Check if registration is enabled
    let registration_enabled = state
        .settings
        .get_bool("admin_registration_enabled", Some("system"), None)
        .await
        .unwrap_or(false);

    if !registration_enabled {
        return Err(AppError::AuthError(
            "Registration is currently disabled".to_string(),
        ));
    }

    // Create admin user
    let (admin, verification_token) = state
        .auth_backend
        .create_admin(&req.email, &req.password)
        .await
        .map_err(|e| AppError::AuthError(e.to_string()))?;

    // Send verification email
    state
        .email_service
        .send_verification_email(&admin.email, &verification_token)
        .await
        .map_err(|e| AppError::AuthError(format!("Failed to send verification email: {}", e)))?;

    Ok(Json(RegisterResponse {
        message: "Registration successful. Please check your email to verify your account."
            .to_string(),
        email: admin.email,
    }))
}

async fn login(
    mut auth_session: AdminAuthSession,
    Json(creds): Json<Credentials>,
) -> AppResult<Json<UserResponse>> {
    let user = auth_session
        .authenticate(creds)
        .await
        .map_err(|e| AppError::AuthError(e.to_string()))?
        .ok_or_else(|| AppError::AuthError("Invalid email or password".to_string()))?;

    auth_session
        .login(&user)
        .await
        .map_err(|e| AppError::AuthError(e.to_string()))?;

    Ok(Json(UserResponse {
        id: user.id,
        email: user.email,
        email_verified: user.email_verified,
    }))
}

async fn logout(mut auth_session: AdminAuthSession) -> AppResult<StatusCode> {
    auth_session
        .logout()
        .await
        .map_err(|e| AppError::AuthError(e.to_string()))?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
struct VerifyQuery {
    token: String,
}

#[derive(Serialize)]
struct VerifyResponse {
    message: String,
    email: String,
}

async fn verify_email(
    State(state): State<AdminState>,
    Query(query): Query<VerifyQuery>,
) -> AppResult<Json<VerifyResponse>> {
    let admin = state
        .auth_backend
        .verify_email(&query.token)
        .await
        .map_err(|e| AppError::AuthError(e.to_string()))?;

    Ok(Json(VerifyResponse {
        message: "Email verified successfully. You can now log in.".to_string(),
        email: admin.email,
    }))
}

#[derive(Serialize)]
struct UserResponse {
    id: uuid::Uuid,
    email: String,
    email_verified: bool,
}

async fn me(auth_session: AdminAuthSession) -> AppResult<Json<UserResponse>> {
    let user = auth_session
        .user
        .ok_or_else(|| AppError::AuthError("Not authenticated".to_string()))?;

    Ok(Json(UserResponse {
        id: user.id,
        email: user.email,
        email_verified: user.email_verified,
    }))
}
