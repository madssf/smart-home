use std::{fs, str::FromStr};

use chrono::{Datelike, Duration, NaiveDateTime, NaiveTime, Weekday};
use serde::Deserialize;

use crate::PriceLevel;

pub enum ActionType {
    ON,
    OFF,
}

impl ActionType {
    pub fn to_string(&self) -> &str {
        match &self {
            ActionType::ON => "on",
            ActionType::OFF => "off",
        }
    }
}

#[derive(Deserialize, Debug)]
struct ScheduleDataJson {
    level: String,
    days: Vec<String>,
    hours: Vec<HoursJson>,
}

#[derive(Deserialize, Debug)]
struct HoursJson {
    from: String,
    to: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ScheduleData {
    level: PriceLevel,
    days: Vec<Weekday>,
    windows: Vec<(NaiveTime, Duration)>,
}

impl ScheduleData {
    fn from_json(json: &ScheduleDataJson) -> Self {
        ScheduleData {
            level: PriceLevel::from_str(&json.level).expect("Failed to parse price level"),
            days: json
                .days
                .iter()
                .map(|day| Weekday::from_str(day).expect("Failed to parse weekday"))
                .collect::<Vec<Weekday>>(),
            windows: json
                .hours
                .iter()
                .map(|x| {
                    let start = NaiveTime::from_str(&x.from).expect("Failed to parse NaiveTime");
                    let stop = NaiveTime::from_str(&x.to).expect("Failed to parse NaiveTime");
                    let duration = stop.signed_duration_since(start);
                    (start, duration)
                })
                .collect::<Vec<(NaiveTime, Duration)>>(),
        }
    }
}

fn read_schedules_from_file() -> String {
    fs::read_to_string("./schedule.json").expect("Missing schedule.json")
}

fn parse_schedules_string_to_json(schedules: &str) -> Vec<ScheduleDataJson> {
    let schedules: Vec<ScheduleDataJson> =
        serde_json::from_str(schedules).expect("Invalid json format");
    schedules
}

fn parse_schedules_json_to_domain(schedules: &[ScheduleDataJson]) -> Vec<ScheduleData> {
    schedules
        .iter()
        .map(ScheduleData::from_json)
        .collect::<Vec<ScheduleData>>()
}

pub fn get_action(price_level: &PriceLevel, time: &NaiveDateTime) -> ActionType {
    let schedules_str = read_schedules_from_file();
    let schedules_json = parse_schedules_string_to_json(&schedules_str);
    let schedules = parse_schedules_json_to_domain(&schedules_json);

    for schedule in schedules {
        let result = matching_schedule(&schedule, price_level, time);
        if result {
            return ActionType::ON;
        }
    }

    ActionType::OFF
}

fn matching_schedule(
    schedule_data: &ScheduleData,
    price_level: &PriceLevel,
    time: &NaiveDateTime,
) -> bool {
    return price_level == &schedule_data.level
        && schedule_data.days.contains(&time.weekday())
        && schedule_data
            .windows
            .iter()
            .map(|window| window.0 < time.time() && window.0 + window.1 > time.time())
            .any(|x| x);
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, NaiveDateTime, NaiveTime, Weekday};

    use crate::PriceLevel;

    use super::{
        matching_schedule, parse_schedules_json_to_domain, HoursJson, ScheduleData,
        ScheduleDataJson,
    };

    #[test]
    fn should_parse_to_domain() {
        let json = vec![ScheduleDataJson {
            level: String::from("CHEAP"),
            days: vec![String::from("MON"), String::from("TUE")],
            hours: vec![HoursJson {
                from: String::from("12:00:00"),
                to: String::from("13:00:00"),
            }],
        }];
        let expected = vec![ScheduleData {
            level: PriceLevel::CHEAP,
            days: vec![Weekday::Mon, Weekday::Tue],
            windows: vec![(NaiveTime::from_hms(12, 0, 0), Duration::hours(1))],
        }];
        assert_eq!(parse_schedules_json_to_domain(&json), expected)
    }

    #[test]
    fn should_only_return_matching_schedule() {
        let schedule_data = ScheduleData {
            level: PriceLevel::CHEAP,
            days: vec![Weekday::Mon, Weekday::Tue],
            windows: vec![(NaiveTime::from_hms(12, 0, 0), Duration::hours(1))],
        };

        assert!(matching_schedule(
            &schedule_data,
            &PriceLevel::CHEAP,
            &NaiveDateTime::from_timestamp(1663592399, 0)
        ));

        // Wrong level
        assert!(!matching_schedule(
            &schedule_data,
            &PriceLevel::NORMAL,
            &NaiveDateTime::from_timestamp(1663592399, 0)
        ));

        // Wrong day
        assert!(!matching_schedule(
            &schedule_data,
            &PriceLevel::CHEAP,
            &NaiveDateTime::from_timestamp(1663765199, 0)
        ));

        // Wrong time
        assert!(!matching_schedule(
            &schedule_data,
            &PriceLevel::CHEAP,
            &NaiveDateTime::from_timestamp(1663678800, 0)
        ));
    }
}
