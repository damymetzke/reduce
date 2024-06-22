/*
* Reduce: Improve productivity by reducing complexity
* Copyright (C) 2024  Damy Metzke
*
* This program is free software: you can redistribute it and/or modify
* it under the terms of the GNU Affero General Public License as published by
* the Free Software Foundation, either version 3 of the License, or
* (at your option) any later version.
*
* This program is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
* GNU Affero General Public License for more details.
*
* You should have received a copy of the GNU Affero General Public License
* along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

mod database;
mod templates;

use std::sync::Arc;

use anyhow::anyhow;
use argon2::{
    password_hash::rand_core::{OsRng, RngCore},
    Argon2, PasswordHash, PasswordVerifier,
};
use askama_axum::IntoResponse;
use axum::{
    http::{HeaderMap, Response, StatusCode},
    routing::{get, post},
    Extension, Form, Router,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{Duration, Local};
use serde::Deserialize;
use sqlx::{Pool, Postgres};

use crate::{error::AppResult, middleware::inject_user_authorization::UserAuthenticationStatus};

use self::{
    database::{create_session, delete_session, fetch_account},
    templates::LoginTemplate,
};

use super::SectionRegistration;

pub async fn get_login(
    Extension(session): Extension<UserAuthenticationStatus>,
) -> AppResult<impl IntoResponse> {
    Ok(LoginTemplate { session })
}

#[derive(Deserialize)]
pub struct PostLoginForm {
    email: Arc<str>,
    password: Arc<str>,
}

pub async fn post_login(
    Extension(session): Extension<UserAuthenticationStatus>,
    Extension(pool): Extension<Pool<Postgres>>,
    Form(PostLoginForm { email, password }): Form<PostLoginForm>,
) -> AppResult<impl IntoResponse> {
    let account = fetch_account(&pool, &email).await?;

    let result = Argon2::default().verify_password(
        password.as_bytes(),
        &PasswordHash::new(&account.password_hash)
            .map_err(|error| anyhow!("Error with generating hash: {:?}", error))?,
    );

    let none_headers = HeaderMap::new();

    match result {
        Err(_) => Ok((none_headers, LoginTemplate { session })),
        Ok(_) => {
            let account_id = account.id;
            let mut session_token_bytes = [0u8; 33];
            let mut csrf_token_bytes = [0u8; 18];

            OsRng.fill_bytes(&mut session_token_bytes);
            OsRng.fill_bytes(&mut csrf_token_bytes);

            let session_token = STANDARD.encode(session_token_bytes);
            let csrf_token = STANDARD.encode(csrf_token_bytes);

            let expires_at = Local::now().naive_local() + Duration::days(1);

            create_session(&pool, account_id, &session_token, expires_at, &csrf_token).await?;

            let mut redirect_headers = HeaderMap::new();
            redirect_headers.insert("HX-Location", "/".parse()?);
            redirect_headers.insert(
                "Set-Cookie",
                format!("session_token={}; Path=/; HttpOnly; Secure", session_token).parse()?,
            );

            Ok((redirect_headers, LoginTemplate { session }))
        }
    }
}

pub async fn post_logout(
    Extension(session): Extension<UserAuthenticationStatus>,
    Extension(pool): Extension<Pool<Postgres>>,
) -> AppResult<Response<String>> {
    Ok(match session {
        UserAuthenticationStatus::Authenticated { session_id, .. } => {
            delete_session(&pool, session_id).await?;
            Response::builder()
                .status(StatusCode::FOUND)
                .header("Location", "/")
                .body("Redirecting".into())?
        }
        _ => Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body("Unauthorized access".into())?,
    })
}

pub fn register() -> SectionRegistration {
    let router = Router::new()
        .route("/login", get(get_login).post(post_login))
        .route("/logout", post(post_logout));

    let navigation_links = Box::from([]);
    SectionRegistration {
        default_section_name: "/auth",
        router,
        navigation_links,
    }
}
