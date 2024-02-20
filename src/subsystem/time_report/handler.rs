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

use std::sync::Arc;

use askama_axum::IntoResponse;
use axum::{extract::Query, http::HeaderMap, Extension, Form};
use chrono::{Local, NaiveDate};
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use tokio::join;
use tracing::info;

use crate::{
    error::AppResult,
    subsystem::time_report::{
        database::{
            fetch_time_report_items, insert_time_entries_with_end_times,
            insert_time_entries_without_end_times,
        },
        handler::params::PostIndexItem,
        template::TimeReportInsertResultTemplate,
    },
};

use self::params::{DeleteIndexParams, PostIndexParams};

use super::{
    database::{delete_time_entries, fetch_category_names},
    template::{
        TimeReportDeleteResultTemplate, TimeReportIndexTemplate, TimeReportPickerTemplate,
        TimeReportsTemplate, AddTimeReportTemplate,
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

    let (categories, reports) = join!(
        fetch_category_names(&pool),
        fetch_time_report_items(&pool, &date)
    );
    let categories = categories?;
    let reports = reports?;

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
            },
            picker_date: date.format("%Y-%m-%d").to_string().into(),
        },
    })
}

pub async fn post_index(
    Extension(pool): Extension<Pool<Postgres>>,
    Form(params): Form<PostIndexParams>,
) -> AppResult<impl IntoResponse> {
    let mut transaction = pool.begin().await?;

    let mut categories = Vec::new();
    let mut start_times = Vec::new();

    let insert_without_end_times = {
        for (category, start_time) in params.items.iter().filter_map(|item| match item {
            PostIndexItem {
                category,
                start_time,
                end_time: None,
            } => Some((category, start_time)),
            _ => None,
        }) {
            categories.push(category.to_string());
            start_times.push(*start_time);
        }

        insert_time_entries_without_end_times(
            &mut *transaction,
            &params.date,
            categories.as_slice(),
            start_times.as_slice(),
        )
        .await?
    };

    let mut categories = Vec::new();
    let mut start_times = Vec::new();
    let mut end_times = Vec::new();

    let insert_with_end_times = {
        for (category, start_time, end_time) in params.items.iter().filter_map(|item| match item {
            PostIndexItem {
                category,
                start_time,
                end_time: Some(end_time),
            } => Some((category, start_time, end_time)),
            _ => None,
        }) {
            categories.push(category.to_string());
            start_times.push(*start_time);
            end_times.push(*end_time);
        }

        insert_time_entries_with_end_times(
            &mut *transaction,
            &params.date,
            categories.as_slice(),
            start_times.as_slice(),
            end_times.as_slice(),
        )
        .await?
    };

    let num_insertions = insert_without_end_times + insert_with_end_times;

    info!("Inserted {} time entries into the database", num_insertions);

    transaction.commit().await?;

    Ok(TimeReportInsertResultTemplate {
        date: params.date.format("%Y-%m-%d").to_string().into(),
        num_insertions,
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

    let picker: TimeReportPickerTemplate =
        fetch_time_report_items(&pool, &date).await?.as_ref().into();

    Ok((headers, picker))
}

pub async fn get_add() -> AppResult<impl IntoResponse> {
    Ok(AddTimeReportTemplate::new(
        Local::now()
            .date_naive()
            .format("%Y-%m-%d")
            .to_string()
            .into(),
        0..5,
        5,
        5,
    ))
}
