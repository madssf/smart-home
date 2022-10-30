use std::env;

use chrono::{NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;

pub mod api;
pub mod clients;
pub mod configuration;
pub mod db;
pub mod domain;
pub mod observability;
pub mod routes;
pub mod service;
pub mod work_handler;

pub fn env_var(name: &str) -> String {
    env::var(name)
        .map_err(|e| format!("{}: {}", name, e))
        .expect(&*format!("Missing env var: {}", name))
}

pub fn now() -> NaiveDateTime {
    let utc = Utc::now().naive_utc();
    let tz: Tz = env::var("TIME_ZONE")
        .expect("Missing TIME_ZONE env var")
        .parse()
        .expect("Failed to parse timezone");
    tz.from_utc_datetime(&utc).naive_local()
}
