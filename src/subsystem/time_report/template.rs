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

use std::{borrow::Borrow, sync::Arc};

use askama::Template;

use super::database::TimeReportItemDTO;

#[derive(Template)]
#[template(path = "api/time-reports/list-item.html", escape = "none")]
pub struct TimeReportItemTemplate {
    category: Box<str>,
    start_time: Box<str>,
    end_time: Box<str>,
    i: u16,
    value: Box<str>,
}

#[derive(Template)]
#[template(path = "api/time-reports/picker.html", escape = "none")]
pub struct TimeReportPickerTemplate {
    pub reports: Box<[TimeReportItemTemplate]>,
}

#[derive(Template)]
#[template(path = "time-reports.html", escape = "none")]
pub struct TimeReportsTemplate {
    pub categories: Box<[Arc<str>]>,
    pub time_reports: TimeReportIndexTemplate,
}

#[derive(Template)]
#[template(path = "api/time-reports/index.html", escape = "none")]
pub struct TimeReportIndexTemplate {
    pub time_report_picker: TimeReportPickerTemplate,
    pub picker_date: Box<str>,
}

#[derive(Template)]
#[template(path = "api/time-reports/add/result.html", escape = "none")]
pub struct TimeReportInsertResultTemplate {
    pub date: Arc<str>,
    pub num_insertions: u64,
}

#[derive(Template)]
#[template(path = "api/time-reports/delete_result.html", escape = "none")]
pub struct TimeReportDeleteResultTemplate {
    pub num_deleted: u64,
}

impl<T> From<(u16, T)> for TimeReportItemTemplate
where
    T: Borrow<TimeReportItemDTO>,
{
    fn from((i, item): (u16, T)) -> Self {
        let item = item.borrow();
        let (end_time, value) = item
            .end_time
            .map(|end_time| {
                (
                    end_time.format("%H:%M").to_string().into(),
                    end_time.format("%H:%M:%S").to_string().into(),
                )
            })
            .unwrap_or(("".into(), "".into()));

        TimeReportItemTemplate {
            category: item.name.clone(),
            start_time: item.start_time.format("%H:%M").to_string().into(),
            end_time,
            i,
            value,
        }
    }
}

impl<T, U> From<T> for TimeReportPickerTemplate
where
    T: IntoIterator<Item = U>,
    (u16, U): Into<TimeReportItemTemplate>,
{
    fn from(value: T) -> Self {
        TimeReportPickerTemplate {
            reports: value
                .into_iter()
                .enumerate()
                .map(|(i, item)| (i as u16, item).into())
                .collect(),
        }
    }
}
