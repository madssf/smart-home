use sqlx::PgPool;

use crate::db::DbError;
use crate::domain::TempSensor;

pub async fn get_temp_sensors(pool: &PgPool) -> Result<Vec<TempSensor>, DbError> {
    let res: Vec<TempSensor> = sqlx::query_as!(TempSensor, "SELECT * FROM temp_sensors")
        .fetch_all(pool)
        .await?;
    Ok(res)
}

pub async fn get_temp_sensor(pool: &PgPool, id: &str) -> Result<Option<TempSensor>, DbError> {
    Ok(
        sqlx::query_as!(TempSensor, "SELECT * FROM temp_sensors WHERE id = $1", id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn update_temp_sensor(
    pool: &PgPool,
    id: &str,
    battery_level: &i32,
) -> Result<(), DbError> {
    sqlx::query!(
        "UPDATE temp_sensors SET battery_level = $2 WHERE id = $1",
        id,
        battery_level
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_temp_sensor(pool: &PgPool, sensor: &TempSensor) -> Result<(), DbError> {
    sqlx::query!(
        "INSERT INTO temp_sensors (id, room_id) VALUES ($1, $2)",
        sensor.id,
        sensor.room_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_temp_sensor(pool: &PgPool, id: &str) -> Result<(), DbError> {
    sqlx::query!("DELETE FROM temp_sensors WHERE id = $1", id,)
        .execute(pool)
        .await?;
    Ok(())
}
