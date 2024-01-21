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

use std::error::Error;

use askama::Template;
use axum::{Extension, Router};

#[derive(Template)]
#[template(path = "index.html", escape = "none")]
struct IndexTemplate;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let db_url = "postgres://user:password@localhost:5432/reduce_dev";
    let db_pool = sqlx::postgres::PgPool::connect(db_url).await?;

    sqlx::migrate!("./migrations").run(&db_pool).await?;

    let app = Router::new();

    let app = routes::register(app).layer(Extension(db_pool));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Starting server listening on port 3000");
    println!("You can open the server using the URL <http://localhost:3000>");
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
