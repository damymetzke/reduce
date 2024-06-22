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
mod middleware;
mod routes;
mod sections;
mod subsystem;
mod template_extend;

use std::{env, error::Error, sync::Arc};

use askama::Template;
use axum::{Extension, Router};
use middleware::{AuthorizeUser, UserAuthenticationStatus};
use sections::{ModuleRegistration, SectionRegistration};
use template_extend::{set_navigation_links, NavigationLink};
use tracing::{Level, Subscriber};
use tracing_subscriber::FmtSubscriber;

#[derive(Template)]
#[template(path = "index.html", escape = "none")]
struct IndexTemplate {
    session: UserAuthenticationStatus,
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

    let registrations = [sections::register()];

    let mut app = Router::new();
    let mut all_navigation_links = Vec::from([
        NavigationLink {
            href: "/".into(),
            title: "Home".into(),
        },
        NavigationLink {
            href: "/upkeep".into(),
            title: "Upkeep".into(),
        },
    ]);

    for ModuleRegistration {
        default_module_name,
        sections,
    } in registrations
    {
        let mut module_router = Router::new();
        for SectionRegistration {
            default_section_name,
            router,
            navigation_links,
        } in sections.as_ref()
        {
            module_router = module_router.nest(default_section_name, router.to_owned());
            all_navigation_links.extend_from_slice(navigation_links.as_ref())
        }

        app = app.nest(default_module_name, module_router);
    }

    let app = app.nest("/upkeep", subsystem::upkeep::routes());

    let app = routes::register(app)
        .layer(Extension(db_pool.clone()))
        .layer(AuthorizeUser { pool: db_pool });

    set_navigation_links(Arc::from(all_navigation_links))?;

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(&*config.server_bind_address)
        .await
        .unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}
