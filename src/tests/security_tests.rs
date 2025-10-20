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

use crate::security::{SecurityConfig, SecurityService};
use crate::tests::{cleanup_test_db, setup_test_db};
use serial_test::serial;
use std::net::IpAddr;

#[tokio::test]
#[serial]
async fn test_rate_limiting_basic() {
    let db = setup_test_db().await;

    let config = SecurityConfig {
        rate_limit_per_minute: 3,
        block_duration_minutes: 1,
        enable_logging: false, // Disable logging for this test
        log_successful_attempts: false,
    };

    let security = SecurityService::new(db.clone(), Some(config));
    let test_ip: IpAddr = "127.0.0.1".parse().unwrap();
    let test_code = "test-code";

    // First 3 requests should pass
    for i in 1..=3 {
        let result = security.check_rate_limit(test_ip, test_code).await;
        assert!(result.is_ok(), "Request {} should pass rate limit", i);
        let (allowed, newly_blocked) = result.unwrap();
        assert!(allowed, "Request {} should be allowed", i);
        assert!(!newly_blocked, "Request {} should not trigger block", i);
    }

    // 4th request should be blocked
    let result = security.check_rate_limit(test_ip, test_code).await;
    assert!(result.is_ok(), "Rate limit check should not error");
    let (allowed, newly_blocked) = result.unwrap();
    assert!(!allowed, "4th request should be blocked");
    assert!(newly_blocked, "4th request should trigger new block");

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
    let ip2: IpAddr = "192.168.1.1".parse().unwrap();
    let test_code = "test-code";

    // Use up rate limit for ip1
    for _ in 1..=3 {
        let _ = security.check_rate_limit(ip1, test_code).await;
    }

    // ip1 should be blocked, but ip2 should still work
    let result1 = security.check_rate_limit(ip1, test_code).await;
    let (allowed, _) = result1.unwrap();
    assert!(!allowed, "IP1 should be blocked");

    let result2 = security.check_rate_limit(ip2, test_code).await;
    let (allowed, _) = result2.unwrap();
    assert!(allowed, "IP2 should still be allowed");

    cleanup_test_db(&db).await;
}

#[tokio::test]
#[serial]
async fn test_access_logging() {
    let db = setup_test_db().await;

    let config = SecurityConfig {
        enable_logging: true,
        log_successful_attempts: true,
        ..Default::default()
    };

    let security = SecurityService::new(db.clone(), Some(config));
    let test_ip: IpAddr = "127.0.0.1".parse().unwrap();

    // Log a successful access
    let result = security
        .log_access_attempt(
            Some(test_ip),
            Some("test-user-agent".to_string()),
            "test-code",
            "view",
            true,
        )
        .await;

    assert!(result.is_ok(), "Logging should succeed");

    // Give async logging time to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify log was created (we could query the database here)
    // For now, just ensure no errors occurred

    cleanup_test_db(&db).await;
}

#[tokio::test]
#[serial]
async fn test_cleanup_old_entries() {
    let db = setup_test_db().await;

    let config = SecurityConfig {
        enable_logging: true,
        ..Default::default()
    };

    let security = SecurityService::new(db.clone(), Some(config));

    // Create some test entries
    let test_ip: IpAddr = "127.0.0.1".parse().unwrap();
    for i in 0..5 {
        let _ = security
            .log_access_attempt(
                Some(test_ip),
                Some("test-user-agent".to_string()),
                &format!("test-code-{}", i),
                "view",
                true,
            )
            .await;
    }

    // Give async logging time to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Test cleanup (won't delete recent entries, but shouldn't error)
    let result = security.cleanup_old_entries().await;
    assert!(result.is_ok(), "Cleanup should succeed");

    cleanup_test_db(&db).await;
}

#[tokio::test]
#[serial]
async fn test_disabled_logging() {
    let db = setup_test_db().await;

    let config = SecurityConfig {
        enable_logging: false,
        ..Default::default()
    };

    let security = SecurityService::new(db.clone(), Some(config));
    let test_ip: IpAddr = "127.0.0.1".parse().unwrap();

    // Should succeed even with logging disabled
    let result = security
        .log_access_attempt(
            Some(test_ip),
            Some("test-user-agent".to_string()),
            "test-code",
            "view",
            true,
        )
        .await;

    assert!(
        result.is_ok(),
        "Logging should succeed (no-op when disabled)"
    );

    cleanup_test_db(&db).await;
}
