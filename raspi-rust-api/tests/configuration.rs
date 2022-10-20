use testcontainers::clients::Cli;
use testcontainers::core::WaitFor;
use testcontainers::images::generic::GenericImage;
use testcontainers::Container;

use rust_home::db;

#[allow(dead_code)]
pub struct DatabaseTestConfig<'a> {
    pub postgres_container: Container<'a, GenericImage>, // Needed to keep container running
    pub db_config: db::DbConfig,
}

impl<'a> DatabaseTestConfig<'a> {
    pub async fn new(cli: &'a Cli) -> DatabaseTestConfig {
        let db = "smarthome";
        let user = "user";
        let password = "password";

        let image = GenericImage::new("docker.io/postgres", "14-alpine")
            .with_wait_for(WaitFor::message_on_stderr(
                "database system is ready to accept connections",
            ))
            .with_env_var("POSTGRES_DB", db)
            .with_env_var("POSTGRES_USER", user)
            .with_env_var("POSTGRES_PASSWORD", password);

        let postgres_container = cli.run(image);
        let connection_string = format!(
            "postgres://{}:{}@127.0.0.1:{}/{}",
            user,
            password,
            postgres_container.get_host_port_ipv4(5432),
            db
        );

        let db_config = db::DbConfig::new(&connection_string)
            .await
            .expect("Could not connect to database!");

        Self {
            postgres_container,
            db_config,
        }
    }
}
