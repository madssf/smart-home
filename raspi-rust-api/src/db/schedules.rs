use std::str::FromStr;

use bigdecimal::{FromPrimitive, ToPrimitive};
use chrono::{Datelike, NaiveDateTime, NaiveTime, Weekday};
use sqlx::types::BigDecimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::DbError;
use crate::domain::{PriceLevel, Schedule};

#[derive(Copy, Clone, Debug)]
struct RoomScheduleEntity {
    room_id: Uuid,
    schedule_id: Uuid,
}

struct ScheduleEntityWrapper {
    schedule: ScheduleEntity,
    time_windows: Vec<ScheduleTimeWindowEntity>,
    temps: Vec<ScheduleTempEntity>,
    room_ids: Vec<RoomScheduleEntity>,
}

struct ScheduleEntity {
    id: Uuid,
    days: Vec<String>,
}

impl ScheduleEntity {
    fn to_domain(
        &self,
        room_schedules: &[RoomScheduleEntity],
        time_windows: &[ScheduleTimeWindowEntity],
        temps: &[ScheduleTempEntity],
    ) -> Schedule {
        Schedule {
            id: self.id,
            temps: temps
                .iter()
                .filter(|temp| temp.schedule_id == self.id)
                .map(|entity| {
                    (
                        PriceLevel::from_str(&entity.price_level).unwrap_or_else(|_| {
                            panic!("Can't convert string to PriceLevel: {}", entity.price_level)
                        }),
                        entity.temp.to_f64().unwrap_or_else(|| {
                            panic!("Can't convert Decimal to f64: {}", entity.temp)
                        }),
                    )
                })
                .collect(),
            days: self
                .days
                .iter()
                .map(|day| {
                    Weekday::from_str(day)
                        .unwrap_or_else(|_| panic!("Can't convert string to Weekday: {}", day))
                })
                .collect(),
            time_windows: time_windows
                .iter()
                .filter(|window| window.schedule_id == self.id)
                .map(|entity| (entity.from_time, entity.to_time))
                .collect(),
            room_ids: room_schedules
                .iter()
                .filter(|room_schedule| room_schedule.schedule_id == self.id)
                .map(|room_schedule| room_schedule.room_id)
                .collect(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct ScheduleTimeWindowEntity {
    schedule_id: Uuid,
    from_time: NaiveTime,
    to_time: NaiveTime,
}

#[derive(Debug, Clone)]
struct ScheduleTempEntity {
    schedule_id: Uuid,
    price_level: String,
    temp: BigDecimal,
}

fn to_entity(schedule: &Schedule) -> Result<ScheduleEntityWrapper, anyhow::Error> {
    Ok(ScheduleEntityWrapper {
        schedule: ScheduleEntity {
            id: schedule.id,
            days: schedule
                .days
                .iter()
                .map(|weekday| weekday.to_string())
                .collect(),
        },
        time_windows: schedule
            .time_windows
            .iter()
            .map(|window| ScheduleTimeWindowEntity {
                schedule_id: schedule.id,
                from_time: window.0,
                to_time: window.1,
            })
            .collect(),
        temps: schedule
            .temps
            .iter()
            .map(|(price_level, temp)| ScheduleTempEntity {
                schedule_id: schedule.id,
                price_level: price_level.to_string(),
                temp: BigDecimal::from_f64(temp.to_owned())
                    .unwrap_or_else(|| panic!("Couldn't convert {} to BigDecimal", temp)),
            })
            .collect(),
        room_ids: schedule
            .room_ids
            .iter()
            .map(|room_id| RoomScheduleEntity {
                schedule_id: schedule.id,
                room_id: *room_id,
            })
            .collect(),
    })
}
pub async fn get_schedules(pool: &PgPool) -> Result<Vec<Schedule>, DbError> {
    let entities: Vec<ScheduleEntity> = sqlx::query_as!(ScheduleEntity, "SELECT * FROM schedules")
        .fetch_all(pool)
        .await?;

    let schedule_temps = sqlx::query_as!(ScheduleTempEntity, "SELECT * FROM schedule_temps")
        .fetch_all(pool)
        .await?;

    let schedule_time_windows = sqlx::query_as!(
        ScheduleTimeWindowEntity,
        "SELECT * FROM schedule_time_windows"
    )
    .fetch_all(pool)
    .await?;

    let room_schedules: Vec<RoomScheduleEntity> =
        sqlx::query_as!(RoomScheduleEntity, "SELECT * FROM room_schedules")
            .fetch_all(pool)
            .await?;
    Ok(entities
        .iter()
        .map(|entity| entity.to_domain(&room_schedules, &schedule_time_windows, &schedule_temps))
        .collect())
}

pub async fn get_room_schedules(pool: &PgPool, room_id: &Uuid) -> Result<Vec<Schedule>, DbError> {
    let room_schedules: Vec<RoomScheduleEntity> = sqlx::query_as!(
        RoomScheduleEntity,
        "SELECT * FROM room_schedules WHERE room_id = $1",
        room_id
    )
    .fetch_all(pool)
    .await?;

    let sched_ids: Vec<Uuid> = room_schedules.iter().map(|r| r.schedule_id).collect();

    let entities: Vec<ScheduleEntity> = sqlx::query_as!(
        ScheduleEntity,
        "SELECT * FROM schedules WHERE id = any($1)",
        &sched_ids
    )
    .fetch_all(pool)
    .await?;

    let schedule_temps = sqlx::query_as!(
        ScheduleTempEntity,
        "SELECT * FROM schedule_temps WHERE schedule_id = any($1)",
        &sched_ids
    )
    .fetch_all(pool)
    .await?;

    let schedule_time_windows = sqlx::query_as!(
        ScheduleTimeWindowEntity,
        "SELECT * FROM schedule_time_windows WHERE schedule_id = any($1)",
        &sched_ids
    )
    .fetch_all(pool)
    .await?;

    Ok(entities
        .iter()
        .map(|entity| entity.to_domain(&room_schedules, &schedule_time_windows, &schedule_temps))
        .collect())
}

pub async fn get_matching_schedule(
    pool: &PgPool,
    room_id: &Uuid,
    time: &NaiveDateTime,
) -> Result<Option<Schedule>, DbError> {
    let room_schedules: Vec<RoomScheduleEntity> = sqlx::query_as!(
        RoomScheduleEntity,
        "SELECT * FROM room_schedules WHERE room_id = $1",
        room_id
    )
    .fetch_all(pool)
    .await?;

    let room_sched_ids: Vec<Uuid> = room_schedules.iter().map(|r| r.schedule_id).collect();

    let schedule_time_windows: Vec<ScheduleTimeWindowEntity> = sqlx::query_as!(
        ScheduleTimeWindowEntity,
        "SELECT * FROM schedule_time_windows WHERE schedule_id = any($1) AND from_time < $2 AND to_time > $2",
        &room_sched_ids,
        time.time(),
    ).fetch_all(pool)
        .await?;

    let sched_ids: Vec<Uuid> = schedule_time_windows
        .iter()
        .map(|r| r.schedule_id)
        .collect();

    let entity: Option<ScheduleEntity> = sqlx::query_as!(
        ScheduleEntity,
        "SELECT * FROM schedules WHERE id = any($1) AND $2 = any(days)",
        &sched_ids,
        time.weekday().to_string(),
    )
    .fetch_optional(pool)
    .await?;

    Ok(match entity {
        None => None,
        Some(entity) => {
            let schedule_temps = sqlx::query_as!(
                ScheduleTempEntity,
                "SELECT * FROM schedule_temps WHERE schedule_id = $1",
                &entity.id
            )
            .fetch_all(pool)
            .await?;
            Some(entity.to_domain(&room_schedules, &schedule_time_windows, &schedule_temps))
        }
    })
}

pub async fn create_schedule(pool: &PgPool, new_schedule: Schedule) -> Result<(), DbError> {
    let wrapper = to_entity(&new_schedule)?;

    let mut tx = pool.begin().await?;
    sqlx::query!(
        r#"
    INSERT INTO schedules (id, days)
    VALUES ($1, $2)
    "#,
        wrapper.schedule.id,
        &wrapper.schedule.days,
    )
    .execute(&mut tx)
    .await?;

    for time_window in wrapper.time_windows {
        sqlx::query!(
            r#"
        INSERT INTO schedule_time_windows (schedule_id, from_time, to_time)
        VALUES ($1, $2, $3)
        "#,
            time_window.schedule_id,
            time_window.from_time,
            time_window.to_time,
        )
        .execute(&mut tx)
        .await?;
    }

    for temp in wrapper.temps {
        sqlx::query!(
            r#"
        INSERT INTO schedule_temps (schedule_id, price_level, temp)
        VALUES ($1, $2, $3)
        "#,
            temp.schedule_id,
            temp.price_level,
            temp.temp,
        )
        .execute(&mut tx)
        .await?;
    }

    for room_schedule in wrapper.room_ids {
        sqlx::query!(
            r#"
        INSERT INTO room_schedules (room_id, schedule_id)
        VALUES ($1, $2)
        "#,
            room_schedule.room_id,
            room_schedule.schedule_id,
        )
        .execute(&mut tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

pub async fn update_schedule(pool: &PgPool, schedule: Schedule) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;

    let wrapper = to_entity(&schedule)?;

    sqlx::query!(
        r#"
        UPDATE schedules
        SET days = $2
        WHERE id = $1
        "#,
        wrapper.schedule.id,
        &wrapper.schedule.days,
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
                wrapper.schedule.id,
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

    let existing_time_windows: Vec<ScheduleTimeWindowEntity> = sqlx::query_as!(
        ScheduleTimeWindowEntity,
        "SELECT * FROM schedule_time_windows WHERE schedule_id = $1",
        schedule.id
    )
    .fetch_all(&mut tx)
    .await?;

    for existing in existing_time_windows.clone() {
        if !schedule
            .time_windows
            .iter()
            .any(|x| x.0 == existing.from_time && x.1 == existing.to_time)
        {
            sqlx::query!(
                r#"
                DELETE FROM schedule_time_windows WHERE schedule_id = $1 AND from_time = $2 AND to_time = $3
                "#,
                schedule.id,
                existing.from_time,
                existing.to_time,
            )
                .execute(&mut tx)
                .await?;
        }
    }

    for time_window in &schedule.time_windows {
        if !existing_time_windows
            .iter()
            .any(|x| x.from_time == time_window.0 && x.to_time == time_window.1)
        {
            sqlx::query!(
                r#"
            INSERT INTO schedule_time_windows (schedule_id, from_time, to_time)
            VALUES ($1, $2, $3)
            "#,
                schedule.id,
                time_window.0,
                time_window.1,
            )
            .execute(&mut tx)
            .await?;
        }
    }

    let existing_temps: Vec<ScheduleTempEntity> = sqlx::query_as!(
        ScheduleTempEntity,
        "SELECT * FROM schedule_temps WHERE schedule_id = $1",
        schedule.id
    )
    .fetch_all(&mut tx)
    .await?;

    for existing in existing_temps.clone() {
        if !schedule.temps.iter().any(|x| {
            x.0.to_string() == existing.price_level && x.1 == &existing.temp.to_f64().unwrap()
        }) {
            sqlx::query!(
                r#"
                DELETE FROM schedule_temps WHERE schedule_id = $1 AND price_level = $2 AND temp = $3
                "#,
                schedule.id,
                existing.price_level,
                existing.temp,
            )
            .execute(&mut tx)
            .await?;
        }
    }

    for temp in &schedule.temps {
        if !existing_temps
            .iter()
            .any(|x| x.price_level == temp.0.to_string() && &x.temp.to_f64().unwrap() == temp.1)
        {
            sqlx::query!(
                r#"
            INSERT INTO schedule_temps (schedule_id, price_level, temp)
            VALUES ($1, $2, $3)
            "#,
                schedule.id,
                temp.0.to_string(),
                BigDecimal::from_f64(temp.1.clone()).unwrap(),
            )
            .execute(&mut tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}

pub async fn delete_schedule(pool: &PgPool, id: &Uuid) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;

    sqlx::query!("DELETE FROM room_schedules WHERE schedule_id = $1", id)
        .execute(&mut tx)
        .await?;

    sqlx::query!(
        "DELETE FROM schedule_time_windows WHERE schedule_id = $1",
        id
    )
    .execute(&mut tx)
    .await?;

    sqlx::query!("DELETE FROM schedule_temps WHERE schedule_id = $1", id)
        .execute(&mut tx)
        .await?;

    sqlx::query!("DELETE FROM schedules WHERE id = $1", id)
        .execute(&mut tx)
        .await?;

    tx.commit().await?;

    Ok(())
}
