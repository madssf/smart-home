use std::collections::HashMap;

use anyhow::Context;
use bigdecimal::BigDecimal;
use bigdecimal::{FromPrimitive, ToPrimitive};
use chrono::NaiveDateTime;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::DbError;
use crate::domain::{Room, TemperatureLog};

struct TemperatureLogEntity {
    room_id: Uuid,
    temp: BigDecimal,
    time: NaiveDateTime,
}

fn to_domain(entities: Vec<TemperatureLogEntity>) -> Result<Vec<TemperatureLog>, DbError> {
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

pub async fn get_temp_logs(pool: &PgPool) -> Result<Vec<TemperatureLog>, DbError> {
    let entities = sqlx::query_as!(TemperatureLogEntity, "SELECT * FROM temperature_logs")
        .fetch_all(pool)
        .await?;

    to_domain(entities)
}

pub async fn get_room_temp_logs(
    pool: &PgPool,
    room_id: &Uuid,
) -> Result<Vec<TemperatureLog>, DbError> {
    let entities = sqlx::query_as!(
        TemperatureLogEntity,
        "SELECT * FROM temperature_logs WHERE room_id = $1",
        room_id
    )
    .fetch_all(pool)
    .await?;

    to_domain(entities)
}

pub async fn get_current_temps(
    pool: &PgPool,
    rooms: Vec<Room>,
) -> Result<HashMap<Uuid, f64>, DbError> {
    let mut temps = HashMap::new();

    for room in rooms {
        let latest_temp = sqlx::query_as!(
            TemperatureLogEntity,
            "SELECT * FROM temperature_logs WHERE room_id = $1 ORDER BY time DESC LIMIT 1",
            room.id,
        )
        .fetch_optional(pool)
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

pub async fn create_temp_log(pool: &PgPool, log_entry: TemperatureLog) -> Result<(), DbError> {
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
    .execute(pool)
    .await?;

    Ok(())
}
