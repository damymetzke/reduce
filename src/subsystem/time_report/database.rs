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
use sqlx::{query, query_as, Executor, Postgres};

#[derive(Debug)]
pub struct CategoryNameDTO {
    pub name: Arc<str>,
}

pub async fn fetch_category_names<'a, T: Executor<'a, Database = Postgres>>(
    executor: T,
) -> Result<Arc<[CategoryNameDTO]>> {
    Ok(query_as! {
        CategoryNameDTO,
        "SELECT name FROM time_categories ORDER BY name ASC"
    }
    .fetch_all(executor)
    .await?
    .into())
}

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

pub async fn insert_time_entries_without_end_times<'a, T: Executor<'a, Database = Postgres>>(
    executor: T,
    date: &NaiveDate,
    categories: &[String],
    start_times: &[NaiveTime],
) -> Result<u64> {
    Ok(query! {
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
        date,
        categories,
        start_times,
    }
    .execute(executor)
    .await?
    .rows_affected())
}

pub async fn insert_time_entries_with_end_times<'a, T: Executor<'a, Database = Postgres>>(
    executor: T,
    date: &NaiveDate,
    categories: &[String],
    start_times: &[NaiveTime],
    end_times: &[NaiveTime],
) -> Result<u64> {
    Ok(query! {
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
        date,
        categories,
        start_times,
        end_times,
    }
    .execute(executor)
    .await?
    .rows_affected())
}

pub async fn delete_time_entries<'a, T: Executor<'a, Database = Postgres>>(
    executor: T,
    date: &NaiveDate,
    times_to_remove: &[NaiveTime],
) -> Result<u64> {
    Ok(query! {
        "
        DELETE FROM time_entries
        WHERE start_time = Any($1::TIME[]) AND day = $2
        ",
        times_to_remove,
        date
    }
    .execute(executor)
    .await?
    .rows_affected())
}
