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

use crate::admin::pagination::{Paginated, PaginationParams};
use crate::entities::{access_log, AccessLog};
use crate::errors::AppResult;
use crate::middleware::AuthenticatedUser;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use sea_orm::{DatabaseConnection, EntityTrait, Order, PaginatorTrait, QueryOrder};
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone)]
pub struct AccessLogState {
    pub db: DatabaseConnection,
}

pub fn access_log_routes() -> Router<AccessLogState> {
    Router::new().route("/api/admin/access-logs", get(list_logs).delete(clear_logs))
}

#[derive(Serialize)]
struct AccessLogResponse {
    id: Uuid,
    access_code: String,
    ip_address: Option<String>,
    user_agent: Option<String>,
    count: Option<i32>,
    last_access_time: Option<String>,
    action: String,
    success: bool,
    created_at: String,
}

impl From<access_log::Model> for AccessLogResponse {
    fn from(model: access_log::Model) -> Self {
        Self {
            id: model.id,
            access_code: model.access_code,
            ip_address: model.ip_address,
            user_agent: model.user_agent,
            count: model.count,
            last_access_time: model
                .last_access_time
                .map(|dt| dt.with_timezone(&chrono::Utc).to_rfc3339()),
            action: model.action,
            success: model.success,
            created_at: model.created_at.with_timezone(&chrono::Utc).to_rfc3339(),
        }
    }
}

async fn list_logs(
    State(state): State<AccessLogState>,
    _user: AuthenticatedUser,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<Paginated<AccessLogResponse>>> {
    // Validate pagination params
    let validated = params.validate();

    // Create paginator
    let paginator = AccessLog::find()
        .order_by(access_log::Column::CreatedAt, Order::Desc)
        .paginate(&state.db, validated.per_page);

    // Get total count and pages
    let total = paginator.num_items().await?;
    let total_pages = paginator.num_pages().await?;
    let logs = paginator.fetch_page(validated.page - 1).await?;
    let log_responses: Vec<AccessLogResponse> = logs.into_iter().map(Into::into).collect();

    Ok(Json(Paginated::new(
        log_responses,
        total,
        validated.page,
        validated.per_page,
        total_pages,
    )))
}

async fn clear_logs(
    State(state): State<AccessLogState>,
    _user: AuthenticatedUser,
) -> AppResult<StatusCode> {
    AccessLog::delete_many().exec(&state.db).await?;
    Ok(StatusCode::NO_CONTENT)
}
