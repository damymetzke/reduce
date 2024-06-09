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
use chrono::NaiveDate;
use sqlx::{query_as, Executor, Postgres};

pub struct FetchUpkeepItem {
    pub description: Arc<str>,
    pub cooldown_days: i32,
    pub due: NaiveDate,
}

pub async fn fetch_upkeep_items<'a, T>(executor: T) -> Result<Arc<[FetchUpkeepItem]>>
where
    T: Executor<'a, Database = Postgres>,
{
    Ok(query_as! {
        FetchUpkeepItem,
        "
        SELECT description, cooldown_days, due FROM upkeep_items
        ORDER BY due ASC
        "
    }
    .fetch_all(executor)
    .await?
    .into())
}
