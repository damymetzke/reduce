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

use askama_axum::IntoResponse;
use axum::{routing::get, Router};

use crate::{error::AppResult, template_extend::NavigationLink};

use self::templates::LoginTemplate;

use super::SectionRegistration;

pub async fn get_login() -> AppResult<impl IntoResponse> {
    Ok(LoginTemplate)
}

pub fn register() -> SectionRegistration {
    let router = Router::new().route("/login", get(get_login));

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
