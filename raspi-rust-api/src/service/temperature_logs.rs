use std::ops::{Add, Range};

use chrono::{Datelike, Duration, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};

use crate::domain::TemperatureLog;
use crate::now;

pub fn generate_temperature_graph(
    logs: Vec<TemperatureLog>,
    time_period: &TimePeriod,
) -> Vec<RoomTempGraphPoint> {
    let now = now();

    if logs.is_empty() {
        return vec![];
    };

    let mut graph: Vec<RoomTempGraphPoint> = vec![];

    let (graph_start, graph_range) = time_period.graph_base(&now, &logs[0].time);

    for i in graph_range {
        let time = graph_start.add(time_period.graph_step(i + 1));
        let temp = time_period.get_temp_value(time, &logs);
        graph.push(RoomTempGraphPoint {
            label: time_period.format_time(time),
            temp: match temp {
                None => match graph.last() {
                    None => logs[0].temp,
                    Some(last_temp) => last_temp.temp,
                },
                Some(temp) => temp,
            },
        })
    }
    graph
}

#[derive(Serialize, Clone)]
pub struct RoomTempGraphPoint {
    pub label: String,
    pub temp: f64,
}

#[derive(Deserialize)]
pub enum TimePeriod {
    #[serde(rename = "day")]
    Day,
    #[serde(rename = "week")]
    Week,
    #[serde(rename = "month")]
    Month,
}

impl TimePeriod {
    fn graph_base(
        &self,
        now: &NaiveDateTime,
        earliest_log_entry: &NaiveDateTime,
    ) -> (NaiveDateTime, Range<i64>) {
        let earliest_possible = *now - (self.graph_step(self.graph_length()));
        let start_time = match self {
            TimePeriod::Day => *now - Duration::days(1),
            TimePeriod::Week | TimePeriod::Month => {
                if earliest_log_entry > &earliest_possible {
                    NaiveDateTime::new(earliest_log_entry.date(), NaiveTime::from_hms(0, 0, 0))
                } else {
                    NaiveDateTime::new(earliest_possible.date(), NaiveTime::from_hms(0, 0, 0))
                }
            }
        };
        let duration_from_start = *now - start_time;
        let range_end = match self {
            TimePeriod::Day => duration_from_start.num_hours(),
            TimePeriod::Week | TimePeriod::Month => duration_from_start.num_days(),
        };
        (start_time, 0..range_end)
    }

    fn graph_length(&self) -> i64 {
        match self {
            TimePeriod::Day => 24,
            TimePeriod::Week => 7,
            TimePeriod::Month => 30,
        }
    }

    fn graph_step(&self, i: i64) -> Duration {
        match self {
            TimePeriod::Day => Duration::hours(i),
            TimePeriod::Week | TimePeriod::Month => Duration::days(i),
        }
    }

    fn format_time(&self, time: NaiveDateTime) -> String {
        match self {
            TimePeriod::Day => time.time().format("%H:%M").to_string(),
            TimePeriod::Week => time.weekday().to_string(),
            TimePeriod::Month => time.format("%d/%m").to_string(),
        }
    }

    fn get_temp_value(&self, time: NaiveDateTime, logs: &[TemperatureLog]) -> Option<f64> {
        match self {
            TimePeriod::Day => Some(Self::find_closest_temp(
                time,
                &logs.iter().collect::<Vec<&TemperatureLog>>(),
            )),
            TimePeriod::Week | TimePeriod::Month => {
                let day_temps: Vec<&TemperatureLog> = logs
                    .iter()
                    .filter(|log_entry| log_entry.time.date() == time.date())
                    .collect();
                if day_temps.is_empty() {
                    None
                } else {
                    let res = (0..24)
                        .map(|i| {
                            let hour = NaiveTime::from_hms(i, 0, 0);
                            Self::find_closest_temp(
                                NaiveDateTime::new(time.date(), hour),
                                &day_temps,
                            )
                        })
                        .fold(0.0, |acc, temp| acc + temp);
                    Some((res / 2.4).round() / 10.0)
                }
            }
        }
    }

    fn find_closest_temp(time: NaiveDateTime, logs: &[&TemperatureLog]) -> f64 {
        let log_entry =
            logs.iter()
                .filter(|entry| entry.time < time)
                .fold(&logs[0], |prev, curr| {
                    if (prev.time - time).num_seconds().abs()
                        > (curr.time - time).num_seconds().abs()
                    {
                        curr
                    } else {
                        prev
                    }
                });
        (log_entry.temp * 10.0).round() / 10.0
    }
}
