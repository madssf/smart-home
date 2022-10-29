set -eux pipefail

cd smart-home
docker compose down
cd ..
rm -rf smart-home
tar -xvzf transfer.tar.gz
cd smart-home
docker compose up --build
