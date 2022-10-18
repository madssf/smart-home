use chrono::{Datelike, Duration, NaiveDateTime, NaiveTime, Weekday};
use strum_macros::{Display, EnumString};
use thiserror::Error;

use crate::clients::FirestoreClient;
use crate::firebase_db::{get_schedules, DbError};
use crate::PriceLevel;

#[derive(EnumString, Display, Debug, Clone, Copy)]
pub enum ActionType {
    ON,
    OFF,
}

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
    firestore_client: &FirestoreClient,
    price_level: &PriceLevel,
    time: &NaiveDateTime,
) -> Result<ActionType, SchedulingError> {
    let schedules: Vec<ScheduleData> = get_schedules(firestore_client).await?;

    for schedule in schedules {
        let result = matching_schedule(&schedule, price_level, time);
        if result {
            return Ok(ActionType::ON);
        }
    }

    Ok(ActionType::OFF)
}

fn matching_schedule(
    schedule_data: &ScheduleData,
    price_level: &PriceLevel,
    time: &NaiveDateTime,
) -> bool {
    return price_level == &schedule_data.price_level
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

    use super::{matching_schedule, ScheduleData};

    #[test]
    fn should_only_return_matching_schedule() {
        let schedule_data = ScheduleData {
            price_level: PriceLevel::CHEAP,
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
