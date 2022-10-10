use std::time::Duration;

use gcp_auth::{AuthenticationManager, CustomServiceAccount, Token};
use reqwest::Client;

use crate::config_env_var;

pub struct ShellyClient {
    pub client: Client,
}

pub struct FirestoreClient {
    pub client: Client,
    pub auth_manager: AuthenticationManager,
}

impl FirestoreClient {
    pub async fn get_token(&self) -> Result<Token, gcp_auth::Error> {
        let scopes = &["https://www.googleapis.com/auth/datastore"];
        self.auth_manager.get_token(scopes).await
    }
}

pub fn get_clients() -> (ShellyClient, FirestoreClient) {
    let base64_key = config_env_var("FB_SA_KEY");
    let decoded_vec = base64::decode(base64_key).expect("Failed to decode FB_SA_KEY");

    let key = match String::from_utf8(decoded_vec) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    let custom_service_account =
        CustomServiceAccount::from_json(&key).expect("Failed to created custom service account");
    let auth_manager = AuthenticationManager::from(custom_service_account);

    let firestore_http_client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create client");

    let shelly_http_client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create client");

    (
        ShellyClient {
            client: shelly_http_client,
        },
        FirestoreClient {
            client: firestore_http_client,
            auth_manager,
        },
    )
}
