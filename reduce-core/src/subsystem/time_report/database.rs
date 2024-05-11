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

use super::shared::TimeReportPickerComment;

#[derive(Debug)]
pub struct ProjectInfoDTO {
    pub name: Arc<str>,
    pub id: i32,
    pub comment: Option<String>,
}

pub async fn fetch_project_info<'a, T: Executor<'a, Database = Postgres>>(
    executor: T,
    date: &NaiveDate,
    names: &[String],
) -> Result<Arc<[ProjectInfoDTO]>> {
    Ok(query_as! {
        ProjectInfoDTO,
        "
        SELECT projects.name, projects.id, time_comments.content AS comment
        FROM projects
        LEFT JOIN time_comments ON projects.id = time_comments.project_id
        AND time_comments.day = $1
        WHERE projects.name = ANY($2)
        ",
        date,
        names,
    }
    .fetch_all(executor)
    .await?
    .into())
}

pub async fn insert_comments<'a, T: Executor<'a, Database = Postgres>>(
    executor: T,
    date: &NaiveDate,
    project_ids: &[i32],
    comments: &[String],
) -> Result<u64> {
    Ok(query! {
        "
        INSERT INTO time_comments (project_id, day, content)
        SELECT t.project_id, $1, t.content
        FROM unnest(
            $2::INT[],
            $3::VARCHAR[]
        ) as t(project_id, content)
        ON CONFLICT (project_id, day) DO UPDATE 
        SET content = EXCLUDED.content
        ",
        date,
        project_ids,
        comments,
    }
    .execute(executor)
    .await?
    .rows_affected())
}

#[derive(Debug)]
pub struct CategoryNameDTO {
    pub name: Arc<str>,
}

pub async fn fetch_category_names<'a, T: Executor<'a, Database = Postgres>>(
    executor: T,
) -> Result<Arc<[CategoryNameDTO]>> {
    Ok(query_as! {
        CategoryNameDTO,
        "SELECT name FROM projects ORDER BY name ASC"
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
            SELECT te.start_time, te.end_time, p.name
            FROM time_entries te
            JOIN projects p ON te.project_id = p.id
            WHERE te.day = $1
            ORDER BY p.name, te.start_time;
        ",
        date

    }
    .fetch_all(executor)
    .await?
    .into())
}

struct TimeReportCommentInternal {
    name: String,
    content: Option<String>,
}

pub async fn fetch_time_report_comments<'a, T>(
    executor: T,
    date: &NaiveDate,
) -> Result<Box<[TimeReportPickerComment]>>
where
    T: Executor<'a, Database = Postgres>,
{
    dbg!(date);
    Ok(query_as! {
        TimeReportCommentInternal,
        "
            SELECT tc.content, p.name
            FROM time_comments tc
            JOIN projects p ON tc.project_id = p.id
            WHERE tc.day = $1
            ORDER BY p.name;
        ",
        date

    }
    .fetch_all(executor)
    .await?
    .into_iter()
    .map(
        |TimeReportCommentInternal { name, content }| TimeReportPickerComment {
            project: name.as_str().into(),
            comments: content.unwrap_or_default().into(),
        },
    )
    .collect())
}

pub async fn insert_time_entries_without_end_times<'a, T: Executor<'a, Database = Postgres>>(
    executor: T,
    date: &NaiveDate,
    categories: &[String],
    start_times: &[NaiveTime],
) -> Result<u64> {
    Ok(query! {
        "
            INSERT INTO time_entries (project_id, day, start_time)
            SELECT
                tc.id AS project_id,
                $1 AS day,
                start_time AS start_time
            FROM
                projects tc
            RIGHT JOIN
                unnest(
                    $2::VARCHAR[],
                    $3::TIME[]
                ) AS t(project_name, start_time)
            ON
                tc.name = t.project_name
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
            INSERT INTO time_entries (project_id, day, start_time, end_time)
            SELECT
                tc.id AS project_id,
                $1 AS day,
                start_time AS start_time,
                end_time AS end_time
            FROM
                projects tc
            RIGHT JOIN
                unnest(
                    $2::VARCHAR[],
                    $3::TIME[],
                    $4::TIME[]
                ) AS t(project_name, start_time, end_time)
            ON
                tc.name = t.project_name
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
