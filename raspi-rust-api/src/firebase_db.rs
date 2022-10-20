use std::str::FromStr;

use chrono::{Duration as CDuration, NaiveDateTime, NaiveTime, Weekday};
use gcp_auth::Error as GCPAuthError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

use crate::clients::FirestoreClient;
use crate::scheduling::ScheduleData;
use crate::{env_var, ActionType, PriceLevel, TempAction};

const SCHEDULES_COLLECTION_NAME: &str = "schedules";
const TEMP_ACTIONS_COLLECTION__NAME: &str = "temp_actions";
const TEMPERATURE_LOG_COLLECTION_NAME: &str = "temperature_log";

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
pub enum FirebaseDbError {
    #[error("Unexpected json format")]
    UnexpectedJsonFormat,
    #[error("AuthClientError: {0}")]
    AuthClientError(#[from] GCPAuthError),
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Parse error: {0}")]
    EnumParseError(#[from] strum::ParseError),
    #[error("Parse error: {0}")]
    TimeParseError(#[from] chrono::ParseError),
    #[error("Json parse error: {0}")]
    JsonParseError(#[from] serde_json::Error),
}

pub async fn get_schedules(
    firestore_client: &FirestoreClient,
) -> Result<Vec<ScheduleData>, FirebaseDbError> {
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
) -> Result<Vec<ScheduleEntity>, FirebaseDbError> {
    let schedules: Result<Vec<ScheduleEntity>, FirebaseDbError> =
        match get_documents(firestore_client, SCHEDULES_COLLECTION_NAME).await {
            Err(e) => return Err(e),
            Ok(documents) => documents.iter().map(parse_as_schedule).collect(),
        };
    schedules
}

pub async fn get_temp_actions(
    firestore_client: &FirestoreClient,
) -> Result<Vec<TempAction>, FirebaseDbError> {
    match get_documents(firestore_client, TEMP_ACTIONS_COLLECTION__NAME).await {
        Ok(documents) => documents.iter().map(parse_as_temp_action).collect(),
        Err(e) => Err(e),
    }
}

pub async fn delete_temp_action(
    firestore_client: &FirestoreClient,
    plug_id: &str,
) -> Result<(), FirebaseDbError> {
    delete_document(firestore_client, TEMP_ACTIONS_COLLECTION__NAME, plug_id).await
}

pub async fn insert_temperature_log(
    firestore_client: &FirestoreClient,
    time: NaiveDateTime,
    room: &str,
    temperature: &f64,
) -> Result<(), FirebaseDbError> {
    let doc_json = json!(
            {
      "fields": {
                "temp": {
                    "stringValue": temperature.to_string()
                },
                "room": {
                    "stringValue": room.to_string()
                },
                "time": {
                    "stringValue": time.to_string()
                },
      },
    });
    create_document(firestore_client, TEMPERATURE_LOG_COLLECTION_NAME, &doc_json).await
}

fn get_base_url(collection_name: &str) -> String {
    let project_id = env_var("PROJECT_ID");
    let user_id = env_var("USER_ID");

    format!(
        "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/users/{}/{}",
        project_id, user_id, collection_name
    )
}

async fn get_documents(
    firestore_client: &FirestoreClient,
    collection_name: &str,
) -> Result<Vec<Value>, FirebaseDbError> {
    let res: Value = firestore_client
        .client
        .get(get_base_url(collection_name))
        .header(
            "Authorization",
            format!("Bearer {}", firestore_client.get_token().await?.as_str()),
        )
        .send()
        .await?
        .json()
        .await?;

    let documents = match res.get("documents") {
        None => return Ok(vec![]),
        Some(docs) => docs,
    };

    match documents.as_array() {
        None => Err(FirebaseDbError::UnexpectedJsonFormat),
        Some(docs_array) => Ok(docs_array.clone()),
    }
}

async fn delete_document(
    firestore_client: &FirestoreClient,
    collection_name: &str,
    document_id: &str,
) -> Result<(), FirebaseDbError> {
    let url = format!("{}/{}", get_base_url(collection_name), document_id);
    firestore_client
        .client
        .delete(url)
        .header(
            "Authorization",
            format!("Bearer {}", firestore_client.get_token().await?.as_str()),
        )
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

async fn create_document(
    firestore_client: &FirestoreClient,
    collection_name: &str,
    document_json: &Value,
) -> Result<(), FirebaseDbError> {
    let url = get_base_url(collection_name);

    let _ = firestore_client
        .client
        .post(url)
        .header(
            "Authorization",
            format!("Bearer {}", firestore_client.get_token().await?.as_str()),
        )
        .json(document_json)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

fn parse_as_temp_action(doc_json: &Value) -> Result<TempAction, FirebaseDbError> {
    let id = get_document_id(doc_json)?;
    let plug_ids = get_string_vec(doc_json, "fields/plug_ids")?;
    let action_type = ActionType::from_str(&get_string_value(doc_json, "/fields/action_type")?)?;
    let expires_at = NaiveDateTime::from_str(&get_string_value(doc_json, "/fields/expires_at")?)?;
    Ok(TempAction {
        id,
        plug_ids,
        action_type,
        expires_at,
    })
}

fn parse_as_schedule(doc_json: &Value) -> Result<ScheduleEntity, FirebaseDbError> {
    let days = get_string_vec(doc_json, "fields/days")?;
    let hours: Result<Vec<HoursEntity>, FirebaseDbError> =
        match doc_json.pointer("/fields/hours/arrayValue/values") {
            None => return Err(FirebaseDbError::UnexpectedJsonFormat),
            Some(hours) => match hours.as_array() {
                None => return Err(FirebaseDbError::UnexpectedJsonFormat),
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

    let price_level = get_string_value(doc_json, "/fields/priceLevel")?;

    let id = get_document_id(doc_json)?;

    Ok(ScheduleEntity {
        id,
        price_level,
        days,
        hours,
    })
}

fn get_string_value(value: &Value, field_path: &str) -> Result<String, FirebaseDbError> {
    match value.pointer(&*format!("{}/stringValue", field_path)) {
        None => Err(FirebaseDbError::UnexpectedJsonFormat),
        Some(maybe_str) => match maybe_str.as_str() {
            None => Err(FirebaseDbError::UnexpectedJsonFormat),
            Some(str) => Ok(str.to_string()),
        },
    }
}

fn get_string_vec(json: &Value, field_path: &str) -> Result<Vec<String>, FirebaseDbError> {
    match json.pointer(&format!("/{}/arrayValue/values", field_path)) {
        None => Err(FirebaseDbError::UnexpectedJsonFormat),
        Some(value) => match value.as_array() {
            None => Err(FirebaseDbError::UnexpectedJsonFormat),
            Some(json_vec) => json_vec
                .iter()
                .map(|value| get_string_value(value, ""))
                .collect(),
        },
    }
}

fn get_document_id(value: &Value) -> Result<String, FirebaseDbError> {
    match value.pointer("/name") {
        None => Err(FirebaseDbError::UnexpectedJsonFormat),
        Some(id) => match id.as_str() {
            None => Err(FirebaseDbError::UnexpectedJsonFormat),
            Some(id) => match id.split('/').last() {
                None => Err(FirebaseDbError::UnexpectedJsonFormat),
                Some(id) => Ok(id.to_string()),
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, NaiveTime, Weekday};

    use crate::firebase_db::{HoursEntity, ScheduleEntity};
    use crate::PriceLevel;

    use super::ScheduleData;

    #[test]
    fn should_parse_to_domain() {
        let entity = ScheduleEntity {
            id: "test-id".to_string(),
            price_level: String::from("CHEAP"),
            days: vec![String::from("MON"), String::from("TUE")],
            hours: vec![HoursEntity {
                from: String::from("13:00"),
                to: String::from("14:00"),
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
