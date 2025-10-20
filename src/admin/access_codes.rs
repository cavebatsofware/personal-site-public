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

use crate::entities::{access_code, AccessCode};
use crate::errors::{AppError, AppResult};
use crate::middleware::AuthenticatedUser;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get},
    Router,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone)]
pub struct AccessCodeState {
    pub db: DatabaseConnection,
}

pub fn access_code_routes() -> Router<AccessCodeState> {
    Router::new()
        .route("/api/admin/access-codes", get(list_codes).post(create_code))
        .route("/api/admin/access-codes/{id}", delete(delete_code))
}

#[derive(Serialize)]
struct AccessCodeResponse {
    id: Uuid,
    code: String,
    name: String,
    expires_at: Option<String>,
    created_at: String,
    is_expired: bool,
    usage_count: i32,
}

impl From<access_code::Model> for AccessCodeResponse {
    fn from(model: access_code::Model) -> Self {
        let now = Utc::now();
        let is_expired = model
            .expires_at
            .as_ref()
            .map(|exp| exp.with_timezone(&Utc) < now)
            .unwrap_or(false);

        Self {
            id: model.id,
            code: model.code,
            name: model.name,
            expires_at: model
                .expires_at
                .map(|dt| dt.with_timezone(&Utc).to_rfc3339()),
            created_at: model.created_at.with_timezone(&Utc).to_rfc3339(),
            is_expired,
            usage_count: model.usage_count,
        }
    }
}

async fn list_codes(
    State(state): State<AccessCodeState>,
    _user: AuthenticatedUser,
) -> AppResult<Json<Vec<AccessCodeResponse>>> {
    let codes = AccessCode::find().all(&state.db).await?;
    let response: Vec<AccessCodeResponse> = codes.into_iter().map(Into::into).collect();
    Ok(Json(response))
}

#[derive(Deserialize)]
struct CreateCodeRequest {
    code: String,
    name: String,
    expires_at: Option<String>, // ISO 8601 format
}

async fn create_code(
    State(state): State<AccessCodeState>,
    user: AuthenticatedUser,
    Json(req): Json<CreateCodeRequest>,
) -> AppResult<(StatusCode, Json<AccessCodeResponse>)> {
    if req.code.trim().is_empty() {
        return Err(AppError::AuthError(
            "Access code cannot be empty".to_string(),
        ));
    }

    // Check if code already exists
    let existing = AccessCode::find()
        .filter(access_code::Column::Code.eq(&req.code))
        .one(&state.db)
        .await?;

    if existing.is_some() {
        return Err(AppError::AuthError(
            "Access code already exists".to_string(),
        ));
    }

    let expires_at = if let Some(exp_str) = req.expires_at {
        Some(
            chrono::DateTime::parse_from_rfc3339(&exp_str)
                .map_err(|_| AppError::AuthError("Invalid expiration date format".to_string()))?
                .into(),
        )
    } else {
        None
    };

    let new_code = access_code::ActiveModel {
        id: Set(Uuid::new_v4()),
        code: Set(req.code),
        name: Set(req.name),
        expires_at: Set(expires_at),
        created_at: Set(Utc::now().into()),
        created_by: Set(user.id),
        usage_count: Set(0),
    };

    let result = new_code.insert(&state.db).await?;

    Ok((StatusCode::CREATED, Json(result.into())))
}

async fn delete_code(
    State(state): State<AccessCodeState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let code = AccessCode::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::AuthError("Access code not found".to_string()))?;

    let active_model: access_code::ActiveModel = code.into();
    active_model.delete(&state.db).await?;

    Ok(StatusCode::NO_CONTENT)
}
