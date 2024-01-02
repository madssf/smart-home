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

## Cross-compiling for Raspberry Pi

### 1. Build the Images for ARM Architecture

On your Mac:

- Use Docker Buildx to create a new builder that has the capability to build ARM images:

```sh
docker buildx create --name arm_builder --use
```

- Build and store the images locally:

Replace `linux/arm/v7` with `linux/arm64` if your Raspberry Pi is running a 64-bit OS.

```sh
docker buildx build --platform linux/arm/v7 -t raspi-rust-api-armv7:latest --load -f raspi-rust-api/Dockerfile.armv7 raspi-rust-api/.
```

```sh
docker buildx build --platform linux/arm/v7 -t frontend-armv7:latest --load -f frontend/Dockerfile frontend/.
```

### 2. Transfer the Images

- To transfer images manually without a registry, save each Docker image to a tar file:

```sh
docker save raspi-rust-api-armv7:latest -o raspi-rust-api-armv7.tar
docker save frontend-armv7:latest -o frontend-armv7.tar
```

- Transfer the tar files to your Raspberry Pi using `scp`:

Replace `pi@raspberrypi.local` with the appropriate username and hostname or IP address for your Raspberry Pi.

```sh
scp frontend-armv7.tar pi@raspberrypi.local:smart-home
scp raspi-rust-api-armv7.tar pi@raspberrypi.local:smart-home
scp docker-compose-armv7.yml pi@raspberrypi.local:smart-home
```

- Transfer the .env files to your Raspberry Pi using `scp`:

```sh
scp .env.db pi@raspberrypi.local:smart-home
scp raspi-rust-api/.env.prod pi@raspberrypi.local:smart-home
scp frontend/.env pi@raspberrypi.local:smart-home
```



### 3. Run the Containers

On your Raspberry Pi:

- Load the images:

```sh
docker load -i smart-home-2/frontend-armv7.tar
docker load -i smart-home-2/raspi-rust-api-armv7.tar
```

- Use `docker-compose` to run the containers:

```sh
cd smart-home-2
docker compose -f docker-compose-armv7.yml up --build
```


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

### Aqara Temperature Sensor
