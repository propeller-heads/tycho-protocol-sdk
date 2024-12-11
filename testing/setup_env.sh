#!/bin/zsh

# Variables
ENV_NAME="propeller-protocol-lib-testing"
PYTHON_VERSION="3.9"
REQUIREMENTS_FILE="requirements.txt"

# Exit immediately if any command fails
set -e

# Step 1: Source environment variables
if [ -f ".env" ]; then
  source .env
else
  echo ".env file not found! Exiting."
  exit 1
fi

# Step 5: Create conda environment
echo "Creating conda environment ${ENV_NAME} with Python ${PYTHON_VERSION}..."
conda create --name $ENV_NAME python=$PYTHON_VERSION -y

# Step 6: Installing the requirements using conda run
echo "Installing packages in the environment..."
echo "Installing the requirements from ${REQUIREMENTS_FILE}..."
source ./pre_build.sh
conda run -n $ENV_NAME pip install -r $REQUIREMENTS_FILE

# Step 6: Installing additional packages using conda run
conda run -n $ENV_NAME pip install --upgrade --force-reinstall "$HOME/Downloads/protosim_py-0.23.1-cp39-cp39-macosx_11_0_arm64.whl"
conda run -n $ENV_NAME pip install psycopg2-binary

# Step 7: Launch Docker Desktop (if installed)
if ! pgrep -x "Docker" > /dev/null; then
  open -a Docker || {
    echo "Docker Desktop could not be opened. Please make sure it is installed."
    exit 1
  }
fi

# Step 8: Wait for Docker Desktop to be running
RETRY_COUNT=0
MAX_RETRIES=10
while ! docker info >/dev/null 2>&1; do
  if [ $RETRY_COUNT -ge $MAX_RETRIES ]; then
    echo "Docker Desktop did not start after $MAX_RETRIES attempts. Exiting."
    exit 1
  fi
  echo "Waiting for Docker Desktop to launch... (Attempt $((RETRY_COUNT+1))/$MAX_RETRIES)"
  RETRY_COUNT=$((RETRY_COUNT+1))
  sleep 5
done

# Step 9: Bring down any running Docker containers
docker compose down

# Step 10: Start the database container
docker compose up -d db

# Step 11: Print success message
echo "Testing environment setup complete"