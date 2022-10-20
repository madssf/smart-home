use std::str::FromStr;

use anyhow::Context;
use bigdecimal::{FromPrimitive, ToPrimitive};
use chrono::{NaiveTime, Weekday};
use sqlx::types::BigDecimal;
use uuid::Uuid;

use crate::db::{DbConfig, DbError};
use crate::domain::{PriceLevel, Schedule};

pub struct SchedulesClient {
    db_config: DbConfig,
}

struct RoomScheduleEntity {
    room_id: Uuid,
    schedule_id: Uuid,
}

struct ScheduleEntity {
    id: Uuid,
    temp: BigDecimal,
    price_level: String,
    days: Vec<String>,
    time_windows: Vec<String>,
}

impl ScheduleEntity {
    fn from_domain(domain: &Schedule) -> Result<ScheduleEntity, anyhow::Error> {
        let temp = BigDecimal::from_f64(domain.temp)
            .context(format!("Can't convert to big decimal: {}", domain.temp))?;
        let price_level = domain.price_level.to_string();
        let days: Vec<String> = domain.days.iter().map(|day| day.to_string()).collect();
        let time_windows: Vec<String> = domain
            .time_windows
            .iter()
            .map(|time_window| {
                let time_window = *time_window;
                format!("{}/{}", time_window.0, time_window.1)
            })
            .collect();
        Ok(ScheduleEntity {
            id: domain.id,
            temp,
            price_level,
            days,
            time_windows,
        })
    }
}

impl SchedulesClient {
    pub fn new(db_config: DbConfig) -> Self {
        Self { db_config }
    }

    pub async fn get_schedules(&self) -> Result<Vec<Schedule>, DbError> {
        let entities: Vec<ScheduleEntity> =
            sqlx::query_as!(ScheduleEntity, "SELECT * FROM schedules")
                .fetch_all(&self.db_config.pool)
                .await?;

        let room_schedules: Vec<RoomScheduleEntity> =
            sqlx::query_as!(RoomScheduleEntity, "SELECT * FROM room_schedules")
                .fetch_all(&self.db_config.pool)
                .await?;

        entities
            .iter()
            .map(|entity| {
                let price_level = PriceLevel::from_str(&entity.price_level).context(format!(
                    "Failed to parse PriceLevel: {}",
                    &entity.price_level
                ))?;

                let days = parse_weekdays(entity.days.clone())?;
                let time_windows = parse_time_windows(entity.time_windows.clone())?;
                let room_ids = room_schedules
                    .iter()
                    .filter(|room_schedule| room_schedule.schedule_id == entity.id)
                    .map(|room_schedule| room_schedule.room_id)
                    .collect();

                Ok(Schedule {
                    id: entity.id,
                    price_level,
                    days,
                    time_windows,
                    room_ids,
                    temp: entity.temp.to_f64().context(format!(
                        "Failed to parse floating point number: {}",
                        entity.temp
                    ))?,
                })
            })
            .collect()
    }
    pub async fn create_schedule(&self, new_schedule: Schedule) -> Result<(), DbError> {
        let mut tx = self.db_config.pool.begin().await?;
        let entity = ScheduleEntity::from_domain(&new_schedule)?;
        sqlx::query!(
            r#"
        INSERT INTO schedules (id, temp, price_level, days, time_windows)
        VALUES ($1, $2, $3, $4, $5)
        "#,
            new_schedule.id,
            entity.temp,
            entity.price_level,
            &entity.days,
            &entity.time_windows
        )
        .execute(&mut tx)
        .await?;

        for room_id in new_schedule.room_ids {
            sqlx::query!(
                r#"
            INSERT INTO room_schedules (room_id, schedule_id)
            VALUES ($1, $2)
            "#,
                room_id,
                entity.id,
            )
            .execute(&mut tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn update_schedule(&self, schedule: Schedule) -> Result<(), DbError> {
        let mut tx = self.db_config.pool.begin().await?;

        let entity = ScheduleEntity::from_domain(&schedule)?;

        sqlx::query!(
            r#"
            UPDATE schedules
            SET temp = $2, price_level = $3, days = $4, time_windows = $5
            WHERE id = $1
            "#,
            entity.id,
            entity.temp,
            entity.price_level,
            &entity.days,
            &entity.time_windows,
        )
        .execute(&mut tx)
        .await?;

        let existing_room_schedules: Vec<RoomScheduleEntity> = sqlx::query_as!(
            RoomScheduleEntity,
            "SELECT * FROM room_schedules WHERE schedule_id = $1",
            schedule.id
        )
        .fetch_all(&mut tx)
        .await?;

        for room_id in &schedule.room_ids {
            if !existing_room_schedules
                .iter()
                .any(|x| x.room_id == *room_id)
            {
                sqlx::query!(
                    r#"
                INSERT INTO room_schedules (room_id, schedule_id)
                VALUES ($1, $2)
                "#,
                    room_id,
                    entity.id,
                )
                .execute(&mut tx)
                .await?;
            }
        }

        for existing in existing_room_schedules {
            if !schedule.room_ids.contains(&existing.room_id) {
                sqlx::query!(
                    r#"
                    DELETE FROM room_schedules WHERE room_id = $1 AND schedule_id = $2
                    "#,
                    existing.room_id,
                    schedule.id
                )
                .execute(&mut tx)
                .await?;
            }
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn delete_schedule(&self, id: &Uuid) -> Result<(), DbError> {
        let mut tx = self.db_config.pool.begin().await?;

        sqlx::query!("DELETE FROM room_schedules WHERE schedule_id = $1", id)
            .execute(&mut tx)
            .await?;

        sqlx::query!("DELETE FROM schedules WHERE id = $1", id)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }
}

fn parse_weekdays(day_entities: Vec<String>) -> Result<Vec<Weekday>, DbError> {
    let mut days = vec![];
    for day in day_entities {
        let weekday =
            Weekday::from_str(&day).context(format!("Failed to parse weekday: {}", day))?;
        days.push(weekday);
    }
    Ok(days)
}

fn parse_time_windows(
    time_window_entities: Vec<String>,
) -> Result<Vec<(NaiveTime, NaiveTime)>, DbError> {
    let mut time_windows = vec![];
    for window in time_window_entities {
        let mut split = window.split('/');
        let from = NaiveTime::from_str(
            split
                .next()
                .context(format!("Failed to parse time window: {}", window))?,
        )
        .context(format!("Failed to parse time: {}", window))?;
        let to = NaiveTime::from_str(
            split
                .next()
                .context(format!("Failed to parse time window: {}", window))?,
        )
        .context(format!("Failed to parse time: {}", window))?;

        time_windows.push((from, to));
    }
    Ok(time_windows)
}
