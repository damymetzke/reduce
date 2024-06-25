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

use askama::Template;

use crate::extensions::{AuthorizedSession, Session};

#[derive(Clone)]
pub struct PartItem {
    pub id: i32,
    pub description: Arc<str>,
    pub due: Arc<str>,
    pub cooldown: Arc<str>,
    pub render_complete: bool,
}

#[derive(Template)]
#[template(path = "modules/upkeep/index.html")]
pub struct IndexTemplate {
    pub due_items: Box<[PartItem]>,
    pub backlog: Box<[PartItem]>,
    pub session: Session,
    pub authorized_session: AuthorizedSession,
}
