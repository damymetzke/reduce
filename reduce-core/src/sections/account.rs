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

use std::rc::Rc;

use askama::DynTemplate;
use axum::{middleware, routing::get, Extension, Router};
use database::fetch_email_for_login;
use sqlx::{Pool, Postgres};
use templates::{CurrentEmailPasswordPartTemplate, IndexTemplate, NewEmailPasswordPartTemplate};

use crate::{
    error::AppResult, extensions::AuthorizedSession,
    middleware::require_authentication::require_authentication,
};

use super::SectionRegistration;

async fn get_index(
    Extension(pool): Extension<Pool<Postgres>>,
    Extension(session): Extension<AuthorizedSession>,
) -> AppResult<IndexTemplate> {
    let (current_methods, new_methods): (Rc<[_]>, Rc<[_]>) =
        match fetch_email_for_login(&pool, session.account_id).await? {
            Some(email) => (
                Rc::new([
                    Box::new(CurrentEmailPasswordPartTemplate { email }) as Box<dyn DynTemplate>
                ]),
                Rc::new([]),
            ),
            None => (
                Rc::new([]),
                Rc::new([Box::new(NewEmailPasswordPartTemplate) as Box<dyn DynTemplate>]),
            ),
        };

    Ok(IndexTemplate {
        session: session.into(),
        current_methods,
        new_methods,
    })
}

pub fn register() -> SectionRegistration {
    let router = Router::new()
        .route("/", get(get_index))
        .layer(middleware::from_fn(require_authentication));

    let navigation_links = Box::from([]);
    SectionRegistration {
        default_section_name: "/account",
        router,
        navigation_links,
    }
}
