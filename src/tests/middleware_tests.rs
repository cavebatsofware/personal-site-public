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

use crate::middleware::security::SecurityContext;
use crate::security::{SecurityConfig, SecurityService};
use crate::tests::{cleanup_test_db, setup_test_db};
use serial_test::serial;
use std::net::IpAddr;

#[tokio::test]
#[serial]
async fn test_security_context_creation() {
    let ip: IpAddr = "192.168.1.1".parse().unwrap();
    let user_agent = Some("test-agent".to_string());

    let context = SecurityContext::new(ip, user_agent.clone());

    assert_eq!(context.ip_address, ip);
    assert_eq!(context.user_agent, user_agent);
}

#[tokio::test]
#[serial]
async fn test_rate_limiting_basic() {
    let db = setup_test_db().await;

    let config = SecurityConfig {
        rate_limit_per_minute: 2,
        block_duration_minutes: 1,
        enable_logging: false,
        log_successful_attempts: false,
    };

    let security = SecurityService::new(db.clone(), Some(config));
    let test_ip: IpAddr = "127.0.0.1".parse().unwrap();

    // First request should pass
    let result1 = security.check_rate_limit(test_ip, "test-key").await;
    assert!(result1.is_ok());
    let (allowed, newly_blocked) = result1.unwrap();
    assert!(allowed);
    assert!(!newly_blocked);

    // Second request should pass
    let result2 = security.check_rate_limit(test_ip, "test-key").await;
    assert!(result2.is_ok());
    let (allowed, newly_blocked) = result2.unwrap();
    assert!(allowed);
    assert!(!newly_blocked);

    // Third request should be blocked (threshold is 2)
    let result3 = security.check_rate_limit(test_ip, "test-key").await;
    assert!(result3.is_ok());
    let (allowed, newly_blocked) = result3.unwrap();
    assert!(!allowed); // Should be blocked
    assert!(newly_blocked); // This is the first time being blocked

    cleanup_test_db(&db).await;
}

#[tokio::test]
#[serial]
async fn test_rate_limiting_different_ips() {
    let db = setup_test_db().await;

    let config = SecurityConfig {
        rate_limit_per_minute: 2,
        block_duration_minutes: 1,
        enable_logging: false,
        log_successful_attempts: false,
    };

    let security = SecurityService::new(db.clone(), Some(config));
    let ip1: IpAddr = "127.0.0.1".parse().unwrap();
    let ip2: IpAddr = "127.0.0.2".parse().unwrap();

    // IP1: Use up its limit
    security.check_rate_limit(ip1, "test-key").await.unwrap();
    security.check_rate_limit(ip1, "test-key").await.unwrap();
    let (allowed, _newly_blocked) = security.check_rate_limit(ip1, "test-key").await.unwrap();
    assert!(!allowed); // IP1 should be blocked

    // IP2: Should still work independently
    let (allowed, _newly_blocked) = security.check_rate_limit(ip2, "test-key").await.unwrap();
    assert!(allowed); // IP2 should still be allowed

    cleanup_test_db(&db).await;
}

#[tokio::test]
#[serial]
async fn test_access_logging_disabled() {
    let db = setup_test_db().await;

    let config = SecurityConfig {
        enable_logging: false,
        ..Default::default()
    };

    let security = SecurityService::new(db.clone(), Some(config));
    let test_ip: IpAddr = "127.0.0.1".parse().unwrap();

    // Should not error even with logging disabled
    let result = security
        .log_access_attempt(
            Some(test_ip),
            Some("test-agent".to_string()),
            "test-code",
            "test-action",
            true,
        )
        .await;

    assert!(result.is_ok());

    cleanup_test_db(&db).await;
}

#[tokio::test]
#[serial]
async fn test_security_config_defaults() {
    let config = SecurityConfig::default();

    assert_eq!(config.rate_limit_per_minute, 30);
    assert_eq!(config.block_duration_minutes, 15);
    assert!(config.enable_logging);
    assert!(config.log_successful_attempts);
}
