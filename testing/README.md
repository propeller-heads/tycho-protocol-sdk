# Substreams Testing

This package provides a comprehensive testing suite for Substreams modules. The testing suite is designed to facilitateend-to-end testing, ensuring that your Substreams modules function as expected.

## Overview

The testing suite builds the `.spkg` for your Substreams module, indexes a specified block range, and verifies that the expected state has been correctly indexed in PostgreSQL. Additionally, it will simulate some transactions using the `SwapAdapter` interface.

## Prerequisites

To use this testing suite, ensure the following requirements are met:

- Latest version of `tycho_indexer`, in a directory included in your system’s PATH.
- Access to PropellerHeads' private PyPI repository and login credentials.
- Chainstack `RPC_URL` for the Ethereum mainnet.
- `DOMAIN_OWNER` for PropellerHeads' AWS account.
- A `STREAMINGFAST_KEY` and `SUBSTREAMS_API_TOKEN`. Sign up at [The Graph Market](https://thegraph.market/dashboard) and create a project.
- Docker installed on your machine.
- [Conda](https://conda.io/projects/conda/en/latest/user-guide/install/index.html) installed.
- [AWS CLI](https://aws.amazon.com/cli/) installed.
<br/>
> **Contact us** to get `tycho_indexer`, PropellerHeads' private PyPI repository access, Chainstack `RPC_URL`, and `DOMAIN_OWNER`, contact us.

## Setting Up the Testing Environment

### First-Time Setup

Follow these steps if this is your first time setting up the `propeller-protocol-lib-testing` environment:

1. Navigate to the `propeller-protocol-lib/testing` directory.
2. Create a `.env` file with the following variables:
    ```bash
    export DOMAIN_OWNER=
    export RPC_URL=
    export STREAMINGFAST_KEY=
    export SUBSTREAMS_API_TOKEN=
    ```
3. Load the environment variables:
    ```bash
    source .env
    ```
4. Generate and export the `CODEARTIFACT_AUTH_TOKEN`:
    ```bash
    CODEARTIFACT_AUTH_TOKEN=$(aws --region eu-central-1 codeartifact get-authorization-token --domain propeller --domain-owner "$DOMAIN_OWNER" --query authorizationToken --output text --duration 1800)
    PIP_INDEX_URL="https://aws:${CODEARTIFACT_AUTH_TOKEN}@propeller-${DOMAIN_OWNER}.d.codeartifact.eu-central-1.amazonaws.com/pypi/protosim/simple/"
    export PIP_INDEX_URL
    ```
    > ❗ **Note:** The `CODEARTIFACT_AUTH_TOKEN` expires every 12 hours and needs regeneration.
5. Load the `.env` file again:
    ```bash
    source .env
    ```
6. Configure AWS CLI:
    ```bash
    aws configure
    ```
    - Enter your AWS key, secret, and region (`eu-central-1`).
7. Authenticate and install dependencies:
    ```bash
    aws codeartifact login --tool pip --repository protosim --domain propeller
    pip install --upgrade --force-reinstall protosim-py==<desired-version>
    ```
    - If the desired version is unavailable, contact us for the file, then run:
      ```bash
      pip install --upgrade --force-reinstall <path-to-file>/<protosim_py-version>
      ```
    - In case of missing dependencies like `tycho-indexer-client`, run:
      ```bash
      pip install --upgrade tycho-indexer-client
      ```
8. Proceed with the [automated setup](#automated-setup).

### Automated Setup

> ❗ **Note:** If the automated setup fails, please follow the [manual setup](#manual-setup)

You can follow this section anytime you want to reinitialize the testing environment.
If it's the first time you are setting up the repo, please follow instead the [first time setup](#first-time-setup).

1. Navigate to the `propeller-protocol-lib/testing` directory.
2. Load the environment variables:
    ```bash
    source .env
    ```
3. Run the setup script:
    ```bash
    ./setup_env.sh
    ```
4. Create python virtual environment for testing:
    ```bash
    conda activate propeller-protocol-lib-testing
    ```
5. Return to the root directory:
    ```bash
    cd ..
    ```
6. Run integration tests:
    ```bash
    python ./testing/src/runner/cli.py --package "<your-package-name>"
    ```
    - For VM traces:
      ```bash
      python ./testing/src/runner/cli.py --package "<your-package-name>" --vm-traces
      ```
    - For debugging:
      ```bash
      python ./testing/src/runner/cli.py --package "<your-package-name>" --tycho-logs
      ```

## Test Configuration

Tests are defined in a `yaml` file. A documented template can be found at
`substreams/ethereum-template/integration_test.tycho.yaml`. 
The configuration file should include:

- The target Substreams config file.
- The corresponding SwapAdapter and args to build it.
- The expected protocol types.
- The tests to be run.

Each test will index all blocks between `start-block` and `stop-block`, verify that the indexed state matches the expected state and optionally simulate transactions using `SwapAdapter` interface.

### VM Runtime File

A VM runtime file is required for the adapter contract. The testing script can build it using your test config, or you can generate it manually using the script located at:

```
evm/scripts/buildRuntime.sh
```

## `setup_env.sh` Script Breakdown

The `setup_env.sh` script automates the environment initialization process. Below is a step-by-step breakdown of the script:

1. **Environment Variables**
   - Verifies and sources the `.env` file
   - Exits if `.env` file is not found

2. **Conda Environment Setup**
   - Creates a new conda environment named `propeller-protocol-lib-testing`
   - Uses Python 3.9 as the base interpreter

3. **Dependencies Installation**
   - Runs `pre_build.sh` script for initial setup
   - Installs requirements from `requirements.txt`
   - Installs specific version of `protosim_py` wheel file
   - Installs `psycopg2-binary` for PostgreSQL connectivity

4. **Docker Setup**
   - Launches Docker Desktop if not running
   - Waits for Docker to be fully operational (up to 10 retry attempts)
   - Brings down any existing containers
   - Starts the database container

### Manual Setup

For a manual setup without the `setup_env.sh` script, follow these steps:

1. Source your environment variables:
   ```bash
   source .env
   ```

2. Create a new conda environment:
   ```bash
   conda create --name propeller-protocol-lib-testing python=3.9 -y
   conda activate propeller-protocol-lib-testing
   ```

3. Install dependencies:
   ```bash
   source ./pre_build.sh
   pip install -r requirements.txt
   pip install --upgrade --force-reinstall "<path-to-file>/<protosim_py-version>"
   pip install psycopg2-binary
   ```

4. Start Docker services:
   ```bash
   # Start Docker Desktop
   open -a Docker
   
   # Wait for Docker to be ready
   docker compose down
   docker compose up -d db
   ```
5. Activate conda environment:
   ```bash
   conda activate propeller-protocol-lib-testing
   ```


