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

mod account;
mod auth;
mod upkeep;

use axum::Router;

pub struct SectionRegistration {
    pub router: Router,
    pub entry_page: &'static str,
    pub title: &'static str,
}

pub struct ModuleRegistration {
    pub default_module_name: &'static str,
    pub sections: Box<[SectionRegistration]>,
}

pub fn register() -> ModuleRegistration {
    let sections = Box::from([auth::register(), account::register(), upkeep::register()]);
    ModuleRegistration {
        default_module_name: "/core",
        sections,
    }
}
