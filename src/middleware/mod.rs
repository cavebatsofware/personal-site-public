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

// Middleware module for cross-cutting concerns
// Handles security, rate limiting, and logging in a centralized way

pub mod access_log;
pub mod admin_auth;
pub mod rate_limit;
pub mod security;

pub use access_log::access_log_middleware;
pub use admin_auth::{require_admin_auth, AuthenticatedUser};
pub use rate_limit::rate_limit_middleware;
pub use security::security_middleware;
