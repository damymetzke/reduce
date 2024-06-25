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

use chrono::NaiveDateTime;

#[derive(Clone, Debug)]
pub enum Session {
    Guest,
    Expired {
        since: NaiveDateTime,
    },
    Authenticated {
        csrf_token: Arc<str>,
        session_id: i32,
    },
}

#[derive(Clone, Debug)]
pub struct AuthorizedSession {
    pub csrf_token: Arc<str>,
    pub session_id: i32,
}

impl From<AuthorizedSession> for Session {
    fn from(
        AuthorizedSession {
            csrf_token,
            session_id,
        }: AuthorizedSession,
    ) -> Self {
        Session::Authenticated {
            csrf_token,
            session_id,
        }
    }
}
