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

use crate::error::AppResult;

use super::templates::{IndexTemplate, PartItem};

pub async fn get_index() -> AppResult<impl IntoResponse> {
    Ok(IndexTemplate {
        due_items: Box::from([
            PartItem {
                title: Box::from("Task one"),
                date_relation: Box::from("Due 2 days ago"),
                rate: Box::from("Every 7 days"),
            },
            PartItem {
                title: Box::from("Task two"),
                date_relation: Box::from("Due today"),
                rate: Box::from("Every month"),
            },
        ]),
        backlog: Box::from([
            PartItem {
                title: Box::from("Task three"),
                date_relation: Box::from("Due tomorrow"),
                rate: Box::from("Every 15 days"),
            },
            PartItem {
                title: Box::from("Task four"),
                date_relation: Box::from("Due in 2 days"),
                rate: Box::from("Every week"),
            },
            PartItem {
                title: Box::from("Task five"),
                date_relation: Box::from("Due in 17 days"),
                rate: Box::from("Every Quarter"),
            },
        ]),
    })
}

