pub mod api;
pub mod clients;
pub mod configuration;
pub mod db;
pub mod domain;
pub mod firebase_db;
pub mod observability;
pub mod prices;
mod routes;
pub mod scheduling;
pub mod shelly_client;
pub mod work_handler;

pub fn env_var(name: &str) -> String {
    std::env::var(name)
        .map_err(|e| format!("{}: {}", name, e))
        .expect(&*format!("Missing config env var: {}", name))
}
