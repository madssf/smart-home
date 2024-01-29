use log::warn;
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

use crate::clients::shelly_client::{ShellyClient, ShellyClientError};
use crate::domain::Plug;

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
        let status = shelly_client.get_plug_status(plug).await;
        let meter = shelly_client.get_meter_values(plug).await;

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
