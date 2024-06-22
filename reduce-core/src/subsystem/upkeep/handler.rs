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

use askama_axum::IntoResponse;
use axum::{debug_handler, extract::Path, Extension, Form};
use chrono::{Duration, Local, NaiveDate};
use serde::Deserialize;
use sqlx::{Pool, Postgres};

use crate::{error::AppResult, middleware::UserAuthenticationStatus};

use super::{
    database::{
        complete_upkeep_item, delete_upkeep_item, fetch_upkeep_items, insert_upkeep_item,
        patch_due_date_upkeep_item,
    },
    templates::{IndexTemplate, PartItem},
};

pub async fn get_index(
    Extension(session): Extension<UserAuthenticationStatus>,
    Extension(pool): Extension<Pool<Postgres>>,
) -> AppResult<impl IntoResponse> {
    let items = fetch_upkeep_items(&pool).await?;
    let mut split_at = 0;
    let today = Local::now().date_naive();
    let items: Box<_> = items
        .iter()
        .map(|item| {
            let is_due = item.due <= today;

            if is_due {
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
                id: item.id,
                description: item.description.clone(),
                due,
                cooldown: format!("Cooldown: {} days", item.cooldown_days).into(),
                render_complete: is_due,
            }
        })
        .collect();

    let (due_items, backlog) = items.split_at(split_at);
    let due_items: Box<[_]> = due_items.into();
    let backlog: Box<[_]> = backlog.into();
    Ok(IndexTemplate {
        due_items,
        backlog,
        session,
    })
}

#[derive(Deserialize)]
pub struct PostIndexForm {
    title: Arc<str>,
    cooldown: i32,
}

pub async fn post_index(
    session: Extension<UserAuthenticationStatus>,
    pool: Extension<Pool<Postgres>>,
    Form(PostIndexForm { title, cooldown }): Form<PostIndexForm>,
) -> AppResult<impl IntoResponse> {
    let due = Local::now().date_naive() + Duration::days(cooldown as i64);
    insert_upkeep_item(&pool.0, title.as_ref(), cooldown, &due).await?;
    get_index(session, pool).await
}

pub async fn post_complete(
    session: Extension<UserAuthenticationStatus>,
    pool: Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
) -> AppResult<impl IntoResponse> {
    complete_upkeep_item(&pool.0, id).await?;
    get_index(session, pool).await
}

pub async fn delete_item(
    session: Extension<UserAuthenticationStatus>,
    pool: Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
) -> AppResult<impl IntoResponse> {
    delete_upkeep_item(&pool.0, id).await?;
    get_index(session, pool).await
}

#[derive(Deserialize)]
pub struct PatchItemForm {
    due_date: NaiveDate,
}

pub async fn patch_item(
    session: Extension<UserAuthenticationStatus>,
    pool: Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
    Form(PatchItemForm { due_date }): Form<PatchItemForm>,
) -> AppResult<impl IntoResponse> {
    patch_due_date_upkeep_item(&pool.0, id, &due_date).await?;
    get_index(session, pool).await
}
