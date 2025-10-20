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

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop mac_address column as it's not available from HTTP requests
        manager
            .alter_table(
                Table::alter()
                    .table(AccessLog::Table)
                    .drop_column(AccessLog::MacAddress)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Re-add mac_address column if rolling back
        manager
            .alter_table(
                Table::alter()
                    .table(AccessLog::Table)
                    .add_column(ColumnDef::new(AccessLog::MacAddress).string())
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum AccessLog {
    Table,
    MacAddress,
}
