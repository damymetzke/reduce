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

mod templates;

use std::rc::Rc;

use askama::DynTemplate;
use axum::{routing::get, Extension, Router};
use templates::{CurrentEmailPasswordPartTemplate, IndexTemplate, NewEmailPasswordPartTemplate};

use crate::extensions::Session;

use super::SectionRegistration;

async fn get_index(Extension(session): Extension<Session>) -> IndexTemplate {
    let current_methods = Rc::new([Box::new(CurrentEmailPasswordPartTemplate {
        email: "user@example.com".into(),
    }) as Box<dyn DynTemplate>]);

    let new_methods = Rc::new([Box::new(NewEmailPasswordPartTemplate) as Box<dyn DynTemplate>]);

    IndexTemplate {
        session,
        current_methods,
        new_methods,
    }
}

pub fn register() -> SectionRegistration {
    let router = Router::new().route("/", get(get_index));

    let navigation_links = Box::from([]);
    SectionRegistration {
        default_section_name: "/account",
        router,
        navigation_links,
    }
}
