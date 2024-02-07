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

use askama_axum::IntoResponse;
use axum::{extract::Query, Extension};
use chrono::{Local, NaiveDate};
use serde::Deserialize;
use sqlx::{Pool, Postgres};

use crate::{error::AppResult, subsystem::time_report::database::fetch_time_report_items};

use super::template::{TimeReportIndexTemplate, TimeReportPickerTemplate, TimeReportsTemplate};

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

    let reports = fetch_time_report_items(&pool, &date).await?;

    Ok(TimeReportsTemplate {
        categories: [].into(),
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

pub async fn post_index() -> impl IntoResponse {
    todo!()
}

pub async fn delete_index() -> impl IntoResponse {
    todo!()
}

pub async fn get_picker() -> impl IntoResponse {
    todo!()
}

pub async fn get_add_form() -> impl IntoResponse {
    todo!()
}

pub async fn get_add_extra() -> impl IntoResponse {
    todo!()
}

pub async fn get_schedule() -> impl IntoResponse {
    todo!()
}
