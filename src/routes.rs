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
use askama_axum::IntoResponse;
use axum::{extract::Query, http::HeaderMap, routing::get, Extension, Router};
use chrono::{NaiveDate, NaiveTime};
use serde::Deserialize;
use sqlx::{query_as, Pool, Postgres};

use crate::{error::AppResult, IndexTemplate};

#[derive(Template)]
#[template(path = "api/time-reports/list-item.html", escape = "none")]
struct TimeReportItemTemplate {
    category: Arc<str>,
    start_time: Arc<str>,
    end_time: Arc<str>,
    i: u16,
    value: Arc<str>,
}

#[derive(Template)]
#[template(path = "api/time-reports/picker.html", escape = "none")]
struct TimeReportPickerTemplate {
    reports: Box<[TimeReportItemTemplate]>,
}

#[derive(Template)]
#[template(path = "api/time-reports/index.html", escape = "none")]
struct TimeReportIndexTemplate {
    time_report_picker: TimeReportPickerTemplate,
    picker_date: Arc<str>,
}

#[derive(Template, Clone)]
#[template(path = "api/time-reports/add/item.html", escape = "none")]
struct AddTimeReportItemTemplate {
    i: u16,
}

#[derive(Template, Clone)]
#[template(path = "api/time-reports/add/extra.html", escape = "none")]
struct AddTimeReportExtraItemTemplate {
    items: Arc<[AddTimeReportItemTemplate]>,
    offset: u16,
    add: u16,
}

#[derive(Template)]
#[template(path = "api/time-reports/add.html", escape = "none")]
struct AddTimeReportTemplate {
    date: Arc<str>,
    items: Arc<[AddTimeReportItemTemplate]>,
    offset: u16,
    add: u16,
}

#[derive(Template)]
#[template(path = "api/time-reports/add/result.html", escape = "none")]
struct AddTimeReportResultTemplate {
    date: Arc<str>,
}

#[derive(Template)]
#[template(path = "api/time-reports/delete_result.html", escape = "none")]
struct DeleteTimeReportsResultTemplate {
    num_deleted: u16,
}

#[derive(Template)]
#[template(path = "time-reports.html", escape = "none")]
struct TimeReportsTemplate {
    categories: Arc<[Arc<str>]>,
    time_reports: TimeReportIndexTemplate,
}

#[derive(Debug)]
struct TimeReportItem {
    name: Box<str>,
    start_time: NaiveTime,
    end_time: Option<NaiveTime>,
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
    .enumerate()
    .map(
        |(
            i,
            TimeReportItem {
                name,
                start_time,
                end_time,
            },
        )| TimeReportItemTemplate {
            category: name.into(),
            start_time: start_time.format("%H:%M").to_string().into(),
            end_time: end_time
                .map(|end_time| end_time.format("%H:%M").to_string().into())
                .unwrap_or("".into()),
            i: i as u16,
            value: start_time.format("%H:%M:%S").to_string().into(),
        },
    )
    .collect();

    Ok(TimeReportPickerTemplate {
        reports: time_reports,
    })
}

async fn index() -> IndexTemplate {
    IndexTemplate
}

fn make_time_report_items(
    offset: u16,
    add: u16,
) -> impl IntoIterator<Item = AddTimeReportItemTemplate> {
    (offset..offset + add).map(move |i| AddTimeReportItemTemplate { i })
}

#[derive(Debug, Deserialize)]
struct AddTimeReportItemsParams {
    offset: u16,
    add: u16,
}

async fn add_time_report_items(
    params: Query<AddTimeReportItemsParams>,
) -> AppResult<AddTimeReportExtraItemTemplate> {
    Ok(AddTimeReportExtraItemTemplate {
        items: make_time_report_items(params.offset, params.add)
            .into_iter()
            .collect(),
        offset: params.offset + params.add,
        add: params.add,
    })
}

#[derive(Debug, Deserialize)]
struct GetTimeReportScheduleParams {
    date: Arc<str>,
}

async fn get_time_report_schedule(
    Extension(db_pool): Extension<Pool<Postgres>>,
    Query(params): Query<GetTimeReportScheduleParams>,
) -> AppResult<TimeReportIndexTemplate> {
    let date = NaiveDate::parse_from_str(params.date.as_ref(), "%Y-%m-%d")?;
    Ok(TimeReportIndexTemplate {
        time_report_picker: make_time_report_picker(date, db_pool).await?,
        picker_date: date.format("%Y-%m-%d").to_string().into(),
    })
}

async fn get_style() -> AppResult<impl IntoResponse> {
    let mut headers = HeaderMap::new();
    headers.insert("Content-type", "text/css".parse()?);

    Ok((
        headers,
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/static/style.css")),
    ))
}

pub fn register(router: Router) -> Router {
    router
        .route("/", get(|| async move { index().await }))
        .route("/time-reports/add/items", get(add_time_report_items))
        .route("/time-reports/schedule", get(get_time_report_schedule))
        .route("/static/style.css", get(get_style))
}
