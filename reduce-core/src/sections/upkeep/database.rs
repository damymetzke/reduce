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
use sqlx::{query, query_as, Executor, Postgres};

pub struct FetchUpkeepItem {
    pub id: i32,
    pub description: Arc<str>,
    pub cooldown_days: i32,
    pub due: NaiveDate,
}

pub async fn fetch_upkeep_items<'a, T>(
    executor: T,
    account_id: i32,
) -> Result<Arc<[FetchUpkeepItem]>>
where
    T: Executor<'a, Database = Postgres>,
{
    Ok(query_as! {
        FetchUpkeepItem,
        "
        SELECT id, description, cooldown_days, due FROM upkeep_items
        WHERE account_id = $1
        ORDER BY due ASC
        ",
        account_id
    }
    .fetch_all(executor)
    .await?
    .into())
}

pub async fn insert_upkeep_item<'a, T>(
    executor: T,
    account_id: i32,
    description: &str,
    cooldown_days: i32,
    due: &NaiveDate,
) -> Result<()>
where
    T: Executor<'a, Database = Postgres>,
{
    query! {
        "
        INSERT INTO upkeep_items (account_id, description, cooldown_days, due)
        VALUES ($1, $2, $3, $4)
        ",
        account_id,
        description,
        cooldown_days,
        due,
    }
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn complete_upkeep_item<'a, T>(executor: T, id: i32, account_id: i32) -> Result<()>
where
    T: Executor<'a, Database = Postgres>,
{
    query! {
        "
        UPDATE upkeep_items
        SET due = CURRENT_DATE + cooldown_days * INTERVAL '1 day'
        WHERE id = $1 AND account_id = $2
        ",
        id,
        account_id,
    }
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn delete_upkeep_item<'a, T>(executor: T, id: i32, account_id: i32) -> Result<()>
where
    T: Executor<'a, Database = Postgres>,
{
    query! {
        "
        DELETE FROM upkeep_items
        WHERE id = $1 AND account_id = $2
        ",
        id,
        account_id,
    }
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn patch_due_date_upkeep_item<'a, T>(
    executor: T,
    id: i32,
    account_id: i32,
    due: &NaiveDate,
) -> Result<()>
where
    T: Executor<'a, Database = Postgres>,
{
    query! {
        "
        UPDATE upkeep_items
        SET due = $3
        WHERE id = $1 AND account_id = $2
        ",
        id,
        account_id,
        due
    }
    .execute(executor)
    .await?;
    Ok(())
}
