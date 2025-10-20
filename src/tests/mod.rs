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

pub mod database_tests;
pub mod middleware_tests;
pub mod security_tests;

use crate::database;
use crate::migration::{Migrator, MigratorTrait};
use sea_orm::DatabaseConnection;

pub async fn setup_test_db() -> DatabaseConnection {
    let db = database::establish_test_connection()
        .await
        .expect("Failed to establish test database connection");

    // Run migrations on test database
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations on test database");

    db
}

pub async fn cleanup_test_db(db: &DatabaseConnection) {
    use crate::entities::AccessLog;
    use sea_orm::EntityTrait;

    // Clean up test data
    AccessLog::delete_many().exec(db).await.ok();
}
