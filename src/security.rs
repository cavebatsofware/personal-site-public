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

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};

use std::net::IpAddr;
use std::sync::Arc;
use uuid::Uuid;

use crate::entities::{access_log, AccessLog};

#[derive(Debug, Clone)]
pub struct RateLimitEntry {
    pub count: u32,
    pub first_attempt: DateTime<Utc>,
    pub last_attempt: DateTime<Utc>,
    pub blocked_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Max requests per minute before blocking (abuse detection)
    pub rate_limit_per_minute: u32,
    /// How long to block IPs that exceed rate limit (in minutes)
    pub block_duration_minutes: i64,
    /// Enable access logging to database
    pub enable_logging: bool,
    /// Log successful access attempts (for tracking)
    pub log_successful_attempts: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            rate_limit_per_minute: 30,
            block_duration_minutes: 15,
            enable_logging: true,
            log_successful_attempts: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SecurityService {
    rate_limit_cache: Arc<DashMap<String, RateLimitEntry>>,
    pub config: SecurityConfig,
    db: DatabaseConnection,
}

impl SecurityService {
    pub fn new(db: DatabaseConnection, config: Option<SecurityConfig>) -> Self {
        Self {
            rate_limit_cache: Arc::new(DashMap::new()),
            config: config.unwrap_or_default(),
            db,
        }
    }

    /// Check if IP is rate limited
    /// Returns: (is_allowed, newly_blocked)
    /// - is_allowed: true if request should proceed, false if blocked
    /// - newly_blocked: true if this request triggered the block, false if already blocked
    pub async fn check_rate_limit(&self, ip: IpAddr, _access_code: &str) -> Result<(bool, bool)> {
        let ip_key = ip.to_string();
        let now = Utc::now();

        // Check if currently blocked
        // blocked IPs don't update cache, preventing memory bloat
        if let Some(entry) = self.rate_limit_cache.get(&ip_key) {
            if let Some(blocked_until) = entry.blocked_until {
                if now < blocked_until {
                    // Already blocked - return immediately without updating cache
                    return Ok((false, false));
                }
            }
        }

        // Update rate limit counters
        let mut newly_blocked = false;

        self.rate_limit_cache
            .entry(ip_key.clone())
            .and_modify(|entry| {
                // Reset counters if enough time has passed
                // this could wait for the entries to be cleared at the 10 minute mark
                // but this operation seems efficient enough to run now
                if now.signed_duration_since(entry.first_attempt) > Duration::minutes(10) {
                    entry.count = 1;
                    entry.first_attempt = now;
                } else {
                    entry.count += 1;
                }
                entry.last_attempt = now;

                // Check if we should block
                if entry.count > self.config.rate_limit_per_minute {
                    entry.blocked_until =
                        Some(now + Duration::minutes(self.config.block_duration_minutes));
                    newly_blocked = true;
                    tracing::info!(
                        "IP exceeded rate limit and is now blocked: {} (count: {})",
                        ip,
                        entry.count
                    );
                }
            })
            .or_insert_with(|| RateLimitEntry {
                count: 1,
                first_attempt: now,
                last_attempt: now,
                blocked_until: None,
            });

        Ok((!newly_blocked, newly_blocked))
    }

    pub async fn log_access_attempt(
        &self,
        ip: Option<IpAddr>,
        user_agent: Option<String>,
        access_code: &str,
        action: &str,
        success: bool,
    ) -> Result<()> {
        // Skip logging if disabled entirely
        if !self.config.enable_logging {
            return Ok(());
        }

        // Skip successful attempts if configured not to log them
        if success && !self.config.log_successful_attempts {
            return Ok(());
        }

        tracing::info!(
            "Logging access attempt: ip={:?} action={} code={} success={}",
            ip,
            action,
            access_code,
            success
        );

        let ip_string = ip.map(|ip| ip.to_string());
        let now = Utc::now();

        let (last_delta, current_count) = if let Some(ref ip_str) = ip_string {
            let last_access = AccessLog::find()
                .filter(access_log::Column::IpAddress.eq(ip_str))
                .order_by_desc(access_log::Column::CreatedAt)
                .one(&self.db)
                .await?;

            if let Some(last) = last_access {
                let delta = now.signed_duration_since(last.created_at.with_timezone(&Utc));
                let delta_ms = Some(delta.num_milliseconds());
                let next_count = last.count.unwrap_or(0) + 1;
                tracing::debug!(
                    "Access count for IP {}: {} (previous: {}, delta: {}ms)",
                    ip_str,
                    next_count,
                    last.count.unwrap_or(0),
                    delta.num_milliseconds()
                );
                (delta_ms, next_count)
            } else {
                tracing::debug!("First access logged for IP: {}", ip_str);
                (None, 1)
            }
        } else {
            (None, 1)
        };

        let access_log = access_log::ActiveModel {
            id: Set(Uuid::new_v4()),
            access_code: Set(access_code.to_string()),
            ip_address: Set(ip_string),
            user_agent: Set(user_agent),
            count: Set(Some(current_count)),
            last_access_time: Set(Some(now.into())),
            last_delta_access: Set(last_delta),
            action: Set(action.to_string()),
            success: Set(success),
            created_at: Set(now.into()),
        };

        access_log
            .insert(&self.db)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to log access attempt: {}", e))?;

        Ok(())
    }

    pub async fn cleanup_old_entries(&self) -> Result<()> {
        // Configurable retention period (default: 30 days)
        let retention_days = std::env::var("ACCESS_LOG_RETENTION_DAYS")
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(30);

        let cutoff = Utc::now() - Duration::days(retention_days);

        let delete_result = AccessLog::delete_many()
            .filter(access_log::Column::CreatedAt.lt(cutoff))
            .exec(&self.db)
            .await?;

        tracing::info!(
            "Cleaned up {} old access log entries from database",
            delete_result.rows_affected
        );

        // Clean up in-memory rate limit cache
        // Remove entries that haven't been accessed in 2x the block duration
        // This prevents memory leaks while allowing blocks to persist their full duration
        let now = Utc::now();
        let cache_retention = Duration::minutes(self.config.block_duration_minutes * 2);

        let before_count = self.rate_limit_cache.len();
        self.rate_limit_cache
            .retain(|_, entry| now.signed_duration_since(entry.last_attempt) < cache_retention);
        let after_count = self.rate_limit_cache.len();

        if before_count > after_count {
            tracing::info!(
                "Cleaned up {} old rate limit cache entries ({} -> {} entries)",
                before_count - after_count,
                before_count,
                after_count
            );
        }

        Ok(())
    }
}
