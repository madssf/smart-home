use strum_macros::{Display, EnumString};

pub mod db;
pub mod plugs;
pub mod prices;
pub mod scheduling;
pub mod shelly_client;

#[derive(Debug, EnumString, Display, Eq, PartialEq)]
pub enum PriceLevel {
    CHEAP,
    NORMAL,
    EXPENSIVE,
}
