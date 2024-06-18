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

mod auth;

use axum::Router;

use crate::template_extend::NavigationLink;

pub struct SectionRegistration {
    pub default_section_name: &'static str,
    pub router: Router,
    pub navigation_links: Box<[NavigationLink]>,
}

pub struct ModuleRegistration {
    pub default_module_name: &'static str,
    pub sections: Box<[SectionRegistration]>,
}

pub fn register() -> ModuleRegistration {
    let sections = Box::from([auth::register()]);
    ModuleRegistration {
        default_module_name: "/core",
        sections,
    }
}
