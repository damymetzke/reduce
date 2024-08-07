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

use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};

use crate::middleware::require_authentication::require_authentication;

use self::handler::{delete_item, get_index, patch_item, post_complete, post_index};

use super::SectionRegistration;

mod database;
mod handler;
mod templates;

pub fn register() -> SectionRegistration {
    let router = Router::new()
        .route("/upkeep", get(get_index).post(post_index))
        .route("/upkeep/complete/:id", post(post_complete))
        .route("/upkeep/:id", delete(delete_item).patch(patch_item))
        .layer(middleware::from_fn(require_authentication));

    SectionRegistration {
        router,
        entry_page: "/upkeep",
        title: "Upkeep",
    }
}
