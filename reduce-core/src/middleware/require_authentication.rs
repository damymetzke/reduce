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

use std::sync::Arc;

use askama_axum::IntoResponse;
use axum::{extract::Request, middleware::Next, response::Response};

use crate::{extensions::Session, middleware::require_authentication::templates::{
    ServerErrorTemplate, UnauthorizedTemplate,
}};

#[derive(Clone, Debug)]
pub struct AuthorizedSession {
    pub csrf_token: Arc<str>,
    pub session_id: i32,
}

pub async fn require_authentication(mut req: Request, next: Next) -> Response {
    let extensions = req.extensions_mut();
    let authentication_status: Option<&Session> = extensions.get();

    let authentication_status = if let Some(authentication_status) = authentication_status {
        authentication_status
    } else {
        return ServerErrorTemplate.into_response();
    };

    match authentication_status {
        Session::Authenticated {
            csrf_token,
            session_id,
        } => {
            extensions.insert(AuthorizedSession {
                csrf_token: csrf_token.clone(),
                session_id: *session_id,
            });
            next.run(req).await
        }
        _ => UnauthorizedTemplate {
            session: authentication_status.clone(),
        }
        .into_response(),
    }
}
