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
use aws_sdk_s3::Client;
use std::env;

#[derive(Clone)]
pub struct S3Service {
    client: Client,
    bucket_name: String,
}

impl S3Service {
    pub async fn new() -> Result<Self> {
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .load()
            .await;
        let client = Client::new(&config);
        let bucket_name =
            env::var("S3_BUCKET_NAME").unwrap_or_else(|_| "personal-site-resumes".to_string());

        Ok(Self {
            client,
            bucket_name,
        })
    }

    /// Fetch a file from S3 at path: {code}/{filename}
    /// For example: get_file("ABC123", "index.html") fetches s3://bucket/ABC123/index.html
    pub async fn get_file(&self, code: &str, filename: &str) -> Result<Vec<u8>> {
        let key = format!("{}/{}", code, filename);

        tracing::info!("Fetching from S3: bucket={}, key={}", self.bucket_name, key);

        let response = self
            .client
            .get_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .send()
            .await?;

        let data = response.body.collect().await?;
        let bytes = data.into_bytes().to_vec();

        tracing::info!("Successfully fetched {} bytes from S3", bytes.len());
        Ok(bytes)
    }

    /// Check if a file exists in S3
    pub async fn file_exists(&self, code: &str, filename: &str) -> bool {
        let key = format!("{}/{}", code, filename);

        match self
            .client
            .head_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .send()
            .await
        {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}
