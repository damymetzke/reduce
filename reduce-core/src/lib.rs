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

mod error;
mod routes;
mod subsystem;

use std::{env, error::Error};

use askama::Template;
use axum::{Extension, Router};
use tracing::{Level, Subscriber};
use tracing_subscriber::FmtSubscriber;

struct PartModule {
    href: Box<str>,
    title: Box<str>,
}

#[derive(Template)]
#[template(path = "index.html", escape = "none")]
struct IndexTemplate{
    modules: Box<[PartModule]>
}

pub fn setup_tracing() -> Result<(), Box<dyn Error>> {
    let is_production = matches!(env::var("ENVIRONMENT").as_deref(), Ok("production"));
    let mut _guard = None;
    let tracing_subscriber: Box<dyn Subscriber + Send + Sync + 'static> =
        match env::var("LOG_DIRECTORY") {
            Ok(log_directory) if is_production => {
                let file_appender = tracing_appender::rolling::daily(log_directory, "reduce.json");
                let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
                _guard = Some(guard);
                Box::from(
                    tracing_subscriber::fmt()
                        .json()
                        .with_writer(non_blocking)
                        .with_max_level(Level::TRACE)
                        .finish(),
                )
            }
            _ => Box::from(
                FmtSubscriber::builder()
                    .with_max_level(Level::TRACE)
                    .finish(),
            ),
        };

    tracing::subscriber::set_global_default(tracing_subscriber)?;
    Ok(())
}

#[derive(Debug)]
pub struct ServerConfig {
    pub db_url: Box<str>,
    pub server_bind_address: Box<str>,
}

pub async fn start_server(config: ServerConfig) -> Result<(), Box<dyn Error>> {
    let db_url = config.db_url;
    let db_pool = sqlx::postgres::PgPool::connect(&db_url).await?;

    sqlx::migrate!("./migrations").run(&db_pool).await?;

    let app = Router::new();
    let app = app
        .nest("/time-reports", subsystem::time_report::routes())
        .nest("/upkeep", subsystem::upkeep::routes());

    let app = routes::register(app).layer(Extension(db_pool));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(&*config.server_bind_address)
        .await
        .unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}
