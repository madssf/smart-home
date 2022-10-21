set -euxo pipefail

rm -rf "smart-home"
mkdir "smart-home"

cp docker-compose.yml smart-home

mkdir "smart-home/raspi-rust-api"
cp -R raspi-rust-api/src smart-home/raspi-rust-api/src
cp -R raspi-rust-api/configuration smart-home/raspi-rust-api/configuration
cp -R raspi-rust-api/db smart-home/raspi-rust-api/db
cp raspi-rust-api/.env.prod smart-home/raspi-rust-api
cp raspi-rust-api/sqlx-data.json smart-home/raspi-rust-api
cp raspi-rust-api/Cargo.lock smart-home/raspi-rust-api
cp raspi-rust-api/Cargo.toml smart-home/raspi-rust-api
cp raspi-rust-api/Dockerfile smart-home/raspi-rust-api
echo "Copied raspi-rust-api"

mkdir "smart-home/frontend"
cp -R frontend/app smart-home/frontend/app
cp -R frontend/styles smart-home/frontend/styles
cp frontend/.env smart-home/frontend
cp frontend/Dockerfile smart-home/frontend
cp frontend/package.json smart-home/frontend
cp frontend/package-lock.json smart-home/frontend
cp frontend/remix.config.js smart-home/frontend
cp frontend/remix.env.d.ts smart-home/frontend
cp frontend/tailwind.config.js smart-home/frontend
cp frontend/tsconfig.json smart-home/frontend
echo "Copied frontend"

cp .env.db smart-home/
echo "Copied database settings"

tar czf transfer.tar.gz smart-home

echo "Finished compressing"

rm -rf "smart-home"

scp transfer.tar.gz pi@raspberrypi.local:

echo "Finished transfer of application files"
scp run.sh pi@raspberrypi.local:

rm transfer.tar.gz

echo "Finished cleanup"
