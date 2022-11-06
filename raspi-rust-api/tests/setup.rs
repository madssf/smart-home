use std::collections::HashMap;

use chrono::{NaiveTime, Weekday};

use rust_home::domain::{PriceLevel, Room, Schedule};

pub fn schedule(rooms: Vec<&Room>) -> Schedule {
    Schedule::new(
        HashMap::from([
            (PriceLevel::VeryCheap, 18.0),
            (PriceLevel::VeryExpensive, 20.0),
        ]),
        vec![Weekday::Mon],
        vec![(
            NaiveTime::from_hms(00, 00, 00),
            NaiveTime::from_hms(12, 0, 0),
        )],
        vec![rooms[0].id],
    )
    .expect("Couldn't create schedule")
}
