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

use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    extract::{rejection::FormRejection, FromRequest, Request},
    response::{IntoResponse, Response},
    Form,
};
use serde::Deserialize;
use thiserror::Error;

use crate::{
    error,
    middleware::{
        inject_user_authorization::UserAuthenticationStatus,
        require_authentication::AuthorizedSession,
    },
};

#[derive(Deserialize)]
struct CsrfTokenInner {
    csrf_token: Arc<str>,
}

#[derive(Deserialize)]
struct CsrfFormInner<T> {
    #[serde(flatten)]
    result: T,
    #[serde(flatten)]
    token: CsrfTokenInner,
}

pub struct CsrfForm<T>(pub T);

#[derive(Error, Debug)]
pub enum CsrfFormRejection {
    #[error(transparent)]
    FormRejection(#[from] FormRejection),
    #[error("Could not authorize user")]
    Unauthorized(UserAuthenticationStatus),
    #[error("Server error")]
    ServerError,
}

impl IntoResponse for CsrfFormRejection {
    fn into_response(self) -> Response {
        match self {
            CsrfFormRejection::FormRejection(rejection) => rejection.into_response(),
            CsrfFormRejection::Unauthorized(session) => {
                error::unauthorized_error(session).into_response()
            }
            CsrfFormRejection::ServerError => error::server_error().into_response(),
        }
    }
}

#[async_trait]
impl<S, T> FromRequest<S> for CsrfForm<T>
where
    S: Send + Sync + 'static,
    Form<CsrfFormInner<T>>: FromRequest<S, Rejection = FormRejection>,
    Form<T>: FromRequest<S, Rejection = FormRejection>,
    T: Clone,
{
    type Rejection = CsrfFormRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        if req.method().is_safe() {
            let extractor = Form::<T>::from_request(req, state).await?;
            return Ok(CsrfForm(extractor.0));
        };

        let session: Option<&AuthorizedSession> = req.extensions().get();

        let session = match session {
            Some(session) => session,
            _ => {
                let session: Option<&UserAuthenticationStatus> = req.extensions().get();
                return match session {
                    Some(session) => Err(CsrfFormRejection::Unauthorized(session.clone())),
                    _ => Err(CsrfFormRejection::ServerError),
                };
            }
        };

        let session = session.clone();

        let extractor = Form::<CsrfFormInner<T>>::from_request(req, state).await?;

        if session.csrf_token != extractor.token.csrf_token {
            Err(CsrfFormRejection::Unauthorized(session.into()))
        } else {
            Ok(CsrfForm(extractor.result.clone()))
        }
    }
}
