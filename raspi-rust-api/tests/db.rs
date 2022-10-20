use testcontainers::clients::Cli;

use configuration::DatabaseTestConfig;
use rust_home::db::plugs;

mod configuration;

#[tokio::test]
async fn can_insert_plug() {
    let docker = Cli::default();
    let test_config = DatabaseTestConfig::new(&docker).await;
    let plugs_client = plugs::PlugsClient::new(test_config.db_config);

    plugs_client
        .create_plug("test_plug", "127.0.0.1", "username", "password")
        .await
        .expect("Could not insert plug");

    let result = plugs_client.get_plugs().await.expect("Can't get plugs");

    let result_plug = result[0].clone();

    assert_eq!(result_plug.password, "password");
    assert_eq!(result_plug.name, "test_plug");
    assert_eq!(result_plug.ip, "127.0.0.1");
    assert_eq!(result_plug.username, "username");
}

#[tokio::test]
async fn can_update_plug() {
    let docker = Cli::default();
    let test_config = DatabaseTestConfig::new(&docker).await;
    let plugs_client = plugs::PlugsClient::new(test_config.db_config);

    plugs_client
        .create_plug("test_plug", "127.0.0.1", "username", "password")
        .await
        .expect("Could not insert plug");

    let stored = plugs_client.get_plugs().await.expect("Can't get plugs");
    let stored_plug = stored[0].clone();

    plugs_client
        .update_plug(
            &stored_plug.id,
            "new_name",
            "127.0.0.2",
            "new_uname",
            "new_pass",
        )
        .await
        .expect("Can't update plug");

    let result = plugs_client.get_plugs().await.expect("Can't get plugs");
    let result_plug = result[0].clone();

    assert_eq!(result_plug.password, "new_pass");
    assert_eq!(result_plug.name, "new_name");
    assert_eq!(result_plug.ip, "127.0.0.2");
    assert_eq!(result_plug.username, "new_uname");
}

#[tokio::test]
async fn can_delete_plug() {
    let docker = Cli::default();
    let test_config = DatabaseTestConfig::new(&docker).await;
    let plugs_client = plugs::PlugsClient::new(test_config.db_config);

    plugs_client
        .create_plug("test_plug", "127.0.0.1", "username", "password")
        .await
        .expect("Could not insert plug");

    let stored = plugs_client.get_plugs().await.expect("Can't get plugs");

    let id = stored[0].clone().id;

    plugs_client
        .delete_plug(&id)
        .await
        .expect("Failed to delete plug");

    let result = plugs_client.get_plugs().await.expect("Can't get plugs");
    assert_eq!(result.len(), 0)
}
