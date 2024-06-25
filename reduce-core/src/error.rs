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

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::warn;

use crate::extensions::Session;

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        warn!("Server error: {}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Something went wrong!".to_string(),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub type AppResult<T> = Result<T, AppError>;

pub fn unauthorized_error(session: Session) -> impl IntoResponse {
    (
        StatusCode::UNAUTHORIZED,
        templates::UnauthorizedError { session },
    )
}

pub fn server_error() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, templates::ServerError)
}
