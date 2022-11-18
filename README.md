# smart-home

## Installation

### Prerequisties

- Raspberry Pi connected to network with static IP and the following installed:
  - SSH keys
  - Docker
  - Cloudflared (for hosting on the web using a tunnel, see [guide to running cloudflared as a service](https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/install-and-setup/tunnel-guide/local/))

### Setup

#### Environment variables
  - `.env.db`
```
POSTGRES_DB=<replace me>
POSTGRES_USER=<replace me>
POSTGRES_PASSWORD=<replace me>
```
- `raspi-rust-api/`: See [separate README](raspi-rust-api/README.md)
- `frontend/`: See [separate README](frontend/README.md)

### Running

- Optional: If SQL queries/tables have been edited:
`cd raspi-rust-api && cargo sqlx prepare`
- From another machine:
  `sh transfer.sh`
- On the Raspberry Pi (user root directory):
  `sh run.sh`

## Plug setup

### Shelly Plug S
1. Connect to plug WIFI and go to 192.168.33.1
2. Go to the `Internet & Security` tab.
3. Under `WIFI MODE - CLIENT`, connect the plug to your network and give it a static IP-address.
4. Add a username and password under `RESTRICT LOGIN`
5. Disable `CLOUD`
6. Add the plug in the smart-home UI

## Temp sensor setup

### Shelly H&T
1. Connect to sensor WIFI and go to 192.168.33.1
2. Go to the `Internet & Security` tab.
3. Under `WIFI MODE - CLIENT`, connect the plug to your network and give it a static IP-address.
4. Add a username and password under `RESTRICT LOGIN`
5. Disable `CLOUD`
6. Go to the `Actions` tab
7. Under `REPORT SENSOR VALUES`, enable and add the following URL:
   - `http://<ip of raspberry pi>:8081/report_ht/<room id>`
