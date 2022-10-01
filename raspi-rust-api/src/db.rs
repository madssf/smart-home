use std::str::FromStr;
use std::time::Duration;

use chrono::{Duration as CDuration, NaiveTime, Weekday};
use gcp_auth::{AuthenticationManager, Error as GCPAuthError};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::scheduling::ScheduleData;
use crate::PriceLevel;

const SCHEDULES_COLLECTION_NAME: &'static str = "schedules";
const PLUGS_COLLECTION_NAME: &'static str = "plugs";

pub fn config_env_var(name: &str) -> String {
    std::env::var(name)
        .map_err(|e| format!("{}: {}", name, e))
        .expect(&*format!("Missing config env var: {}", name))
}

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

pub async fn get_schedules(client: &Client) -> Result<Vec<ScheduleData>, DbError> {
    match get_schedule_entities(client).await {
        Ok(entities) => Ok(entities
            .iter()
            .map(ScheduleEntity::to_domain)
            .collect::<Vec<ScheduleData>>()),
        Err(e) => Err(e),
    }
}

async fn get_schedule_entities(client: &Client) -> Result<Vec<ScheduleEntity>, DbError> {
    let project_id = config_env_var("PROJECT_ID");
    let user_id = config_env_var("USER_ID");

    // Create an instance
    let auth_manager = AuthenticationManager::new().await?;

    let scopes = &["https://www.googleapis.com/auth/datastore"];
    let token = auth_manager.get_token(scopes).await?;

    let url = format!("https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/users/{}/schedules", project_id, user_id);

    let res: Value = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token.as_str()))
        .send()
        .await?
        .json()
        .await?;

    let documents = res.get("documents").expect("Failed to parse json");

    let schedules: Result<Vec<ScheduleEntity>, DbError> = match documents.as_array() {
        None => return Err(DbError::UnexpectedJsonFormat),
        Some(documents) => documents.iter().map(parse_as_schedule).collect(),
    };

    schedules
}

fn parse_as_schedule(value: &Value) -> Result<ScheduleEntity, DbError> {
    let days: Result<Vec<String>, DbError> = match value.pointer("/fields/days/arrayValue/values") {
        None => return Err(DbError::UnexpectedJsonFormat),
        Some(days) => match days.as_array() {
            None => return Err(DbError::UnexpectedJsonFormat),
            Some(days_obj) => days_obj
                .iter()
                .map(|day| match day.pointer("/stringValue") {
                    None => Err(DbError::UnexpectedJsonFormat),
                    Some(day) => match day.as_str() {
                        None => Err(DbError::UnexpectedJsonFormat),
                        Some(day) => Ok(day.to_string()),
                    },
                })
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
                        let from = match hour.pointer("/mapValue/fields/from/stringValue") {
                            None => Err(DbError::UnexpectedJsonFormat),
                            Some(from) => match from.as_str() {
                                None => Err(DbError::UnexpectedJsonFormat),
                                Some(from) => Ok(from.to_string()),
                            },
                        };

                        let from = match from {
                            Ok(from) => from,
                            Err(_) => return Err(DbError::UnexpectedJsonFormat),
                        };

                        let to = match hour.pointer("/mapValue/fields/to/stringValue") {
                            None => Err(DbError::UnexpectedJsonFormat),
                            Some(from) => match from.as_str() {
                                None => Err(DbError::UnexpectedJsonFormat),
                                Some(from) => Ok(from.to_string()),
                            },
                        };

                        let to = match to {
                            Ok(to) => to,
                            Err(_) => return Err(DbError::UnexpectedJsonFormat),
                        };

                        Ok(HoursEntity { from, to })
                    })
                    .collect(),
            },
        };

    let hours = match hours {
        Ok(hours) => hours,
        Err(e) => return Err(e),
    };

    let price_level = value.pointer("/fields/priceLevel/stringValue");

    let price_level = match price_level {
        Some(price_level) => match price_level.as_str() {
            None => return Err(DbError::UnexpectedJsonFormat),
            Some(price_level) => price_level.to_string(),
        },
        None => return Err(DbError::UnexpectedJsonFormat),
    };

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
