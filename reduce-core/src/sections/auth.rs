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
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use askama_axum::IntoResponse;
use axum::{http::HeaderMap, routing::get, Extension, Form, Router};
use serde::Deserialize;
use sqlx::{Pool, Postgres};

use crate::{error::AppResult, template_extend::NavigationLink};

use self::{database::fetch_account, templates::LoginTemplate};

use super::SectionRegistration;

pub async fn get_login() -> AppResult<impl IntoResponse> {
    Ok(LoginTemplate)
}

#[derive(Deserialize)]
pub struct PostLoginForm {
    email: Arc<str>,
    password: Arc<str>,
}

pub async fn post_login(
    Extension(pool): Extension<Pool<Postgres>>,
    Form(PostLoginForm { email, password }): Form<PostLoginForm>,
) -> AppResult<impl IntoResponse> {
    let account = fetch_account(&pool, &email).await?;

    let result = Argon2::default().verify_password(
        password.as_bytes(),
        &PasswordHash::new(&account.password_hash)
            .map_err(|error| anyhow!("Error with generating hash: {:?}", error))?,
    );

    let mut redirect_headers = HeaderMap::new();
    redirect_headers.insert("HX-Location", "/".parse()?);
    let none_headers = HeaderMap::new();

    match result {
        Ok(_) => Ok((redirect_headers, LoginTemplate)),
        Err(_) => Ok((none_headers, LoginTemplate)),
    }

}

pub fn register() -> SectionRegistration {
    let router = Router::new().route("/login", get(get_login).post(post_login));

    let navigation_links = Box::from([NavigationLink {
        href: "/core/auth/login".into(),
        title: "Login".into(),
    }]);
    SectionRegistration {
        default_section_name: "/auth",
        router,
        navigation_links,
    }
}
