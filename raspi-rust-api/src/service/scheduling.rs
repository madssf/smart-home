use chrono::{Datelike, Duration, NaiveDateTime, NaiveTime, Weekday};
use thiserror::Error;

use crate::db::DbError;
use crate::domain::{PriceLevel, Schedule};

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

pub fn find_matching_schedule(
    schedules: Vec<Schedule>,
    price_level: &PriceLevel,
    time: &NaiveDateTime,
) -> Option<Schedule> {
    schedules.into_iter().find(|schedule| {
        price_level == &schedule.price_level
            && schedule.days.contains(&time.weekday())
            && schedule
                .time_windows
                .iter()
                .map(|window| window.0 < time.time() && window.1 > time.time())
                .any(|x| x)
    })
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDateTime, NaiveTime, Weekday};
    use uuid::Uuid;

    use crate::domain::{PriceLevel, Schedule};

    use super::find_matching_schedule;

    #[test]
    fn should_only_return_matching_schedule() {
        let room_id = Uuid::new_v4();

        let schedule = Schedule {
            id: Uuid::new_v4(),
            price_level: PriceLevel::CHEAP,
            days: vec![Weekday::Mon, Weekday::Tue],
            time_windows: vec![(NaiveTime::from_hms(12, 0, 0), NaiveTime::from_hms(13, 0, 0))],
            temp: 0.0,
            room_ids: vec![room_id],
        };

        let schedule_2 = Schedule {
            id: Uuid::new_v4(),
            price_level: PriceLevel::NORMAL,
            days: vec![Weekday::Mon, Weekday::Tue],
            time_windows: vec![(NaiveTime::from_hms(12, 0, 0), NaiveTime::from_hms(13, 0, 0))],
            temp: 0.0,
            room_ids: vec![room_id],
        };

        let schedule_3 = Schedule {
            id: Uuid::new_v4(),
            price_level: PriceLevel::CHEAP,
            days: vec![Weekday::Mon, Weekday::Tue],
            time_windows: vec![(NaiveTime::from_hms(12, 0, 0), NaiveTime::from_hms(13, 0, 0))],
            temp: 0.0,
            room_ids: vec![Uuid::new_v4()],
        };

        let schedules = vec![schedule.clone(), schedule_2.clone(), schedule_3];

        assert_eq!(
            find_matching_schedule(
                schedules.clone(),
                &PriceLevel::CHEAP,
                &NaiveDateTime::from_timestamp(1663592399, 0)
            ),
            Some(schedule)
        );

        // Wrong level
        assert_eq!(
            find_matching_schedule(
                schedules.clone(),
                &PriceLevel::NORMAL,
                &NaiveDateTime::from_timestamp(1663592399, 0)
            ),
            Some(schedule_2)
        );

        // Wrong day
        assert_eq!(
            find_matching_schedule(
                schedules.clone(),
                &PriceLevel::CHEAP,
                &NaiveDateTime::from_timestamp(1663765199, 0)
            ),
            None
        );

        // Wrong time
        assert_eq!(
            find_matching_schedule(
                schedules,
                &PriceLevel::CHEAP,
                &NaiveDateTime::from_timestamp(1663678800, 0)
            ),
            None
        );
    }
}
