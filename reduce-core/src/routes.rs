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

use askama_axum::IntoResponse;
use axum::{http::HeaderMap, routing::get, Router};

use crate::{error::AppResult, IndexTemplate, PartModule};

async fn index() -> IndexTemplate {
    IndexTemplate {
        modules: [
            PartModule {
                href: "/time-reports".into(),
                title: "Time".into(),
            },
            PartModule {
                href: "/upkeep".into(),
                title: "Upkeep".into(),
            },
        ]
        .into(),
    }
}

async fn get_style() -> AppResult<impl IntoResponse> {
    let mut headers = HeaderMap::new();
    headers.insert("Content-type", "text/css".parse()?);

    Ok((
        headers,
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/static/style.css")),
    ))
}

pub fn register(router: Router) -> Router {
    router
        .route("/", get(|| async move { index().await }))
        .route("/static/style.css", get(get_style))
}
