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

use axum::{
    extract::ConnectInfo,
    http::{HeaderMap, Request},
    middleware::Next,
    response::Response,
};
use std::net::{IpAddr, SocketAddr};

/// Security context extracted from the request
/// This is stored in request extensions for use by other middleware and handlers
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub ip_address: IpAddr,
    pub user_agent: Option<String>,
}

impl SecurityContext {
    pub fn new(ip_address: IpAddr, user_agent: Option<String>) -> Self {
        Self {
            ip_address,
            user_agent,
        }
    }
}

/// Security middleware that extracts IP address, user agent, and other security-relevant information
/// This runs early in the middleware stack to provide context for subsequent middleware
pub async fn security_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let ip_address = extract_client_ip(&headers, addr.ip());

    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| sanitize_user_agent(s));

    // Log all headers for debugging
    tracing::debug!(
        "Incoming request: method={} uri={} ip={} user_agent={:?}",
        request.method(),
        request.uri(),
        ip_address,
        user_agent
    );

    tracing::debug!("Request headers:");
    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            tracing::debug!("  {}: {}", name, value_str);
        } else {
            tracing::debug!("  {}: <non-UTF8 value>", name);
        }
    }

    let security_context = SecurityContext::new(ip_address, user_agent);

    request.extensions_mut().insert(security_context);

    next.run(request).await
}

fn extract_client_ip(headers: &HeaderMap, fallback_ip: IpAddr) -> IpAddr {
    if let Some(forwarded_for) = headers.get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse::<IpAddr>() {
                    tracing::debug!("Using X-Forwarded-For IP: {}", ip);
                    return ip;
                }
            }
        }
    }

    tracing::debug!("Using socket IP (no proxy headers): {}", fallback_ip);
    fallback_ip
}

fn sanitize_user_agent(user_agent: &str) -> String {
    const MAX_LENGTH: usize = 500;

    let sanitized: String = user_agent
        .chars()
        .filter(|c| !c.is_control() || *c == ' ' || *c == '\t')
        .take(MAX_LENGTH)
        .collect();

    sanitized
}
