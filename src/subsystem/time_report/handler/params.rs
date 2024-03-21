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

use std::{collections::HashMap, rc::Rc, sync::Arc};

use anyhow::{anyhow, Result};
use chrono::{NaiveDate, NaiveTime};
use serde::{
    de::{Error, MapAccess, Visitor},
    Deserialize,
};

#[derive(Debug, Clone)]
pub struct PostIndexItem {
    pub category: Arc<str>,
    pub start_time: NaiveTime,
    pub end_time: Option<NaiveTime>,
    pub comment: Arc<str>,
}

#[derive(Debug)]
pub struct PostIndexParams {
    pub date: NaiveDate,
    pub items: Arc<[PostIndexItem]>,
}

#[derive(Debug)]
pub struct DeleteIndexParams {
    pub date: NaiveDate,
    pub selected_times: Arc<[NaiveTime]>,
}

fn parse_time(raw: &str) -> Result<NaiveTime> {
    let parts: Rc<_> = raw.trim().split(':').collect();
    let (hour, minute): (Rc<str>, Rc<str>) = match parts.as_ref() {
        [hour, minute] if (hour.len() == 1 || hour.len() == 2) && minute.len() == 2 => {
            ((*hour).into(), (*minute).into())
        }
        [full] if full.len() == 3 => ((*full)[0..1].into(), (*full)[1..2].into()),
        [full] if full.len() == 4 => ((*full)[0..2].into(), (*full)[2..4].into()),
        _ => return Err(anyhow!("Time string is improperly formatted")),
    };

    let hour = hour.parse()?;
    let minute = minute.parse()?;

    NaiveTime::from_hms_opt(hour, minute, 0).ok_or(anyhow!("Could not convert numbers to time"))
}

impl<'de> Deserialize<'de> for PostIndexParams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PostIndexParamsVisitor;

        impl<'de> Visitor<'de> for PostIndexParamsVisitor {
            type Value = PostIndexParams;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct PostIndexParams")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut date: Option<NaiveDate> = None;
                let mut categories: HashMap<u16, Arc<str>> = HashMap::new();
                let mut start_times: HashMap<u16, NaiveTime> = HashMap::new();
                let mut end_times: HashMap<u16, NaiveTime> = HashMap::new();
                let mut comments: HashMap<u16, Arc<str>> = HashMap::new();

                while let Some(key) = map.next_key::<Rc<str>>()? {
                    if key.as_ref() == "date" {
                        let date_raw: Rc<str> = map.next_value()?;
                        date = Some(
                            NaiveDate::parse_from_str(date_raw.as_ref(), "%Y-%m-%d")
                                .map_err(Error::custom)?,
                        );
                        continue;
                    }

                    let (index, key) = if let Some((index, key)) = key.split_once("--") {
                        (index, key)
                    } else {
                        continue;
                    };

                    let index: u16 = index.parse().map_err(Error::custom)?;

                    let value: &str = map.next_value()?;
                    if value.is_empty() {
                        break;
                    }
                    match key {
                        "category" => {
                            categories.insert(index, value.into());
                        }
                        "start-time" => {
                            start_times.insert(index, parse_time(value).map_err(Error::custom)?);
                        }
                        "end-time" => {
                            end_times.insert(index, parse_time(value).map_err(Error::custom)?);
                        }
                        "comment" => {
                            comments.insert(index, value.into());
                        }
                        _ => {}
                    };
                }

                let mut index = 0;
                let mut items = Vec::new();
                let empty_string: Arc<str> = Arc::from("");

                for i in 0.. {
                    match (categories.get(&i), start_times.get(&i)) {
                        (Some(category), Some(start_time)) => {
                            let end_time = end_times.get(&index).cloned();
                            let comment = comments.get(&i).cloned().unwrap_or(empty_string.clone());
                            let item = PostIndexItem {
                                category: category.clone(),
                                start_time: *start_time,
                                end_time,
                                comment,
                            };
                            items.push(item);
                            index += 1;
                        }
                        (None, None) => {
                            index = i;
                            break;
                        }
                        _ => {
                            return Err(serde::de::Error::custom(
                                "For any index, both category and start_time must be defined.",
                            ));
                        }
                    }
                }

                if categories.len() != index as usize
                    || start_times.len() != index as usize
                    || end_times.keys().any(|i| i >= &index)
                {
                    return Err(serde::de::Error::custom(
                        "The number of items in the collections must be the same for each index.",
                    ));
                }

                Ok(PostIndexParams {
                    date: date.ok_or_else(|| serde::de::Error::missing_field("date"))?,
                    items: items.into(),
                })
            }
        }
        deserializer.deserialize_any(PostIndexParamsVisitor)
    }
}

impl<'de> Deserialize<'de> for DeleteIndexParams {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DeleteIndexParamsVisitor;

        impl<'de> Visitor<'de> for DeleteIndexParamsVisitor {
            type Value = DeleteIndexParams;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct PostIndexParams")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut date = None;
                let mut selected_times = Vec::new();

                while let Some(key) = map.next_key::<Rc<str>>()? {
                    if key.as_ref() == "date" {
                        let date_raw: Rc<str> = map.next_value()?;
                        date = Some(
                            NaiveDate::parse_from_str(date_raw.as_ref(), "%Y-%m-%d")
                                .map_err(Error::custom)?,
                        );
                        continue;
                    };

                    if !key.ends_with("--select-item") {
                        continue;
                    };

                    let value: Rc<str> = map.next_value()?;

                    let time = NaiveTime::parse_from_str(value.as_ref(), "%H:%M:%S")
                        .map_err(Error::custom)?;
                    selected_times.push(time);
                }

                let date = date.ok_or(Error::custom("Missing date"))?;
                let selected_times = selected_times.into();

                Ok(DeleteIndexParams {
                    date,
                    selected_times,
                })
            }
        }
        deserializer.deserialize_any(DeleteIndexParamsVisitor)
    }
}
