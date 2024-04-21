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

use std::rc::Rc;

use anyhow::{anyhow, Result};
use chrono::NaiveTime;
use itertools::Itertools;

pub fn parse_time(raw: &str) -> Result<NaiveTime> {
    let parts: Rc<_> = raw.trim().split(':').collect();
    let (hour, minute): (Rc<str>, Rc<str>) = match parts.as_ref() {
        [hour, minute] if (hour.len() == 1 || hour.len() == 2) && minute.len() == 2 => {
            ((*hour).into(), (*minute).into())
        }
        [full] if full.len() == 3 => ((*full)[0..1].into(), (*full)[1..3].into()),
        [full] if full.len() == 4 => ((*full)[0..2].into(), (*full)[2..4].into()),
        _ => return Err(anyhow!("Time string is improperly formatted")),
    };

    let hour = hour.parse()?;
    let minute = minute.parse()?;

    NaiveTime::from_hms_opt(hour, minute, 0).ok_or(anyhow!("Could not convert numbers to time"))
}

pub fn normalize_comment(input: &str) -> String {
    let mut lines: Box<_> = input
        .split(';')
        .flat_map(|value| value.split('\n'))
        .map(|line| line.split_whitespace().join(" "))
        .collect();

    lines.sort();

    lines.iter().join("\n")
}

#[cfg(test)]
mod test {
    use std::iter::zip;

    use chrono::NaiveTime;

    use crate::subsystem::time_report::logic::parse_time;

    #[test]
    fn test_parse_time() {
        let input = ["0915", "1245", "930", "10:30", "07:45", "9:00"];

        let expected = [
            NaiveTime::from_hms_opt(9, 15, 0).unwrap(),
            NaiveTime::from_hms_opt(12, 45, 0).unwrap(),
            NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
            NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
            NaiveTime::from_hms_opt(7, 45, 0).unwrap(),
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        ];

        for (input, expected) in zip(input.iter(), expected.iter()) {
            let result = parse_time(input).unwrap();
            assert_eq!(result, *expected);
        }
    }
}
