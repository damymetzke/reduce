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
use sqlx::{query_as, Executor, Postgres};

pub struct FetchAccount {
    pub email: Arc<str>,
    pub password_hash: Arc<str>,
}

pub async fn fetch_account<'a, T>(executor: T, email: &str) -> Result<FetchAccount>
where
    T: Executor<'a, Database = Postgres>,
{
    Ok(query_as! {
        FetchAccount,
        "
        SELECT email, password_hash FROM accounts
        WHERE email = $1
        LIMIT 1
        ",
        email
    }
    .fetch_one(executor)
    .await?)
}
