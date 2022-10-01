use strum_macros::{Display, EnumString};

pub mod db;
pub mod prices;
pub mod scheduling;
pub mod shelly_client;

#[derive(Debug, EnumString, Display, Eq, PartialEq)]
pub enum PriceLevel {
    CHEAP,
    NORMAL,
    EXPENSIVE,
}

#[derive(Debug)]
pub struct Plug {
    pub name: String,
    pub ip: String,
    pub username: String,
    pub password: String,
}
