use std::collections::HashMap;

use anyhow::Context;
use bigdecimal::BigDecimal;
use bigdecimal::{FromPrimitive, ToPrimitive};
use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::db::{DbConfig, DbError};
use crate::domain::{Room, TemperatureLog};

pub struct TemperatureLogsClient {
    db_config: DbConfig,
}

struct TemperatureLogEntity {
    room_id: Uuid,
    temp: BigDecimal,
    time: NaiveDateTime,
}

impl TemperatureLogsClient {
    pub fn new(db_config: DbConfig) -> Self {
        Self { db_config }
    }

    pub async fn get_temp_logs(&self) -> Result<Vec<TemperatureLog>, DbError> {
        let entities = sqlx::query_as!(TemperatureLogEntity, "SELECT * FROM temperature_logs")
            .fetch_all(&self.db_config.pool)
            .await?;

        entities
            .iter()
            .map(|entity| {
                Ok(TemperatureLog {
                    room_id: entity.room_id,
                    time: entity.time,
                    temp: entity.temp.to_f64().context(format!(
                        "Failed to parse floating point number: {}",
                        entity.temp
                    ))?,
                })
            })
            .collect()
    }

    pub async fn get_current_temps(&self, rooms: Vec<Room>) -> Result<HashMap<Uuid, f64>, DbError> {
        let mut temps = HashMap::new();

        for room in rooms {
            let latest_temp = sqlx::query_as!(
                TemperatureLogEntity,
                "SELECT * FROM temperature_logs WHERE room_id = $1 ORDER BY time DESC LIMIT 1",
                room.id,
            )
            .fetch_optional(&self.db_config.pool)
            .await?;

            if let Some(entry) = latest_temp {
                let temp = entry.temp.to_f64().context(format!(
                    "Failed to parse floating point number: {}",
                    entry.temp
                ))?;
                temps.insert(room.id, temp);
            }
        }

        Ok(temps)
    }

    pub async fn create_temp_log(&self, log_entry: TemperatureLog) -> Result<(), DbError> {
        let temp = BigDecimal::from_f64(log_entry.temp)
            .context(format!("Can't convert to big decimal: {}", log_entry.temp))?;
        sqlx::query!(
            r#"
            INSERT INTO temperature_logs (room_id, time, temp)
            VALUES ($1, $2, $3)
        "#,
            log_entry.room_id,
            log_entry.time,
            temp
        )
        .execute(&self.db_config.pool)
        .await?;

        Ok(())
    }
}
