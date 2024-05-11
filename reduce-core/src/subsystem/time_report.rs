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

use axum::{routing::get, Router};

use self::handler::{
    delete_index, get_add, get_add_items, get_index, get_picker, get_schedule, post_index,
};

mod database;
mod handler;
mod logic;
mod shared;
mod template;

pub fn routes() -> Router {
    Router::new()
        .route("/", get(get_index).post(post_index).delete(delete_index))
        .route("/picker", get(get_picker))
        .route("/add", get(get_add))
        .route("/add/items", get(get_add_items))
        .route("/schedule", get(get_schedule))
}
