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
use sqlx::{query, Executor, Postgres};

pub async fn fetch_email_for_login<'a, T>(executor: T, user_id: i32) -> Result<Option<Arc<str>>>
where
    T: Executor<'a, Database = Postgres>,
{
    Ok(
        match query! {
            "
        SELECT email FROM email_password_logins
        WHERE account_id = $1
        LIMIT 1
        ",
            user_id
        }
        .fetch_one(executor)
        .await
        {
            Ok(result) => Some(result.email.into()),
            Err(sqlx::Error::RowNotFound) => None,
            Err(err) => return Err(err.into()),
        },
    )
}
