use chrono::NaiveDateTime;
use gcp_auth::Error as GCPAuthError;
use serde_json::{json, Value};
use thiserror::Error;

use crate::clients::FirestoreClient;
use crate::env_var;

const TEMPERATURE_LOG_COLLECTION_NAME: &str = "temperature_log";

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
