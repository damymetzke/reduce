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

use anyhow::Result;
use askama::Template;
use axum::{extract::Query, routing::get, Extension, Router};
use chrono::{Local, NaiveDate};
use itertools::Itertools;
use serde::Deserialize;
use sqlx::{query_as, Pool, Postgres};

use crate::{error::AppResult, IndexTemplate};

#[derive(Template)]
#[template(path = "api/time-reports/list-item.html", escape = "none")]
struct TimeReportItemTemplate {
    start_time: Arc<str>,
    end_time: Arc<str>,
}

#[derive(Template)]
#[template(path = "api/time-reports/list-category.html", escape = "none")]
struct TimeReportCategoryTemplate {
    name: String,
    times: Box<[TimeReportItemTemplate]>,
}

#[derive(Template)]
#[template(path = "api/time-reports/index.html", escape = "none")]
struct TimeReportPickerTemplate {
    reports: Box<[TimeReportCategoryTemplate]>,
}

#[derive(Template)]
#[template(path = "time-reports.html", escape = "none")]
struct TimeReportsTemplate {
    time_reports_today: Box<[TimeReportCategoryTemplate]>,
    picker_date: Arc<str>,
    time_report_picker: TimeReportPickerTemplate,
}

#[derive(Debug)]
struct TimeReportItem {
    name: Box<str>,
    start_time: Box<str>,
    end_time: Box<str>,
}

async fn make_time_report_picker(
    today: NaiveDate,
    db_pool: Pool<Postgres>,
) -> Result<TimeReportPickerTemplate> {
    let time_reports = query_as! {
        TimeReportItem,
        "
            SELECT te.start_time, te.end_time, tc.name
            FROM time_entries te
            JOIN time_categories tc ON te.category_id = tc.id
            WHERE te.day = $1
            ORDER BY tc.name, te.start_time;
        ",
        today

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

    Ok(TimeReportPickerTemplate {
        reports: time_reports,
    })
}

async fn index() -> IndexTemplate {
    IndexTemplate
}

fn convert_time(raw: &str) -> Arc<str> {
    match (raw.get(0..2), raw.get(2..4)) {
        (Some(hour), Some(minute)) => Arc::from(format!("{h}:{m}", h = hour, m = minute)),
        _ => Arc::from("_err_"),
    }
}

async fn time_reports(db_pool: Extension<Pool<Postgres>>) -> AppResult<TimeReportsTemplate> {
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
    .fetch_all(&db_pool.0)
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

    let today = Local::now().date_naive();

    Ok(TimeReportsTemplate {
        time_reports_today,
        picker_date: today.format("%Y-%m-%d").to_string().into(),
        time_report_picker: make_time_report_picker(today, db_pool.0).await?,
    })
}

#[derive(Debug, Deserialize)]
struct TimeReportPickerParams {
    date: Arc<str>,
}

async fn time_report_picker(
    params: Query<TimeReportPickerParams>,
    db_pool: Extension<Pool<Postgres>>,
) -> AppResult<TimeReportPickerTemplate> {
    let date = NaiveDate::parse_from_str(params.date.as_ref(), "%Y-%m-%d");
    Ok(make_time_report_picker(date?, db_pool.0).await?)
}

pub fn register(router: Router) -> Router {
    router
        .route("/", get(|| async move { index().await }))
        .route("/time-reports", get(time_reports))
        .route("/api/time-reports", get(time_report_picker))
}
