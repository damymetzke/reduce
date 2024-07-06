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

use std::{rc::Rc, sync::Arc};

use anyhow::anyhow;
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use askama::DynTemplate;
use axum::{
    debug_handler, middleware,
    routing::{get, post},
    Extension, Router,
};
use database::{
    fetch_email_for_login, fetch_password_hash_for_login, insert_email_password_login,
    update_password,
};
use rand::rngs::OsRng;
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use templates::{CurrentEmailPasswordPartTemplate, IndexTemplate, NewEmailPasswordPartTemplate};

use crate::{
    error::AppResult, extensions::AuthorizedSession, extractors::csrf_form::CsrfForm,
    middleware::require_authentication::require_authentication,
};

use super::SectionRegistration;

async fn index_template(
    pool: &Pool<Postgres>,
    session: AuthorizedSession,
) -> AppResult<IndexTemplate> {
    let (current_methods, new_methods): (Rc<[_]>, Rc<[_]>) =
        match fetch_email_for_login(pool, session.account_id).await? {
            Some(email) => (
                Rc::new([Box::new(CurrentEmailPasswordPartTemplate {
                    email,
                    authorized_session: session.clone(),
                }) as Box<dyn DynTemplate>]),
                Rc::new([]),
            ),
            None => (
                Rc::new([]),
                Rc::new([Box::new(NewEmailPasswordPartTemplate {
                    authorized_session: session.clone(),
                }) as Box<dyn DynTemplate>]),
            ),
        };

    Ok(IndexTemplate {
        session: session.into(),
        current_methods,
        new_methods,
    })
}

async fn get_index(
    Extension(pool): Extension<Pool<Postgres>>,
    Extension(session): Extension<AuthorizedSession>,
) -> AppResult<IndexTemplate> {
    index_template(&pool, session).await
}

#[derive(Deserialize, Clone)]
struct PostPasswordForm {
    email: Arc<str>,
    password: Arc<str>,
}
async fn post_password(
    Extension(pool): Extension<Pool<Postgres>>,
    Extension(session): Extension<AuthorizedSession>,
    CsrfForm(PostPasswordForm { email, password }): CsrfForm<PostPasswordForm>,
) -> AppResult<IndexTemplate> {
    let argon_context = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon_context
        .hash_password(password.as_bytes(), &salt)
        .map_err(|err| anyhow!("{}", err))?;

    insert_email_password_login(
        &pool,
        session.account_id,
        &email,
        password_hash.to_string().as_str(),
    )
    .await?;
    index_template(&pool, session).await
}

#[derive(Deserialize, Clone)]
struct PutPasswordForm {
    current_password: Arc<str>,
    confirm_current_password: Arc<str>,
    new_password: Arc<str>,
}

#[debug_handler]
async fn put_password(
    Extension(pool): Extension<Pool<Postgres>>,
    Extension(session): Extension<AuthorizedSession>,
    CsrfForm(PutPasswordForm {
        current_password,
        confirm_current_password,
        new_password,
    }): CsrfForm<PutPasswordForm>,
) -> AppResult<IndexTemplate> {
    if current_password != confirm_current_password {
        return Err(anyhow!("Incorrect password").into());
    };

    let password_hash = fetch_password_hash_for_login(&pool, session.account_id).await?;

    let argon_context = Argon2::default();
    let result = argon_context.verify_password(
        current_password.as_bytes(),
        &PasswordHash::new(&password_hash)
            .map_err(|error| anyhow!("Error with generating hash: {:?}", error))?,
    );

    match result {
        Err(_) => Ok(index_template(&pool, session).await?),
        Ok(_) => {
            let salt = SaltString::generate(&mut OsRng);
            let password_hash = argon_context
                .hash_password(new_password.as_bytes(), &salt)
                .map_err(|err| anyhow!("{}", err))?;
            update_password(
                &pool,
                session.account_id,
                password_hash.to_string().as_str(),
            )
            .await?;
            Ok(index_template(&pool, session).await?)
        }
    }
}

pub fn register() -> SectionRegistration {
    let router = Router::new()
        .route("/account", get(get_index))
        .route("/account/password", post(post_password).put(put_password))
        .layer(middleware::from_fn(require_authentication));

    SectionRegistration {
        router,
        entry_page: "/account",
        title: "Account",
    }
}
