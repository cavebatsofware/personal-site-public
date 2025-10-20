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

use crate::entities::{admin_user, AdminUser};
use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum_login::{AuthUser, AuthnBackend, UserId};
use chrono::Utc;
use rand::Rng;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::{env, fmt};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUserAuth {
    pub id: Uuid,
    pub email: String,
    pub email_verified: bool,
}

impl AuthUser for AdminUserAuth {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.email.as_bytes()
    }
}

#[derive(Clone)]
pub struct AdminAuthBackend {
    db: DatabaseConnection,
    allowed_domain: String,
}

impl AdminAuthBackend {
    pub fn new(db: DatabaseConnection) -> Self {
        let allowed_domain = env::var("SITE_DOMAIN").unwrap();

        Self { db, allowed_domain }
    }

    pub async fn create_admin(
        &self,
        email: &str,
        password: &str,
    ) -> Result<(admin_user::Model, String)> {
        // Validate email domain
        if !email.ends_with(&format!("@{}", self.allowed_domain)) {
            anyhow::bail!("Email must be from {} domain", self.allowed_domain);
        }

        // Check if user already exists
        let existing = AdminUser::find()
            .filter(admin_user::Column::Email.eq(email))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            anyhow::bail!("Admin user with this email already exists");
        }

        // Hash password
        let password_hash = hash_password(password)?;

        // Generate verification token
        let verification_token = generate_verification_token();
        let verification_expires = Utc::now() + chrono::Duration::hours(24);

        let admin = admin_user::ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set(email.to_string()),
            password_hash: Set(password_hash),
            email_verified: Set(false),
            verification_token: Set(Some(verification_token.clone())),
            verification_token_expires_at: Set(Some(verification_expires.into())),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };

        let result = admin.insert(&self.db).await?;

        Ok((result, verification_token))
    }

    pub async fn verify_email(&self, token: &str) -> Result<admin_user::Model> {
        let admin = AdminUser::find()
            .filter(admin_user::Column::VerificationToken.eq(token))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Invalid verification token"))?;

        // Check if token is expired
        if let Some(expires_at) = admin.verification_token_expires_at {
            if Utc::now() > expires_at.with_timezone(&Utc) {
                anyhow::bail!("Verification token has expired");
            }
        } else {
            anyhow::bail!("No verification token expiration set");
        }

        // Mark as verified
        let mut admin_active: admin_user::ActiveModel = admin.into();
        admin_active.email_verified = Set(true);
        admin_active.verification_token = Set(None);
        admin_active.verification_token_expires_at = Set(None);
        admin_active.updated_at = Set(Utc::now().into());

        let updated = admin_active.update(&self.db).await?;

        Ok(updated)
    }
}

#[derive(Debug)]
pub struct AuthError(anyhow::Error);

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for AuthError {}

impl From<anyhow::Error> for AuthError {
    fn from(err: anyhow::Error) -> Self {
        AuthError(err)
    }
}

impl From<sea_orm::DbErr> for AuthError {
    fn from(err: sea_orm::DbErr) -> Self {
        AuthError(err.into())
    }
}

impl AuthnBackend for AdminAuthBackend {
    type User = AdminUserAuth;
    type Credentials = Credentials;
    type Error = AuthError;

    fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> impl std::future::Future<Output = Result<Option<Self::User>, Self::Error>> + Send {
        let db = self.db.clone();
        async move {
            let admin = AdminUser::find()
                .filter(admin_user::Column::Email.eq(&creds.email))
                .one(&db)
                .await
                .map_err(AuthError::from)?;

            let Some(admin) = admin else {
                return Ok(None);
            };

            // Verify password
            let valid =
                verify_password(&creds.password, &admin.password_hash).map_err(AuthError::from)?;
            if !valid {
                return Ok(None);
            }

            // Check if email is verified
            if !admin.email_verified {
                return Err(AuthError(anyhow::anyhow!(
                    "Email not verified. Please check your email for verification link."
                )));
            }

            Ok(Some(AdminUserAuth {
                id: admin.id,
                email: admin.email,
                email_verified: admin.email_verified,
            }))
        }
    }

    fn get_user(
        &self,
        user_id: &UserId<Self>,
    ) -> impl std::future::Future<Output = Result<Option<Self::User>, Self::Error>> + Send {
        let user_id = *user_id;
        let db = self.db.clone();
        async move {
            let admin = AdminUser::find_by_id(user_id)
                .one(&db)
                .await
                .map_err(AuthError::from)?;

            Ok(admin.map(|a| AdminUserAuth {
                id: a.id,
                email: a.email,
                email_verified: a.email_verified,
            }))
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
        .to_string();
    Ok(password_hash)
}

fn verify_password(password: &str, password_hash: &str) -> Result<bool> {
    let parsed_hash =
        PasswordHash::new(password_hash).map_err(|e| anyhow::anyhow!("Invalid hash: {}", e))?;
    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

fn generate_verification_token() -> String {
    let token_bytes: [u8; 32] = rand::thread_rng().gen();
    hex::encode(token_bytes)
}
