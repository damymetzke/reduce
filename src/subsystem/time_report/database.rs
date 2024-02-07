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
use chrono::{NaiveDate, NaiveTime};
use sqlx::{query_as, Executor, Postgres};

#[derive(Debug)]
pub struct TimeReportItemDTO {
    pub name: Box<str>,
    pub start_time: NaiveTime,
    pub end_time: Option<NaiveTime>,
}

pub async fn fetch_time_report_items<'a, T: Executor<'a, Database = Postgres>>(
    executor: T,
    date: &NaiveDate,
) -> Result<Arc<[TimeReportItemDTO]>> {
    Ok(query_as! {
        TimeReportItemDTO,
        "
            SELECT te.start_time, te.end_time, tc.name
            FROM time_entries te
            JOIN time_categories tc ON te.category_id = tc.id
            WHERE te.day = $1
            ORDER BY tc.name, te.start_time;
        ",
        date

    }
    .fetch_all(executor)
    .await?
    .into())
}
