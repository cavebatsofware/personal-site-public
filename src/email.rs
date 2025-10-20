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
use aws_sdk_sesv2::{
    types::{Body, Content, Destination, EmailContent, Message},
    Client as SesClient,
};
use std::env;

#[derive(Clone)]
pub struct EmailService {
    client: SesClient,
    from_email: String,
    site_url: String,
}

impl EmailService {
    pub async fn new() -> Result<Self> {
        let mut config_loader = aws_config::defaults(aws_config::BehaviorVersion::latest());

        // Override region if AWS_REGION is set in environment
        if let Ok(region) = env::var("AWS_REGION") {
            config_loader = config_loader.region(aws_sdk_sesv2::config::Region::new(region));
        }

        let config = config_loader.load().await;
        let client = SesClient::new(&config);

        let from_email = env::var("AWS_SES_FROM_EMAIL")
            .unwrap_or_else(|_| "noreply@cavebatsoftware.com".to_string());

        let site_url = env::var("SITE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

        Ok(Self {
            client,
            from_email,
            site_url,
        })
    }

    pub async fn send_verification_email(
        &self,
        to_email: &str,
        verification_token: &str,
    ) -> Result<()> {
        let verification_url = format!(
            "{}/admin/verify-email?token={}",
            self.site_url, verification_token
        );

        let subject = "Verify Your Admin Account";
        let html_body = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Verify Your Email</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px;">
    <div style="background-color: #f4f4f4; border-radius: 5px; padding: 20px; margin-bottom: 20px;">
        <h1 style="color: #2c3e50; margin-top: 0;">Welcome to Cave Bat Software Admin</h1>
        <p>Thank you for registering as an admin user. Please verify your email address to complete your registration.</p>
    </div>

    <div style="background-color: white; border: 1px solid #ddd; border-radius: 5px; padding: 20px; margin-bottom: 20px;">
        <p>Click the button below to verify your email address:</p>
        <div style="text-align: center; margin: 30px 0;">
            <a href="{}"
               style="background-color: #3498db; color: white; padding: 12px 30px; text-decoration: none; border-radius: 5px; display: inline-block; font-weight: bold;">
                Verify Email Address
            </a>
        </div>
        <p style="color: #666; font-size: 14px;">Or copy and paste this link into your browser:</p>
        <p style="word-break: break-all; color: #3498db; font-size: 14px;">{}</p>
    </div>

    <div style="color: #666; font-size: 12px; text-align: center;">
        <p>This verification link will expire in 24 hours.</p>
        <p>If you didn't request this verification email, you can safely ignore it.</p>
    </div>
</body>
</html>
"#,
            verification_url, verification_url
        );

        let text_body = format!(
            r#"
Welcome to Cave Bat Software Admin

Thank you for registering as an admin user. Please verify your email address to complete your registration.

Verification Link: {}

This verification link will expire in 24 hours.

If you didn't request this verification email, you can safely ignore it.
"#,
            verification_url
        );

        let destination = Destination::builder().to_addresses(to_email).build();

        let subject_content = Content::builder().data(subject).charset("UTF-8").build()?;

        let html_content = Content::builder()
            .data(html_body)
            .charset("UTF-8")
            .build()?;

        let text_content = Content::builder()
            .data(text_body)
            .charset("UTF-8")
            .build()?;

        let body = Body::builder()
            .html(html_content)
            .text(text_content)
            .build();

        let message = Message::builder()
            .subject(subject_content)
            .body(body)
            .build();

        let email_content = EmailContent::builder().simple(message).build();

        self.client
            .send_email()
            .from_email_address(&self.from_email)
            .destination(destination)
            .content(email_content)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send verification email: {}", e))?;

        tracing::info!("Verification email sent to {}", to_email);

        Ok(())
    }
}
