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

use std::{collections::HashMap, rc::Rc, sync::Arc};

use anyhow::{anyhow, Result};
use askama::Template;
use axum::{extract::Query, routing::get, Extension, Form, Router};
use chrono::{Local, NaiveDate, NaiveTime};
use itertools::Itertools;
use serde::Deserialize;
use sqlx::{query, query_as, Executor, Pool, Postgres};

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
                    let value = start_time.format("%H:%M:%S").to_string().into();
                    let start_time = start_time.format("%H:%M").to_string().into();
                    let end_time = end_time
                        .map(|end_time| end_time.format("%H:%M").to_string().into())
                        .unwrap_or("?".into());

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
                    let value = start_time.format("%H:%M:%S").to_string().into();
                    let start_time = start_time.format("%H:%M").to_string().into();
                    let end_time = end_time
                        .map(|end_time| end_time.format("%H:%M").to_string().into())
                        .unwrap_or("?".into());

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

#[derive(Debug, Clone)]
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

fn parse_time(raw: &str) -> Result<NaiveTime> {
    let parts: Rc<_> = raw.trim().split(':').collect();
    let (hour, minute): (Rc<str>, Rc<str>) = match parts.as_ref() {
        [hour, minute] if (hour.len() == 1 || hour.len() == 2) && minute.len() == 2 => {
            ((*hour).into(), (*minute).into())
        }
        [full] if full.len() == 3 => ((*full)[0..1].into(), (*full)[1..2].into()),
        [full] if full.len() == 4 => ((*full)[0..2].into(), (*full)[2..4].into()),
        _ => return Err(anyhow!("Time string is improperly formatted")),
    };

    let hour = hour.parse()?;
    let minute = minute.parse()?;

    Ok(NaiveTime::from_hms_opt(hour, minute, 0)
        .ok_or(anyhow!("Could not convert numbers to time"))?)
}

struct CreateTimeReportCollection {
    categories: Box<[String]>,
    start_times: Box<[NaiveTime]>,
}

fn extract_collections<T: Iterator<Item = CreateTimeReportItem>>(
    value: T,
) -> Result<CreateTimeReportCollection> {
    let (left, right) = value.tee();
    let categories: Box<_> = left.map(|value| value.category.to_string()).collect();
    let start_times: Box<_> = right
        .map(|value| parse_time(value.start_time.as_ref()))
        .try_collect()?;

    Ok(CreateTimeReportCollection {
        categories,
        start_times,
    })
}

async fn create_time_report(
    Extension(db_pool): Extension<Pool<Postgres>>,
    Form(params): Form<HashMap<Arc<str>, Arc<str>>>,
) -> AppResult<AddTimeReportResultTemplate> {
    let params = hash_map_to_create_time_report_params(&params)?;

    let mut with_end_times: Vec<_> = Default::default();
    let mut without_end_times: Vec<_> = Default::default();

    for item in params.items.as_ref() {
        let category = item.category.to_string();
        let start_time = parse_time(item.start_time.as_ref())?;

        match item.end_time.clone() {
            Some(end_time) => {
                with_end_times.push((category, start_time, parse_time(end_time.as_ref())?))
            }
            None => without_end_times.push((category, start_time)),
        }
    }

    let mut transaction = db_pool.begin().await?;

    // With end times
    {
        let categories: Box<_> = with_end_times
            .iter()
            .map(|(value, _, _)| value.clone())
            .collect();
        let start_times: Box<_> = with_end_times
            .iter()
            .map(|(_, value, _)| value.clone())
            .collect();
        let end_times: Box<_> = with_end_times
            .iter()
            .map(|(_, _, value)| value.clone())
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
                    $3::TIME[],
                    $4::TIME[]
                ) AS t(category_name, start_time, end_time)
            ON
                tc.name = t.category_name
        ",
            params.date,
            categories.as_ref(),
            start_times.as_ref(),
            end_times.as_ref(),
        }
        .execute(&mut *transaction)
        .await?;
    }
    // Without end times
    {
        let categories: Box<_> = without_end_times
            .iter()
            .map(|(value, _)| value.clone())
            .collect();
        let start_times: Box<_> = without_end_times.iter().map(|(_, value)| *value).collect();
        query! {
            "
            INSERT INTO time_entries (category_id, day, start_time)
            SELECT
                tc.id AS category_id,
                $1 AS day,
                start_time AS start_time
            FROM
                time_categories tc
            RIGHT JOIN
                unnest(
                    $2::VARCHAR[],
                    $3::TIME[]
                ) AS t(category_name, start_time)
            ON
                tc.name = t.category_name
        ",
            params.date,
            categories.as_ref(),
            start_times.as_ref(),
        }
        .execute(&mut *transaction)
        .await?;
    }
    transaction.commit().await?;
    Ok(AddTimeReportResultTemplate)
}

async fn delete_time_reports(
    Extension(db_pool): Extension<Pool<Postgres>>,
    Form(body): Form<HashMap<Arc<str>, Arc<str>>>,
) -> AppResult<DeleteTimeReportsResultTemplate> {
    let times_to_remove: Box<_> = body
        .iter()
        .filter_map(|(key, value)| {
            if key.ends_with("--select-item") {
                Some(NaiveTime::parse_from_str(value, "%H:%M:%S"))
            } else {
                None
            }
        })
        .try_collect()?;

    let date = body.get("date").ok_or(anyhow!("Missing date"))?;
    let date = NaiveDate::parse_from_str(date, "%Y-%m-%d")?;

    let num_deleted = query! {
        "
        DELETE FROM time_entries
        WHERE start_time = Any($1::TIME[]) AND day = $2
        ",
        times_to_remove.as_ref(),
        date
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
