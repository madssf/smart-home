use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

use crate::clients::shelly_client::{ShellyClient, ShellyClientError};
use crate::domain::Plug;

#[derive(Serialize)]
pub struct PlugStatus {
    name: String,
    room_id: Uuid,
    is_on: bool,
    power: f64,
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
        let status = shelly_client.get_plug_status(plug).await?;
        let meter = shelly_client.get_meter_values(plug).await?;
        plug_statuses.push(PlugStatus {
            name: plug.name.clone(),
            room_id: plug.room_id,
            is_on: status.ison,
            power: meter.power,
        })
    }
    Ok(plug_statuses)
}
