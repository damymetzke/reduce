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

pub mod params;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use anyhow::anyhow;
use askama_axum::IntoResponse;
use axum::{extract::Query, http::HeaderMap, Extension, Form};
use chrono::{Local, NaiveDate, NaiveTime};
use itertools::{chain, Either, Itertools};
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use tokio::join;

use crate::{
    error::AppResult,
    subsystem::time_report::{
        database::{
            fetch_project_info, fetch_time_report_items, insert_comments,
            insert_time_entries_without_end_times,
        },
        logic::{normalize_comment, parse_time},
    },
};

use self::params::DeleteIndexParams;

use super::{
    database::{delete_time_entries, fetch_category_names, fetch_time_report_comments, insert_time_entries_with_end_times},
    template::{
        AddTimeReportExtraItemTemplate, AddTimeReportTemplate, TimeReportDeleteResultTemplate, TimeReportIndexTemplate, TimeReportInsertResultTemplate, TimeReportPickerTemplate, TimeReportsTemplate
    },
};

#[derive(Debug, Deserialize)]
pub struct TimeReportsParams {
    date: Option<Box<str>>,
}

pub async fn get_index(
    Extension(pool): Extension<Pool<Postgres>>,
    Query(params): Query<TimeReportsParams>,
) -> AppResult<impl IntoResponse> {
    let date = params
        .date
        .and_then(|date| NaiveDate::parse_from_str(date.as_ref(), "%Y-%m-%d").ok())
        .unwrap_or(Local::now().date_naive());

    let (categories, reports, comments) = join!(
        fetch_category_names(&pool),
        fetch_time_report_items(&pool, &date),
        fetch_time_report_comments(&pool, &date),
    );
    let categories = categories?;
    let reports = reports?;
    let comments = comments?;

    Ok(TimeReportsTemplate {
        categories: categories
            .iter()
            .map(|category| category.name.clone())
            .collect(),
        time_reports: TimeReportIndexTemplate {
            time_report_picker: TimeReportPickerTemplate {
                reports: reports
                    .iter()
                    .enumerate()
                    .map(|(i, report)| (i as u16, report).into())
                    .collect(),
                comments,
            },
            picker_date: date.format("%Y-%m-%d").to_string().into(),
        },
    })
}

pub async fn post_index(
    Extension(pool): Extension<Pool<Postgres>>,
    Form(params): Form<HashMap<Arc<str>, Arc<str>>>,
) -> AppResult<impl IntoResponse> {
    let date = params.get("date").ok_or(anyhow!("Date was not provided"))?;
    let date = NaiveDate::parse_from_str(date, "%Y-%m-%d")?;

    let indexes: HashSet<_> = params
        .iter()
        .filter_map(|(key, value)| match value.as_ref() {
            "" => None,
            _ => Some(key),
        })
        .filter_map(|key| key.split_once("--"))
        .filter_map(|(index, _)| index.parse::<u16>().ok())
        .collect();

    let key_convert = |input: String| {
        params
            .get(input.as_str())
            .and_then(|value| match value.as_ref() {
                "" => None,
                _ => Some(value),
            })
    };

    let (without_end_times, with_end_times): (Vec<_>, Vec<_>) = indexes
        .iter()
        .filter_map(|index| {
            match (
                key_convert(format!("{}--project", index)),
                key_convert(format!("{}--start-time", index)),
                key_convert(format!("{}--end-time", index)),
                key_convert(format!("{}--comment", index)),
            ) {
                (Some(project), Some(start_time), end_time, comment) => {
                    Some((project.as_ref(), start_time.as_ref(), end_time, comment))
                }
                _ => None,
            }
        })
        .partition_map(|(project, start_time, end_time, comment)| match end_time {
            None => Either::Left((project, start_time, comment)),
            Some(end_time) => Either::Right((project, start_time, end_time.as_ref(), comment)),
        });

    let project_names: HashSet<_> = chain(
        without_end_times.iter().map(|(project, ..)| project),
        with_end_times.iter().map(|(project, ..)| project),
    )
    .copied()
    .collect();
    let project_names: Box<_> = project_names.into_iter().map(String::from).collect();

    let project_info = fetch_project_info(&pool, &date, project_names.as_ref()).await?;

    let mut comments: HashMap<&str, String> = HashMap::default();

    for (project, comment) in chain!(
        project_info
            .iter()
            .map(|info| (info.name.as_ref(), info.comment.clone())),
        without_end_times.iter().map(|(project, _, comment)| (
            *project,
            comment.map(|value| String::from(value.as_ref()))
        )),
        with_end_times.iter().map(|(project, _, _, comment)| (
            *project,
            comment.map(|value| String::from(value.as_ref()))
        )),
    )
    .filter_map(|(project, comment)| comment.map(|comment| (project, comment)))
    {
        if let Some(value) = comments.get_mut(project) {
            value.push(';');
            value.push_str(comment.as_ref());
            continue;
        }

        comments.insert(project, comment);
    }

    let project_name_to_id: HashMap<_, _> = project_info
        .iter()
        .map(|info| (info.name.as_ref(), info.id))
        .collect();

    let mut transaction = pool.begin().await?;

    let _num_comments = {
        let (project_ids, comments): (Vec<_>, Vec<_>) = comments
            .iter()
            .filter_map(|(key, value)| {
                project_name_to_id
                    .get(*key)
                    .map(|key| (*key, normalize_comment(value.as_str())))
            })
            .unzip();

        insert_comments(
            &mut *transaction,
            &date,
            project_ids.as_slice(),
            comments.as_slice(),
        )
        .await?
    };

    let num_without_end_times = {
        let (categories, start_times): (Vec<_>, Vec<_>) = without_end_times
            .iter()
            .filter_map(|(project, start_time, _)| {
                parse_time(start_time)
                    .ok()
                    .map(|start_time| (String::from(*project), start_time))
            })
            .unzip();

        insert_time_entries_without_end_times(
            &mut *transaction,
            &date,
            categories.as_slice(),
            start_times.as_slice(),
        )
        .await?
    };

    let num_with_end_times = {
        let (categories, start_times, end_times): (Vec<_>, Vec<_>, Vec<_>) = with_end_times
            .iter()
            .filter_map(|(project, start_time, end_time, _)| {
                parse_time(start_time)
                    .ok()
                    .zip(parse_time(end_time).ok())
                    .map(|(start_time, end_time)| (String::from(*project), start_time, end_time))
            })
            .multiunzip();

        insert_time_entries_with_end_times(
            &mut *transaction,
            &date,
            categories.as_slice(),
            start_times.as_slice(),
            end_times.as_slice(),
        )
        .await?
    };

    transaction.commit().await?;

    Ok(TimeReportInsertResultTemplate {
        date: date.format("%Y-%m-%d").to_string().into(),
        num_insertions: num_without_end_times + num_with_end_times,
    })
}

