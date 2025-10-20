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
use crate::s3::S3Service;
use crate::security::SecurityService;
use crate::settings::SettingsService;
use anyhow::Result;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::env;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub security: SecurityService,
    pub settings: SettingsService,
    pub s3: S3Service,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        // Establish database connection (migrations run separately via MIGRATE_DB=true)
        let db = crate::database::establish_connection()
            .await
            .map_err(|e| anyhow::anyhow!("Database connection failed: {}", e))?;

        // Create security service with configurable settings
        let security_config = crate::security::SecurityConfig {
            rate_limit_per_minute: env::var("RATE_LIMIT_PER_MINUTE")
                .unwrap_or_default()
                .parse()
                .unwrap(),
            block_duration_minutes: env::var("BLOCK_DURATION_MINUTES")
                .unwrap_or_default()
                .parse()
                .unwrap(),
            enable_logging: env::var("ENABLE_ACCESS_LOGGING")
                .unwrap_or_default()
                .parse()
                .unwrap(),
            log_successful_attempts: env::var("LOG_SUCCESSFUL_ATTEMPTS")
                .unwrap_or_default()
                .parse()
                .unwrap(),
        };

        let security = SecurityService::new(db.clone(), Some(security_config.clone()));
        let settings = SettingsService::new(db.clone());
        let s3 = S3Service::new().await?;

        // RUST_LOG=warn recommended for most deployments, info and debug generate lots of logs
        tracing::info!("Database connected and services initialized");
        tracing::info!(
            "Security config: rate_limit={}/min, block_duration={}min, logging_enabled={}, log_successful={}",
            security_config.rate_limit_per_minute,
            security_config.block_duration_minutes,
            security_config.enable_logging,
            security_config.log_successful_attempts
        );

        Ok(AppState {
            db,
            security,
            settings,
            s3,
        })
    }

    /// Check if code is valid in database and increment usage count
    pub async fn is_valid_code(&self, code: &str) -> Result<bool> {
        // Check database
        let db_code = AccessCode::find()
            .filter(access_code::Column::Code.eq(code))
            .one(&self.db)
            .await?;

        if let Some(db_code) = db_code {
            // Check if expired
            if let Some(expires_at) = db_code.expires_at {
                if expires_at.with_timezone(&Utc) < Utc::now() {
                    return Ok(false); // Expired
                }
            }

            // Increment usage count
            let mut active_code: access_code::ActiveModel = db_code.into();
            active_code.usage_count = Set(active_code.usage_count.unwrap() + 1);
            active_code.update(&self.db).await?;

            return Ok(true);
        }

        Ok(false)
    }
}
