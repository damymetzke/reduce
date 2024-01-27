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

use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Result};
use askama::Template;
use axum::{extract::Query, routing::get, Extension, Form, Router};
use chrono::{Local, NaiveDate};
use itertools::Itertools;
use serde::Deserialize;
use sqlx::{query, query_as, Pool, Postgres};

use crate::{error::AppResult, IndexTemplate};

#[derive(Template)]
#[template(path = "api/time-reports/list-item.html", escape = "none")]
struct TimeReportItemTemplate {
    start_time: Arc<str>,
    end_time: Arc<str>,
    i: u16,
    value: Arc<str>,
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
struct AddTimeReportResultTemplate;

#[derive(Template)]
#[template(path = "api/time-reports/delete_result.html", escape = "none")]
struct DeleteTimeReportsResultTemplate {
    num_deleted: u16,
}

#[derive(Template)]
#[template(path = "time-reports.html", escape = "none")]
struct TimeReportsTemplate {
    time_reports_today: Box<[TimeReportCategoryTemplate]>,
    picker_date: Arc<str>,
    time_report_picker: TimeReportPickerTemplate,
    categories: Arc<[Arc<str>]>,
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
            .enumerate()
            .map(
                |(
                    i,
                    TimeReportItem {
                        start_time,
                        end_time,
                        ..
                    },
                )| {
                    let value = start_time.as_ref().into();
                    let start_time = convert_time(start_time.as_ref());
                    let end_time = convert_time(end_time.as_ref());

                    TimeReportItemTemplate {
                        start_time,
                        end_time,
                        i: i as u16,
                        value,
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

async fn time_reports(
    Extension(db_pool): Extension<Pool<Postgres>>,
) -> AppResult<TimeReportsTemplate> {
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
            .enumerate()
            .map(
                |(
                    i,
                    TimeReportItem {
                        start_time,
                        end_time,
                        ..
                    },
                )| {
                    let value = start_time.as_ref().into();
                    let start_time = convert_time(start_time.as_ref());
                    let end_time = convert_time(end_time.as_ref());

                    TimeReportItemTemplate {
                        start_time,
                        end_time,
                        i: i as u16,
                        value,
                    }
                },
            )
            .collect(),
    })
    .collect();

    let categories = query_as! {
        GetCategoryNames,
        "SELECT name FROM time_categories ORDER BY name ASC"
    }
    .fetch_all(&db_pool)
    .await?
    .into_iter()
    .map(|value| value.name)
    .collect();

    let today = Local::now().date_naive();

    Ok(TimeReportsTemplate {
        time_reports_today,
        picker_date: today.format("%Y-%m-%d").to_string().into(),
        time_report_picker: make_time_report_picker(today, db_pool).await?,
        categories,
    })
}

#[derive(Debug)]
struct CreateTimeReportItem {
    category: Arc<str>,
    start_time: Arc<str>,
    end_time: Option<Arc<str>>,
}

#[derive(Debug)]
struct CreateTimeReportParams {
    date: NaiveDate,
    items: Arc<[CreateTimeReportItem]>,
}

fn extract_new_time_report_items(
    input: &HashMap<Arc<str>, Arc<str>>,
) -> Result<Arc<[CreateTimeReportItem]>> {
    let mut categories: HashMap<u16, &Arc<str>> = Default::default();
    let mut start_times: HashMap<u16, &Arc<str>> = Default::default();
    let mut end_times: HashMap<u16, &Arc<str>> = Default::default();

    for value in input {
        let (key, value) = value;
        if value.is_empty() {
            continue;
        };

        let (left, right) = match key.as_ref().split_once('-') {
            Some(value) => value,
            _ => continue,
        };

        let i: u16 = match left.parse() {
            Ok(value) => value,
            _ => continue,
        };

        if right == "-category" {
            categories.insert(i, value);
        };
        if right == "-start-time" {
            start_times.insert(i, value);
        };
        if right == "-end-time" {
            end_times.insert(i, value);
        };
    }

    let mut result: Vec<CreateTimeReportItem> = Default::default();

    let mut i: u16 = 0;
    loop {
        let category = categories.get(&i);
        let start_time = start_times.get(&i);
        let end_time = end_times.get(&i).map(|value| (*value).clone());

        let item = match (category, start_time, end_time) {
            (Some(category), Some(start_time), end_time) => CreateTimeReportItem {
                category: (*category).clone(),
                start_time: (*start_time).clone(),
                end_time,
            },
            (_, None, None) => break,
            _ => return Err(anyhow!("Invalid list of items")),
        };

        result.push(item);

        i += 1;
    }

    if end_times.keys().any(|value| *value >= i) {
        return Err(anyhow!("Invalid list of items"));
    }

    Ok(result.into())
}

fn hash_map_to_create_time_report_params(
    input: &HashMap<Arc<str>, Arc<str>>,
) -> Result<CreateTimeReportParams> {
    let date = input
        .get("date")
        .ok_or(anyhow!("Expected 'date' parameter"))?;
    let date = NaiveDate::parse_from_str(date.as_ref(), "%Y-%m-%d")?;

    let items = extract_new_time_report_items(input)?;
    Ok(CreateTimeReportParams { date, items })
}

async fn create_time_report(
    Extension(db_pool): Extension<Pool<Postgres>>,
    Form(params): Form<HashMap<Arc<str>, Arc<str>>>,
) -> AppResult<AddTimeReportResultTemplate> {
    let params = hash_map_to_create_time_report_params(&params)?;

    let categories: Box<_> = params
        .items
        .iter()
        .map(|value| value.category.to_string())
        .collect();
    let start_times: Box<_> = params
        .items
        .iter()
        .map(|value| value.start_time.to_string())
        .collect();
    let end_times: Box<_> = params
        .items
        .iter()
        .map(|value| value.end_time.clone().unwrap_or(Arc::from("")).to_string())
        .collect();

    query! {
        "
            INSERT INTO time_entries (category_id, day, start_time, end_time)
            SELECT
                tc.id AS category_id,
                $1 AS day,
                start_time AS start_time,
                end_time AS end_time
            FROM
                time_categories tc
            RIGHT JOIN
                unnest(
                    $2::VARCHAR[],
                    $3::VARCHAR[],
                    $4::VARCHAR[]
                ) AS t(category_name, start_time, end_time)
            ON
                tc.name = t.category_name
        ",
        params.date,
        categories.as_ref(),
        start_times.as_ref(),
        end_times.as_ref(),
    }
    .execute(&db_pool)
    .await?;

    Ok(AddTimeReportResultTemplate)
}

async fn delete_time_reports(
    Extension(db_pool): Extension<Pool<Postgres>>,
    Form(body): Form<HashMap<Arc<str>, Arc<str>>>,
) -> AppResult<DeleteTimeReportsResultTemplate> {
    let times_to_remove: Box<_> = body.values().map(ToString::to_string).collect();

    let num_deleted = query! {
        "
        DELETE FROM time_entries
        WHERE start_time = Any($1::text[])
        ",
        times_to_remove.as_ref()
    }
    .execute(&db_pool)
    .await?
    .rows_affected() as u16;

    Ok(DeleteTimeReportsResultTemplate { num_deleted })
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

struct GetCategoryNames {
    name: Arc<str>,
}

async fn add_time_report_items(
    params: Query<AddTimeReportItemsParams>,
) -> AppResult<AddTimeReportExtraItemTemplate> {
    Ok(AddTimeReportExtraItemTemplate {
        items: make_time_report_items(params.offset, params.add)
            .into_iter()
            .collect(),
        offset: params.offset,
        add: params.add,
    })
}

async fn add_time_report() -> AppResult<AddTimeReportTemplate> {
    Ok(AddTimeReportTemplate {
        date: Local::now()
            .date_naive()
            .format("%Y-%m-%d")
            .to_string()
            .into(),
        items: make_time_report_items(0, 5)
            .into_iter()
            .collect::<Vec<_>>()
            .as_slice()
            .into(),
        offset: 5,
        add: 5,
    })
}

pub fn register(router: Router) -> Router {
    router
        .route("/", get(|| async move { index().await }))
        .route(
            "/time-reports",
            get(time_reports)
                .post(create_time_report)
                .delete(delete_time_reports),
        )
        .route("/api/time-reports", get(time_report_picker))
        .route("/api/time-reports/add", get(add_time_report))
        .route("/api/time-reports/add/items", get(add_time_report_items))
}
