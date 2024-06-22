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

use std::{future::Future, pin::Pin, sync::Arc};

use axum::http::Request;
use axum_extra::extract::CookieJar;
use chrono::{Local, NaiveDateTime};
use sqlx::{query_as, Pool, Postgres};
use tower::{Layer, Service};

#[derive(Clone, Debug)]
pub struct AuthorizeUserService<Inner> {
    inner: Inner,
    pool: Pool<Postgres>,
}

#[derive(Clone, Debug)]
pub enum UserAuthenticationStatus {
    Guest,
    Expired {
        since: NaiveDateTime,
    },
    Session {
        csrf_token: Arc<str>,
        session_id: i32,
    },
}

#[derive(Debug)]
struct SessionData {
    csrf_token: Arc<str>,
    expires_at: NaiveDateTime,
    id: i32,
}

impl<Body, Inner> Service<Request<Body>> for AuthorizeUserService<Inner>
where
    Inner: Service<Request<Body>> + std::marker::Send + Clone + 'static,
    Body: std::marker::Send + 'static,
    Inner::Future: Send + 'static,
    Inner::Response: 'static,
    Inner::Error: 'static,
{
    type Response = Inner::Response;
    type Error = Inner::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        let cookies = CookieJar::from_headers(req.headers());
        let session_token: Option<Box<str>> = cookies
            .get("session_token")
            .map(|token| token.value().into());

        let pool = self.pool.clone();

        let fut = async move {
            if let Some(session_token) = session_token {
                // Example async database query using sqlx
                let result = query_as!{SessionData, "SELECT csrf_token, expires_at, id FROM sessions WHERE session_token = $1", &session_token}
                    .fetch_one(&pool)
                    .await;

                match result {
                    Ok(data) => {
                        let now = Local::now().naive_local();
                        if now >= data.expires_at {
                            req.extensions_mut()
                                .insert(UserAuthenticationStatus::Expired {
                                    since: data.expires_at,
                                })
                        } else {
                            req.extensions_mut()
                                .insert(UserAuthenticationStatus::Session {
                                    csrf_token: data.csrf_token,
                                    session_id: data.id,
                                })
                        }
                    }
                    Err(_) => req.extensions_mut().insert(UserAuthenticationStatus::Guest),
                };
            } else {
                req.extensions_mut().insert(UserAuthenticationStatus::Guest);
            };
            req
        };

        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let req = fut.await;

            inner.call(req).await
        })
    }
}

#[derive(Clone, Debug)]
pub struct AuthorizeUser {
    pub pool: Pool<Postgres>,
}

impl<Inner> Layer<Inner> for AuthorizeUser {
    type Service = AuthorizeUserService<Inner>;

    fn layer(&self, inner: Inner) -> Self::Service {
        AuthorizeUserService {
            inner,
            pool: self.pool.clone(),
        }
    }
}
