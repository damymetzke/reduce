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

use std::{rc::Rc, sync::Arc};

use askama::{DynTemplate, Template};

use crate::extensions::{AuthorizedSession, Session};

#[derive(Template)]
#[template(path = "sections/account/index.html")]
pub struct IndexTemplate {
    pub session: Session,
    pub current_methods: Rc<[Box<dyn DynTemplate>]>,
    pub new_methods: Rc<[Box<dyn DynTemplate>]>,
}

#[derive(Template)]
#[template(path = "sections/account/authenticate-methods/current-email-password.part.html")]
pub struct CurrentEmailPasswordPartTemplate {
    pub email: Arc<str>,
    pub authorized_session: AuthorizedSession,
}

#[derive(Template)]
#[template(path = "sections/account/authenticate-methods/new-email-password.part.html")]
pub struct NewEmailPasswordPartTemplate {
    pub authorized_session: AuthorizedSession,
}