pub async fn delete_index(
    Extension(pool): Extension<Pool<Postgres>>,
    Form(body): Form<DeleteIndexParams>,
) -> AppResult<impl IntoResponse> {
    let num_deleted = delete_time_entries(&pool, &body.date, body.selected_times.as_ref()).await?;

    Ok(TimeReportDeleteResultTemplate { num_deleted })
}

#[derive(Debug, Deserialize)]
pub struct TimeReportPickerParams {
    date: Arc<str>,
}

pub async fn get_picker(
    params: Query<TimeReportPickerParams>,
    Extension(pool): Extension<Pool<Postgres>>,
) -> AppResult<impl IntoResponse> {
    let date = NaiveDate::parse_from_str(params.date.as_ref(), "%Y-%m-%d")?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Push-Url",
        format!("/time-reports?date={}", params.date).parse()?,
    );

    let (reports, comments) = join!(
        fetch_time_report_items(&pool, &date),
        fetch_time_report_comments(&pool, &date),
    );

    let reports = reports?
        .iter()
        .enumerate()
        .map(|(i, value)| (i as u16, value).into())
        .collect();
    let comments = comments?;

    let picker = TimeReportPickerTemplate { reports, comments };

    Ok((headers, picker))
}

pub async fn get_add() -> AppResult<impl IntoResponse> {
    Ok(AddTimeReportTemplate::new(
        Local::now()
            .date_naive()
            .format("%Y-%m-%d")
            .to_string()
            .into(),
        0,
        5,
    ))
}

#[derive(Debug, Deserialize)]
pub struct AddTimeReportItemsParams {
    offset: u16,
    add: u16,
}

pub async fn get_add_items(
    Query(params): Query<AddTimeReportItemsParams>,
) -> AppResult<impl IntoResponse> {
    Ok(AddTimeReportExtraItemTemplate::new(
        params.offset,
        params.add,
    ))
}

#[derive(Debug, Deserialize)]
pub struct GetTimeReportScheduleParams {
    date: Arc<str>,
}

pub async fn get_schedule(
    Extension(pool): Extension<Pool<Postgres>>,
    Query(params): Query<GetTimeReportScheduleParams>,
) -> AppResult<impl IntoResponse> {
    let date = NaiveDate::parse_from_str(params.date.as_ref(), "%Y-%m-%d")?;

    let (reports, comments) = join!(
        fetch_time_report_items(&pool, &date),
        fetch_time_report_comments(&pool, &date)
    );

    let reports = reports?;
    let comments = comments?;

    Ok(TimeReportIndexTemplate {
        time_report_picker: TimeReportPickerTemplate {
            reports: reports
                .iter()
                .enumerate()
                .map(|(i, report)| (i as u16, report).into())
                .collect(),
            comments,
        },
        picker_date: date.format("%Y-%m-%d").to_string().into(),
    })
}
