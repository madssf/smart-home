use log::warn;
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

use crate::clients::shelly_client::{MeterValues, RelayStatus, ShellyClient, ShellyClientError};
use crate::domain::Plug;
use crate::observability::{get_app_environment, Environment};

#[derive(Serialize)]
pub struct PlugStatus {
    name: String,
    room_id: Uuid,
    scheduled: bool,
    is_on: Option<bool>,
    power: Option<f64>,
}
#[derive(Error, Debug)]
pub enum PlugServiceError {
    #[error("ShellyClientError: {0}")]
    ShellyClientError(#[from] ShellyClientError),
}

pub async fn get_plug_statuses(
    plugs: &Vec<Plug>,
    shelly_client: &ShellyClient,
) -> Result<Vec<PlugStatus>, PlugServiceError> {
    let mut plug_statuses = vec![];
    for plug in plugs {
        let (status, meter) = if is_dummy_plug(plug) {
            let (status, meter) = dummy_plug_data(&plug.ip.ip().to_string());
            (Ok(status), Ok(meter))
        } else {
            (
                shelly_client.get_plug_status(plug).await,
                shelly_client.get_meter_values(plug).await,
            )
        };

        if status.is_err() || meter.is_err() {
            warn!(
                "Error getting status or meter values for plug {}",
                plug.name
            );
        }

        plug_statuses.push(PlugStatus {
            name: plug.name.clone(),
            room_id: plug.room_id,
            scheduled: plug.scheduled,
            is_on: status.map(|s| s.ison).ok(),
            power: meter.map(|m| m.power).ok(),
        });
    }
    Ok(plug_statuses)
}

// Returns true if environment is dev and ip starts with 123.123
pub fn is_dummy_plug(plug: &Plug) -> bool {
    get_app_environment() == &Environment::Dev && plug.ip.ip().to_string().starts_with("123.123")
}

// The last three digits is the power in watts
// The digit before that is 0 if the plug is off and 1 if it is on (power is 0 if off)
fn dummy_plug_data(ip: &str) -> (RelayStatus, MeterValues) {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        // Logging a warning as the IP format is unexpected, returning default values
        warn!("IP address format is incorrect, cannot extract dummy data.");
        return (
            RelayStatus {
                ison: false,
                has_timer: false,
                timer_started: 0,
                timer_duration: 0,
                timer_remaining: 0,
                overpower: false,
                source: "".to_string(),
            },
            MeterValues {
                power: 0.0,
                overpower: 0.0,
                is_valid: false,
                timestamp: 0,
                counters: vec![],
                total: 0,
            },
        );
    }

    let power_status = parts[3];
    let (ison, power) = if let Ok(power_status_num) = power_status.parse::<u8>() {
        let is_on = parts[2] == "1";
        (is_on, if is_on { power_status_num as f64 } else { 0.0 })
    } else {
        // Logging a warning as the last part of the IP does not contain valid digit information for dummy data
        warn!(
            "Unexpected format in the last part of the IP address: {}",
            power_status
        );
        (false, 0.0)
    };

    (
        RelayStatus {
            ison,
            has_timer: false,
            timer_started: 0,
            timer_duration: 0,
            timer_remaining: 0,
            overpower: false,
            source: "".to_string(),
        },
        MeterValues {
            power,
            overpower: 0.0,
            is_valid: false,
            timestamp: 0,
            counters: vec![],
            total: 0,
        },
    )
}
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use sqlx::types::ipnetwork::{IpNetwork, Ipv4Network};

    use crate::domain::Plug;

    use super::*;

    #[test]
    fn test_is_dummy_plug() {
        assert!(is_dummy_plug(&Plug {
            id: Uuid::new_v4(),
            name: "Test plug 1".to_string(),
            ip: IpNetwork::from(Ipv4Network::from_str("123.123.0.123").unwrap()),
            username: "test".to_string(),
            password: "test".to_string(),
            room_id: Uuid::new_v4(),
            scheduled: false,
        }));
        assert!(!is_dummy_plug(&Plug {
            id: Uuid::new_v4(),
            name: "Test plug 1".to_string(),
            ip: IpNetwork::from(Ipv4Network::from_str("192.168.0.123").unwrap()),
            username: "test".to_string(),
            password: "test".to_string(),
            room_id: Uuid::new_v4(),
            scheduled: false,
        }));
    }

    #[test]
    fn test_dummy_plug_data_off() {
        let (status, meter) = dummy_plug_data("123.123.0.123");
        assert!(!status.ison);
        assert_eq!(meter.power, 0.0);
    }

    #[test]
    fn test_dummy_plug_data_on() {
        let (status, meter) = dummy_plug_data("123.123.1.30");
        assert!(status.ison);
        assert_eq!(meter.power, 30.0);
    }
}
