pub mod api;
pub mod configuration;
pub mod db;
pub mod domain;
pub mod observability;
pub mod prices;
mod routes;
pub mod scheduling;
pub mod shelly_client;
pub mod work_handler;

pub fn env_var(name: &str) -> String {
    std::env::var(name)
        .map_err(|e| format!("{}: {}", name, e))
        .expect(&*format!("Missing env var: {}", name))
}
