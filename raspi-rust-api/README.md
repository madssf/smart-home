# raspi-rust-api

## Setup

### Environment variables

`.env` 

For SQLX during development.
If you change init_dev_db.sh you need to change this too.
```
DATABASE_URL=postgres://postgres:password@localhost:5432/smarthome
```

`.env.prod`
```
TIBBER_API_TOKEN=<replace me>
TIME_ZONE=<replace me>
RUST_LOG=<set this or don't!>
ENVIRONMENT=production
APP_DATABASE__USERNAME=<prod db username>
APP_DATABASE__PASSWORD=<prod db password>
APP_DATABASE__DATABASE_NAME=<prod db name>


```

### SQLX Offline Mode

```bash
cargo sqlx prepare
```
