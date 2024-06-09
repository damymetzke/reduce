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
use axum::Extension;
use chrono::Local;
use sqlx::{Pool, Postgres};

use crate::error::AppResult;

use super::{
    database::fetch_upkeep_items,
    templates::{IndexTemplate, PartItem},
};

pub async fn get_index(Extension(pool): Extension<Pool<Postgres>>) -> AppResult<impl IntoResponse> {
    let items = fetch_upkeep_items(&pool).await?;
    let mut split_at = 0;
    let today = Local::now().date_naive();
    let items: Box<_> = items
        .iter()
        .map(|item| {
            if item.due <= today {
                split_at += 1;
            };
            let due_difference = (item.due - today).num_days();
            let due = match due_difference {
                -1 => "Due yesterday".into(),
                0 => "Due today".into(),
                1 => "Due tomorrow".into(),
                due if due < 0 => format!("Due {} days ago", -due).into(),
                due => format!("Due in {} days", due).into(),
            };
            PartItem {
                description: item.description.clone(),
                due,
                cooldown: format!("Cooldown: {} days", item.cooldown_days).into(),
            }
        })
        .collect();

    let (due_items, backlog) = items.split_at(split_at);
    let due_items: Box<[_]> = due_items.into();
    let backlog: Box<[_]> = backlog.into();
    Ok(IndexTemplate {
        due_items,
        backlog,
    })
}
