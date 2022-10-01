use std::str::FromStr;

use chrono::{Duration as CDuration, NaiveTime, Weekday};
use gcp_auth::{AuthenticationManager, Error as GCPAuthError};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::clients::FirestoreClient;
use crate::scheduling::ScheduleData;
use crate::{config_env_var, Plug, PriceLevel};

const SCHEDULES_COLLECTION_NAME: &str = "schedules";
const PLUGS_COLLECTION_NAME: &str = "plugs";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScheduleEntity {
    pub id: String,
    pub price_level: String,
    pub days: Vec<String>,
    pub hours: Vec<HoursEntity>,
}

impl ScheduleEntity {
    pub fn to_domain(&self) -> ScheduleData {
        ScheduleData {
            price_level: PriceLevel::from_str(&self.price_level)
                .expect("Failed to parse price level"),
            days: self
                .days
                .iter()
                .map(|day| Weekday::from_str(day).expect("Failed to parse weekday"))
                .collect::<Vec<Weekday>>(),
            windows: self
                .hours
                .iter()
                .map(|x| {
                    let start = NaiveTime::from_str(&format!("{}:00", x.from))
                        .expect("Failed to parse NaiveTime");
                    let stop = NaiveTime::from_str(&format!("{}:00", x.to))
                        .expect("Failed to parse NaiveTime");
                    let duration = stop.signed_duration_since(start);
                    (start, duration)
                })
                .collect::<Vec<(NaiveTime, CDuration)>>(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HoursEntity {
    pub from: String,
    pub to: String,
}
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Unexpected json format")]
    UnexpectedJsonFormat,
    #[error("AuthClientError: {0}")]
    AuthClientError(#[from] GCPAuthError),
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
}

pub async fn get_schedules(
    firestore_client: &FirestoreClient,
) -> Result<Vec<ScheduleData>, DbError> {
    match get_schedule_entities(firestore_client).await {
        Ok(entities) => Ok(entities
            .iter()
            .map(ScheduleEntity::to_domain)
            .collect::<Vec<ScheduleData>>()),
        Err(e) => Err(e),
    }
}

async fn get_schedule_entities(
    firestore_client: &FirestoreClient,
) -> Result<Vec<ScheduleEntity>, DbError> {
    let schedules: Result<Vec<ScheduleEntity>, DbError> =
        match get_documents(firestore_client, SCHEDULES_COLLECTION_NAME).await {
            Err(e) => return Err(e),
            Ok(documents) => documents.iter().map(parse_as_schedule).collect(),
        };
    schedules
}

pub async fn get_plugs(firestore_client: &FirestoreClient) -> Result<Vec<Plug>, DbError> {
    let plugs: Result<Vec<Plug>, DbError> =
        match get_documents(firestore_client, PLUGS_COLLECTION_NAME).await {
            Err(e) => return Err(e),
            Ok(documents) => documents.iter().map(parse_as_plug).collect(),
        };
    plugs
}

async fn get_documents(
    firestore_client: &FirestoreClient,
    collection_name: &str,
) -> Result<Vec<Value>, DbError> {
    let project_id = config_env_var("PROJECT_ID");
    let user_id = config_env_var("USER_ID");

    let scopes = &["https://www.googleapis.com/auth/datastore"];
    let token = firestore_client.auth_manager.get_token(scopes).await?;

    let url = format!(
        "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/users/{}/{}",
        project_id, user_id, collection_name
    );

    let res: Value = firestore_client
        .client
        .get(url)
        .header("Authorization", format!("Bearer {}", token.as_str()))
        .send()
        .await?
        .json()
        .await?;

    let documents = res.get("documents").expect("Failed to parse json");

    match documents.as_array() {
        None => Err(DbError::UnexpectedJsonFormat),
        Some(docs_array) => Ok(docs_array.clone()),
    }
}

fn parse_as_plug(value: &Value) -> Result<Plug, DbError> {
    let name = get_string_value(value, "/fields/name")?;
    let ip = get_string_value(value, "/fields/ip")?;
    let username = get_string_value(value, "/fields/username")?;
    let password = get_string_value(value, "/fields/password")?;
    Ok(Plug {
        name,
        ip,
        username,
        password,
    })
}

fn parse_as_schedule(value: &Value) -> Result<ScheduleEntity, DbError> {
    let days: Result<Vec<String>, DbError> = match value.pointer("/fields/days/arrayValue/values") {
        None => return Err(DbError::UnexpectedJsonFormat),
        Some(days) => match days.as_array() {
            None => return Err(DbError::UnexpectedJsonFormat),
            Some(days_obj) => days_obj
                .iter()
                .map(|day| get_string_value(day, ""))
                .collect(),
        },
    };

    let days = match days {
        Ok(days) => days,
        Err(e) => return Err(e),
    };

    let hours: Result<Vec<HoursEntity>, DbError> =
        match value.pointer("/fields/hours/arrayValue/values") {
            None => return Err(DbError::UnexpectedJsonFormat),
            Some(hours) => match hours.as_array() {
                None => return Err(DbError::UnexpectedJsonFormat),
                Some(hours_obj) => hours_obj
                    .iter()
                    .map(|hour| {
                        let from = get_string_value(hour, "/mapValue/fields/from")?;
                        let to = get_string_value(hour, "/mapValue/fields/to")?;
                        Ok(HoursEntity { from, to })
                    })
                    .collect(),
            },
        };

    let hours = match hours {
        Ok(hours) => hours,
        Err(e) => return Err(e),
    };

    let price_level = get_string_value(value, "/fields/priceLevel")?;

    let id = match value.pointer("/name") {
        None => return Err(DbError::UnexpectedJsonFormat),
        Some(id) => match id.as_str() {
            None => return Err(DbError::UnexpectedJsonFormat),
            Some(id) => match id.split('/').last() {
                None => return Err(DbError::UnexpectedJsonFormat),
                Some(id) => id.to_string(),
            },
        },
    };

    Ok(ScheduleEntity {
        id,
        price_level,
        days,
        hours,
    })
}

fn get_string_value(value: &Value, field_path: &str) -> Result<String, DbError> {
    match value.pointer(&*format!("{}/stringValue", field_path)) {
        None => Err(DbError::UnexpectedJsonFormat),
        Some(maybe_str) => match maybe_str.as_str() {
            None => {
                println!("here");
                Err(DbError::UnexpectedJsonFormat)
            }
            Some(str) => Ok(str.to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, NaiveTime, Weekday};

    use crate::db::{HoursEntity, ScheduleEntity};
    use crate::PriceLevel;

    use super::ScheduleData;

    #[test]
    fn should_parse_to_domain() {
        let entity = ScheduleEntity {
            id: "test-id".to_string(),
            price_level: String::from("CHEAP"),
            days: vec![String::from("MON"), String::from("TUE")],
            hours: vec![HoursEntity {
                from: String::from("13:00:00"),
                to: String::from("14:00:00"),
            }],
        };
        let expected = ScheduleData {
            price_level: PriceLevel::CHEAP,
            days: vec![Weekday::Mon, Weekday::Tue],
            windows: vec![(NaiveTime::from_hms(13, 0, 0), Duration::hours(1))],
        };
        assert_eq!(ScheduleEntity::to_domain(&entity), expected)
    }
}
