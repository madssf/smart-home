set -eux pipefail

cd smart-home
docker compose down
cd ..
tar -xvzf transfer.tar.gz
cd smart-home
docker compose up --build