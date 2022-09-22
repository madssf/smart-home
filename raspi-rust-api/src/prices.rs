use strum_macros::{EnumString, Display};
use tibber::{TibberSession, PriceLevel as TPriceLevel, PriceInfo as TPriceInfo};
use std::env;

#[derive(Debug, EnumString, Display, Eq, PartialEq)]
pub enum PriceLevel {
    CHEAP,
    NORMAL,
    EXPENSIVE
}

impl PriceLevel {
    fn from_tibber_price_level(tibber_price_level: &TPriceLevel) -> Self {
        match tibber_price_level {
            TPriceLevel::VeryCheap => PriceLevel::CHEAP,
            TPriceLevel::Cheap => PriceLevel::CHEAP,
            TPriceLevel::Normal => PriceLevel::NORMAL,
            TPriceLevel::Expensive => PriceLevel::EXPENSIVE,
            TPriceLevel::VeryExpensive => PriceLevel::EXPENSIVE,
            TPriceLevel::Other(_) => PriceLevel::NORMAL,
            TPriceLevel::None => PriceLevel::NORMAL,
        }
    }
}

pub struct PriceInfo {
    pub amount: f64,
    pub currency: String,
    pub level: PriceLevel
}

impl PriceInfo {
    fn from_tibber_price_info(tibber_price_info: &TPriceInfo) -> Self {
        PriceInfo { amount: tibber_price_info.total, currency: String::from(&tibber_price_info.currency), level: PriceLevel::from_tibber_price_level(&tibber_price_info.level) }
    }
}

pub fn get_current_price() -> PriceInfo {

    let api_token = env::var("TIBBER_API_TOKEN").expect("Mssing TIBBER_API_TOKEN env var");
    let conn = TibberSession::new(api_token);

    let user = conn.get_user().unwrap();

    if user.homes.len() < 1 {
        panic!("No home found for user: {:?} ", user)
    }
    let tibber_price_info = conn.get_current_price(&user.homes[0]);
    PriceInfo::from_tibber_price_info(&tibber_price_info.unwrap())
}
