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
TIBBER_HOME_ID=<replace me>
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

### Simulating data in dev environment

#### Simulating Plug Data Using Dummy IPs

Dummy IP addresses (starting with 123.123) simulate plug status and power metrics within a development environment. 

IPs encode data: the last three digits reflect power in watts; the preceding single digit indicates status (1 for on, 0 for off, with off plugs generating 0 watts). 

For example, 123.123.0.123 represents an off plug with no power, while 123.123.1.030 signals a 30-watt, active plug. This facilitates scenario tests without real hardware.

#### Adding temperature log entries

Since we don't have temperature sensors in the dev environment, you can add temperature log entries manually using the following command:

Example:

```bash
curl -v -X POST http://localhost:8081/temperature_logs \
-H "Content-Type: application/json" \
-d '{"room_id":"a58e280a-d0c1-4c11-bd7a-7c5c01680783", "temp":22, "time":"2023-04-07T14:30:00"}'
```

