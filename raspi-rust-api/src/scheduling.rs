use chrono::{Datelike, Duration, NaiveDateTime, NaiveTime, Weekday};
use thiserror::Error;

use crate::db::DbError;
use crate::domain::{ActionType, PriceLevel, Schedule};

#[derive(Debug, PartialEq, Eq)]
pub struct ScheduleData {
    pub price_level: PriceLevel,
    pub days: Vec<Weekday>,
    pub windows: Vec<(NaiveTime, Duration)>,
}

#[derive(Error, Debug)]
pub enum SchedulingError {
    #[error("DbError: {0}")]
    FailedToGetSchedules(#[from] DbError),
}

pub async fn get_action(
    schedules: Vec<Schedule>,
    price_level: &PriceLevel,
    time: &NaiveDateTime,
) -> Result<ActionType, SchedulingError> {
    for schedule in schedules {
        let result = matching_schedule(&schedule, price_level, time);
        if result {
            return Ok(ActionType::ON);
        }
    }

    Ok(ActionType::OFF)
}

fn matching_schedule(schedule: &Schedule, price_level: &PriceLevel, time: &NaiveDateTime) -> bool {
    return price_level == &schedule.price_level
        && schedule.days.contains(&time.weekday())
        && schedule
            .time_windows
            .iter()
            .map(|window| window.0 < time.time() && window.1 > time.time())
            .any(|x| x);
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDateTime, NaiveTime, Weekday};
    use uuid::Uuid;

    use crate::domain::{PriceLevel, Schedule};

    use super::matching_schedule;

    #[test]
    fn should_only_return_matching_schedule() {
        let schedule = Schedule {
            id: Uuid::new_v4(),
            price_level: PriceLevel::CHEAP,
            days: vec![Weekday::Mon, Weekday::Tue],
            time_windows: vec![(NaiveTime::from_hms(12, 0, 0), NaiveTime::from_hms(13, 0, 0))],
            temp: 0.0,
            room_ids: vec![],
        };

        assert!(matching_schedule(
            &schedule,
            &PriceLevel::CHEAP,
            &NaiveDateTime::from_timestamp(1663592399, 0)
        ));

        // Wrong level
        assert!(!matching_schedule(
            &schedule,
            &PriceLevel::NORMAL,
            &NaiveDateTime::from_timestamp(1663592399, 0)
        ));

        // Wrong day
        assert!(!matching_schedule(
            &schedule,
            &PriceLevel::CHEAP,
            &NaiveDateTime::from_timestamp(1663765199, 0)
        ));

        // Wrong time
        assert!(!matching_schedule(
            &schedule,
            &PriceLevel::CHEAP,
            &NaiveDateTime::from_timestamp(1663678800, 0)
        ));
    }
}
