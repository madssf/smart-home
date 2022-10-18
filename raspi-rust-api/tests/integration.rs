use testcontainers::clients::Cli;

use rust_home::db::plugs;

use crate::config::DatabaseTestConfig;

mod config;

#[tokio::test]
async fn can_insert_plug() {
    let docker = Cli::default();
    let test_config = DatabaseTestConfig::new(&docker).await;
    let client = plugs::Client::new(test_config.db_config);

    client
        .create_plug(
            "test_plug".to_string(),
            "127.0.0.1".to_string(),
            "username".to_string(),
            "password".to_string(),
        )
        .await
        .expect("Could not insert plug");

    let result = client.get_plugs().await.expect("Can't get plugs");

    let result_plug = result[0].clone();

    assert_eq!(result_plug.password, "password");
    assert_eq!(result_plug.name, "test_plug");
    assert_eq!(result_plug.ip, "127.0.0.1");
    assert_eq!(result_plug.password, "password");
}
