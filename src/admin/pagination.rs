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

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

fn default_page() -> u64 {
    1
}

fn default_per_page() -> u64 {
    100
}

impl PaginationParams {
    /// Validate and normalize pagination parameters
    /// - page: minimum 1
    /// - per_page: minimum 1, maximum 500
    pub fn validate(&self) -> ValidatedPagination {
        ValidatedPagination {
            page: self.page.max(1),
            per_page: self.per_page.clamp(1, 500),
        }
    }
}

pub struct ValidatedPagination {
    pub page: u64,
    pub per_page: u64,
}

#[derive(Serialize)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

impl<T> Paginated<T> {
    pub fn new(data: Vec<T>, total: u64, page: u64, per_page: u64, total_pages: u64) -> Self {
        Self {
            data,
            total,
            page,
            per_page,
            total_pages,
        }
    }
}
