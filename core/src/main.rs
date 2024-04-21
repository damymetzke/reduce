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

#[derive(Template)]
#[template(path = "index.html", escape = "none")]
struct IndexTemplate;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let is_production = matches!(env::var("ENVIRONMENT").as_deref(), Ok("production"));
    dbg!(is_production);

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

    let db_url = "postgres://user:password@localhost:5432/reduce_dev";
    let db_pool = sqlx::postgres::PgPool::connect(db_url).await?;

    sqlx::migrate!("./migrations").run(&db_pool).await?;

    let app = Router::new();
    let app = app.nest("/time-reports", subsystem::time_report::routes());

    let app = routes::register(app).layer(Extension(db_pool));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Starting server listening on port 3000");
    println!("You can open the server using the URL <http://localhost:3000>");
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
