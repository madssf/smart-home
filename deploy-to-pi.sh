#!/bin/bash

# Default Configuration variables
PI_USERNAME=pi
PI_HOSTNAME=raspberrypi.local
PI_DIRECTORY=smart-home
ARM_VERSION=linux/arm/v7
API_IMAGE_NAME=raspi-rust-api-armv7
FRONTEND_IMAGE_NAME=frontend-armv7
COMPOSE_FILE=docker-compose-armv7.yml

# Flags to control build and transfer steps
SKIP_API=false
SKIP_FRONTEND=false
ATTACH_LOGS=true

# Help menu
function display_help() {
    echo "Usage: $0 [options]"
    echo "   -h, --help                 Display help"
    echo "   --skip-api                 Skip building and transferring the API image"
    echo "   --skip-frontend            Skip building and transferring the frontend image"
    echo "   --no-attach                Do not attach to container logs after starting"
    exit 0
}

# Function to gracefully handle failures
handle_error() {
    echo "Error: $1"
    echo "Deployment failed. Cleaning up..."
    ssh $PI_USERNAME@$PI_HOSTNAME "cd $PI_DIRECTORY && docker compose -f $COMPOSE_FILE down"
    exit 1
}

# Argument parsing
while [[ "$#" -gt 0 ]]; do
    case "$1" in
        -h|--help) display_help;;
        --skip-api) SKIP_API=true;;
        --skip-frontend) SKIP_FRONTEND=true;;
        --no-attach) ATTACH_LOGS=false;;
        *) echo "Error: Invalid option."; display_help;;
    esac
    shift
done

# Create Docker builder for ARM architecture if it doesn't exist
docker buildx ls | grep arm_builder > /dev/null 2>&1 || docker buildx create --name arm_builder --use

# Building Docker images if not skipped
if [ "$SKIP_API" = false ]; then
    echo "Building Rust API image for ARM architecture..."
    docker buildx build --platform $ARM_VERSION -t $API_IMAGE_NAME:latest --load -f raspi-rust-api/Dockerfile.armv7 raspi-rust-api/. || handle_error "Rust API image build failed"
    echo "Saving Rust API image..."
    docker save $API_IMAGE_NAME:latest -o $API_IMAGE_NAME.tar || handle_error "Saving API image failed"
fi

if [ "$SKIP_FRONTEND" = false ]; then
    echo "Building Frontend image for ARM architecture..."
    docker buildx build --platform $ARM_VERSION -t $FRONTEND_IMAGE_NAME:latest --load -f frontend/Dockerfile frontend/. || handle_error "Frontend image build failed"
    echo "Saving Frontend image..."
    docker save $FRONTEND_IMAGE_NAME:latest -o $FRONTEND_IMAGE_NAME.tar || handle_error "Saving Frontend image failed"
fi

# Transferring Docker images and the compose file to Raspberry Pi if not skipped
if [ "$SKIP_API" = false ]; then
    echo "Transferring API image to Raspberry Pi..."
    scp $API_IMAGE_NAME.tar $PI_USERNAME@$PI_HOSTNAME:$PI_DIRECTORY || handle_error "Transferring API image failed"
fi

if [ "$SKIP_FRONTEND" = false ]; then
    echo "Transferring Frontend image to Raspberry Pi..."
    scp $FRONTEND_IMAGE_NAME.tar $PI_USERNAME@$PI_HOSTNAME:$PI_DIRECTORY || handle_error "Transferring Frontend image failed"
fi

# Always transfer the docker-compose file and environment variables files
echo "Transferring docker-compose file and environment variable files to Raspberry Pi..."
scp $COMPOSE_FILE $PI_USERNAME@$PI_HOSTNAME:$PI_DIRECTORY || handle_error "Transferring compose file failed"
scp .env.db $PI_USERNAME@$PI_HOSTNAME:$PI_DIRECTORY || handle_error "Transferring .env.db failed"
scp raspi-rust-api/.env.prod $PI_USERNAME@$PI_HOSTNAME:$PI_DIRECTORY || handle_error "Transferring API .env failed"
scp frontend/.env $PI_USERNAME@$PI_HOSTNAME:$PI_DIRECTORY || handle_error "Transferring Frontend .env failed"

# SSH into Raspberry Pi and start the services
echo "Connecting to the Raspberry Pi to start the containers..."
ssh -t $PI_USERNAME@$PI_HOSTNAME << EOF
set -e
cd $PI_DIRECTORY
echo "Stopping and removing any existing containers..."
docker compose -f $COMPOSE_FILE down || exit 1

if [ "$SKIP_API" = false ]; then
    echo "Loading API image..."
    docker load -i $API_IMAGE_NAME.tar || exit 1
fi

if [ "$SKIP_FRONTEND" = false ]; then
    echo "Loading Frontend image..."
    docker load -i $FRONTEND_IMAGE_NAME.tar || exit 1
fi

echo "Starting containers..."
docker compose -f $COMPOSE_FILE up -d || exit 1

if [ "$ATTACH_LOGS" = true ]; then
    echo "Attaching to containers..."
    docker compose -f $COMPOSE_FILE logs -f
else
    echo "Started containers without attaching to logs."
fi
EOF

if [ $? -ne 0 ]; then
    handle_error "Failed to start containers"
fi

echo "Deployment to Raspberry Pi complete."
