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

pub use sea_orm_migration::prelude::*;

mod m20250119_000001_create_access_log;
mod m20250120_000001_drop_mac_address;
mod m20250121_000001_create_admin_users;
mod m20250122_000001_create_access_codes;
mod m20250123_000001_add_usage_count;
mod m20250124_000001_create_settings;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250119_000001_create_access_log::Migration),
            Box::new(m20250120_000001_drop_mac_address::Migration),
            Box::new(m20250121_000001_create_admin_users::Migration),
            Box::new(m20250122_000001_create_access_codes::Migration),
            Box::new(m20250123_000001_add_usage_count::Migration),
            Box::new(m20250124_000001_create_settings::Migration),
        ]
    }
}
