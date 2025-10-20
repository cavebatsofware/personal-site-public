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
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::entities::{setting, Setting};

#[derive(Debug, Clone)]
pub struct SettingsService {
    db: DatabaseConnection,
}

impl SettingsService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Get a setting value by key, category, and optional entity_id
    pub async fn get(
        &self,
        key: &str,
        category: Option<&str>,
        entity_id: Option<Uuid>,
    ) -> Result<Option<String>> {
        let mut query = Setting::find().filter(setting::Column::Key.eq(key));

        if let Some(cat) = category {
            query = query.filter(setting::Column::Category.eq(cat));
        } else {
            query = query.filter(setting::Column::Category.is_null());
        }

        if let Some(eid) = entity_id {
            query = query.filter(setting::Column::EntityId.eq(eid));
        } else {
            query = query.filter(setting::Column::EntityId.is_null());
        }

        let setting = query.one(&self.db).await?;
        Ok(setting.map(|s| s.value))
    }

    /// Get a boolean setting value
    pub async fn get_bool(
        &self,
        key: &str,
        category: Option<&str>,
        entity_id: Option<Uuid>,
    ) -> Result<bool> {
        let value = self.get(key, category, entity_id).await?;
        Ok(value.map(|v| v == "true").unwrap_or(false))
    }

    /// Set a setting value, creating it if it doesn't exist
    pub async fn set(
        &self,
        key: &str,
        value: &str,
        category: Option<&str>,
        entity_id: Option<Uuid>,
    ) -> Result<()> {
        // Try to find existing setting
        let mut query = Setting::find().filter(setting::Column::Key.eq(key));

        if let Some(cat) = category {
            query = query.filter(setting::Column::Category.eq(cat));
        } else {
            query = query.filter(setting::Column::Category.is_null());
        }

        if let Some(eid) = entity_id {
            query = query.filter(setting::Column::EntityId.eq(eid));
        } else {
            query = query.filter(setting::Column::EntityId.is_null());
        }

        let existing = query.one(&self.db).await?;

        if let Some(existing_setting) = existing {
            // Update existing
            let mut active: setting::ActiveModel = existing_setting.into();
            active.value = Set(value.to_string());
            active.updated_at = Set(chrono::Utc::now().into());
            active.update(&self.db).await?;
        } else {
            // Create new
            let new_setting = setting::ActiveModel {
                id: Set(Uuid::new_v4()),
                key: Set(key.to_string()),
                value: Set(value.to_string()),
                category: Set(category.map(|s| s.to_string())),
                entity_id: Set(entity_id),
                created_at: Set(chrono::Utc::now().into()),
                updated_at: Set(chrono::Utc::now().into()),
            };
            new_setting.insert(&self.db).await?;
        }

        Ok(())
    }

    /// Get all settings
    pub async fn get_all(&self) -> Result<Vec<setting::Model>> {
        let settings = Setting::find().all(&self.db).await?;
        Ok(settings)
    }
}
