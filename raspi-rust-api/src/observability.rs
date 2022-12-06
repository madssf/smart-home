use strum_macros::{Display, EnumString};

use crate::env_var;

#[derive(Display, EnumString, Debug, Eq, PartialEq)]
pub enum Environment {
    Dev,
    Production,
}

pub fn get_app_environment() -> &'static Environment {
    let env_var = env_var("ENVIRONMENT");
    match env_var.as_str() {
        "production" => &Environment::Production,
        _ => &Environment::Dev,
    }
}
