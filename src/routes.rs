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

use std::rc::Rc;

use askama::Template;
use axum::{routing::get, Router};
use chrono::Local;
use itertools::Itertools;
use sqlx::{query_as, Pool, Postgres};

use crate::{error::AppResult, IndexTemplate};

#[derive(Template)]
#[template(path = "api/time-reports/list-item.html", escape = "none")]
struct TimeReportItemTemplate {
    start_time: Rc<str>,
    end_time: Rc<str>,
}

#[derive(Template)]
#[template(path = "api/time-reports/list-category.html", escape = "none")]
struct TimeReportCategoryTemplate {
    name: String,
    times: Box<[TimeReportItemTemplate]>,
}

#[derive(Template)]
#[template(path = "time-reports.html", escape = "none")]
struct TimeReportsTemplate {
    time_reports_today: Box<[TimeReportCategoryTemplate]>,
}

#[derive(Debug)]
struct TimeReportItem {
    name: Box<str>,
    start_time: Box<str>,
    end_time: Box<str>,
}

async fn index() -> IndexTemplate {
    IndexTemplate
}

fn convert_time(raw: &str) -> Rc<str> {
    match (raw.get(0..2), raw.get(2..4)) {
        (Some(hour), Some(minute)) => Rc::from(format!("{h}:{m}", h = hour, m = minute)),
        _ => Rc::from("_err_"),
    }
}

async fn time_reports(db_pool: Pool<Postgres>) -> AppResult<TimeReportsTemplate> {
    let time_reports_today = query_as! {
        TimeReportItem,
        "
            SELECT te.start_time, te.end_time, tc.name
            FROM time_entries te
            JOIN time_categories tc ON te.category_id = tc.id
            WHERE te.day = $1
            ORDER BY tc.name, te.start_time;
        ",
        Local::now().date_naive()

    }
    .fetch_all(&db_pool)
    .await?
    .into_iter()
    .group_by(|item| String::from(item.name.as_ref()))
    .into_iter()
    .map(|(name, times)| TimeReportCategoryTemplate {
        name,
        times: times
            .into_iter()
            .map(
                |TimeReportItem {
                     start_time,
                     end_time,
                     ..
                 }| {
                    let start_time = convert_time(start_time.as_ref());
                    let end_time = convert_time(end_time.as_ref());

                    TimeReportItemTemplate {
                        start_time,
                        end_time,
                    }
                },
            )
            .collect(),
    })
    .collect();

    Ok(TimeReportsTemplate { time_reports_today })
}

pub fn register(router: Router, db_pool: Pool<Postgres>) -> Router {
    router
        .route("/", get(|| async move { index().await }))
        .route(
            "/time-reports",
            get(|| async move { time_reports(db_pool).await }),
        )
}
