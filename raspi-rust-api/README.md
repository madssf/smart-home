# raspi-rust-api

## Setup

### Environment variables

`.env/.envrc`
```
TIBBER_API_TOKEN=<replace_me>
TIME_ZONE=<replace_me>
PROJECT_ID=<gcp_project_id>
USER_ID=<firebase_user_id>
FB_SA_KEY=<base64 encoded firebase service account key json>
RUST_LOG=<rust log level>

```

### SQLX Offline Mode

```bash
cargo sqlx prepare
```
