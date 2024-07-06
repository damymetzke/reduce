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

use std::{env, sync::Arc};

use anyhow::{anyhow, Result};
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
use sqlx::{Executor, Pool, Postgres};

use crate::{error::AppResult, extensions::Session};

use self::{
    database::{
        create_session, delete_session, fetch_bootstrap_secret_exists, fetch_email_login_details,
        insert_bootstrap_secret, BootstrapSecretResult,
    },
    templates::{BootstrapTemplate, LoginTemplate},
};

use super::SectionRegistration;

async fn setup_session<'a, T>(pool: T, account_id: i32) -> Result<HeaderMap>
where
    T: Executor<'a, Database = Postgres>,
{
    let mut session_token_bytes = [0u8; 33];
    let mut csrf_token_bytes = [0u8; 18];

    OsRng.fill_bytes(&mut session_token_bytes);
    OsRng.fill_bytes(&mut csrf_token_bytes);

    let session_token = STANDARD.encode(session_token_bytes);
    let csrf_token = STANDARD.encode(csrf_token_bytes);

    let expires_at = Local::now().naive_local() + Duration::days(1);

    create_session(pool, account_id, &session_token, expires_at, &csrf_token).await?;

    let mut redirect_headers = HeaderMap::new();
    redirect_headers.insert("HX-Location", "/".parse()?);
    redirect_headers.insert(
        "Set-Cookie",
        format!("session_token={}; Path=/; HttpOnly; Secure", session_token).parse()?,
    );
    Ok(redirect_headers)
}

pub async fn get_login(Extension(session): Extension<Session>) -> AppResult<impl IntoResponse> {
    Ok(LoginTemplate { session })
}

#[derive(Deserialize)]
pub struct PostLoginForm {
    email: Arc<str>,
    password: Arc<str>,
}

pub async fn post_login(
    Extension(session): Extension<Session>,
    Extension(pool): Extension<Pool<Postgres>>,
    Form(PostLoginForm { email, password }): Form<PostLoginForm>,
) -> AppResult<impl IntoResponse> {
    let account = fetch_email_login_details(&pool, &email).await?;

    let result = Argon2::default().verify_password(
        password.as_bytes(),
        &PasswordHash::new(&account.password_hash)
            .map_err(|error| anyhow!("Error with generating hash: {:?}", error))?,
    );

    let none_headers = HeaderMap::new();

    match result {
        Err(_) => Ok((none_headers, LoginTemplate { session })),
        Ok(_) => {
            let redirect_headers = setup_session(&pool, account.account_id).await?;

            Ok((redirect_headers, LoginTemplate { session }))
        }
    }
}

pub async fn post_logout(
    Extension(session): Extension<Session>,
    Extension(pool): Extension<Pool<Postgres>>,
) -> AppResult<Response<String>> {
    Ok(match session {
        Session::Authenticated { session_id, .. } => {
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

pub async fn get_bootstrap(Extension(session): Extension<Session>) -> impl IntoResponse {
    BootstrapTemplate { session }
}

#[derive(Deserialize)]
pub struct PostBootstrapForm {
    bootstrap_secret: Arc<str>,
}

pub async fn post_bootstrap(
    Extension(session): Extension<Session>,
    Extension(pool): Extension<Pool<Postgres>>,
    Form(PostBootstrapForm { bootstrap_secret }): Form<PostBootstrapForm>,
) -> AppResult<impl IntoResponse> {
    let var = match env::var("REDUCE_BOOTSTRAP_SECRET") {
        Ok(var) => var,
        Err(e) => return Err(e.into()),
    };

    if *var != *bootstrap_secret {
        return Err(anyhow!("Given secret does not match environment secret").into());
    };

    if fetch_bootstrap_secret_exists(&pool, &bootstrap_secret).await? {
        return Err(anyhow!("You cannot reuse a secret").into());
    };

    let BootstrapSecretResult { account_id } =
        insert_bootstrap_secret(&pool, &bootstrap_secret).await?;

    let header_map = setup_session(&pool, account_id).await?;

    Ok((header_map, BootstrapTemplate { session }))
}

pub fn register() -> SectionRegistration {
    let router = Router::new()
        .route("/auth/login", get(get_login).post(post_login))
        .route("/auth/logout", post(post_logout))
        .route("/auth/bootstrap", get(get_bootstrap).post(post_bootstrap));

    SectionRegistration {
        router,
        entry_page: "",
        title: "",
    }
}
