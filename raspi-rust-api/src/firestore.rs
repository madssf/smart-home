use chrono::{DateTime, Utc};
use firestore::*;
use serde::{Deserialize, Serialize};

use crate::prices::PriceLevel;

pub fn config_env_var(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|e| format!("{}: {}", name, e))
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ScheduleEntity {
    id: String,
    level: String,
    days: Vec<String>,
    hours: Vec<HoursEntity>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct HoursEntity {
    from: String,
    to: String,
}

pub async fn get() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    // Create an instance
    let db = FirestoreDb::new(&config_env_var("PROJECT_ID")?).await?;

    const TEST_COLLECTION_NAME: &'static str = "schedules";

    let test_schedule = ScheduleEntity {
        id: "test-id".to_string(),
        level: PriceLevel::CHEAP.to_string(),
        days: vec!["MON".to_string()],
        hours: vec![HoursEntity {from: "17:00:00".to_string(), to: "20:00:00".to_string()}],
    };

    // Remove if it already exist
    db.delete_by_id(TEST_COLLECTION_NAME, &test_schedule.id)
        .await?;

    // Let's insert some data
    db.create_obj(TEST_COLLECTION_NAME, &test_schedule.id, &test_schedule)
        .await?;

    // Update some field in it
    let updated_obj = db
        .update_obj(
            TEST_COLLECTION_NAME,
            &test_schedule.id,
            &ScheduleEntity {
                days: [test_schedule.days.clone(), vec!["TUE".to_string()]].concat(),
                ..test_schedule.clone()
            },
            Some(vec![path!(ScheduleEntity::days)]),
        )
        .await?;

    println!("Updated object: {:?}", updated_obj);

    // Get object by id
    let find_it_again: ScheduleEntity =
        db.get_obj(TEST_COLLECTION_NAME, &test_schedule.id).await?;

    println!("Should be the same: {:?}", find_it_again);

    // Query our data
    let objects: Vec<ScheduleEntity> = db
        .query_obj(
            FirestoreQueryParams::new(TEST_COLLECTION_NAME.into()).with_filter(
                FirestoreQueryFilter::Compare(Some(FirestoreQueryFilterCompare::Equal(
                    path!(ScheduleEntity::level),
                    find_it_again.level.into(),
                ))),
            ),
        )
        .await?;

    println!("Now in the list: {:?}", objects);

    Ok(())
}