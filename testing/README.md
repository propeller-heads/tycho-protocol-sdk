# Substreams Testing

This package provides a comprehensive testing suite for Substreams modules. The testing suite is designed to facilitate
end-to-end testing, ensuring that your Substreams modules function as expected.

## Overview

The testing suite builds the `.spkg` for your Substreams module, indexes a specified block range, and verifies that the
expected state has been correctly indexed in PostgreSQL.
Additionally, it will also try to simulate some transactions using the `SwapAdapter` interface.

## Prerequisites

- Latest version of our `tycho_indexer`. Please contact us to obtain the latest version. Once acquired, place it in a directory that is included in your system’s PATH.
- Access to PropellerHeads' private PyPI repository and login credentials. Please contact us.
- Chainstack `RPC_URL` for the Ethereum mainnet. Please contact us to get it.
- The `DOMAIN_OWNER` for PropellerHeads' AWS account. Please contact us to get it.

- A `STREAMINGFAST_KEY` and `SUBSTREAMS_API_TOKEN`, you'll need to sign up to [TheGraph Market](https://thegraph.market/dashboard) and create a new project.
- Docker installed on your machine.
- [Conda](https://conda.io/projects/conda/en/latest/user-guide/install/index.html) installed on your machine.
- [AWS cli](https://aws.amazon.com/cli/) installed on your machine.

## First time `propeller-protocol-lib-testing` environment setup

Once satisfied all the [prerequisites](#prerequisites) you can continue with the setup.

1. Head to `propeller-protocol-lib/testing` directory.
2. Create a `.env` file and put the following variables inside:
    ```bash
    export DOMAIN_OWNER=827659017777
    export RPC_URL=https://ethereum-mainnet.core.chainstack.com/71bdd37d35f18d55fed5cc5d138a8fac
    export STREAMINGFAST_KEY=
    export SUBSTREAMS_API_TOKEN=
    ```
2. Run `source .env` to load the environment variables.
3. Run:
    ```bash
    CODEARTIFACT_AUTH_TOKEN=$(aws --region eu-central-1 codeartifact get-authorization-token --domain propeller --domain-owner "$DOMAIN_OWNER" --query authorizationToken --output text --duration 1800)
    PIP_INDEX_URL="https://aws:${CODEARTIFACT_AUTH_TOKEN}@propeller-${DOMAIN_OWNER}.d.codeartifact.eu-central-1.amazonaws.com/pypi/protosim/simple/"
    export PIP_INDEX_URL
    ```
    > ❗️ **Important:** You’ll nedd to regenerate the `CODEARTIFACT_AUTH_TOKEN` every 12 hours.
4. Run `source .env` again.
5. Run `aws configure`, you'll be prompted to enter the following: 
    - `key` (you should have received it from us).
    - `secret` (you should have received it from us).
    - `region`: `eu-central-1`.
6. Run `aws codeartifact login --tool pip --repository protosim --domain propeller`
7. Run `pip install --upgrade --force-reinstall protosim-py==<desired-version>` (ask us for the version you need).
> ❗️ **Important:** If the desired version is not available tell us and we'll provide you with the file.
8. If the desired version is not available tell us and we'll provide you with the file.
    - After we have provided you with the file run `pip install --upgrade --force-reinstall <path-to-file>/<protosim_py-version>` 
    - In case of tycho-indexer-client not found, run `pip install --upgrade tycho-indexer-client`
9. Now follow the [new testing environment automated setup](#automated-propeller-protocol-lib-testing-environment-setup)


## Automated `propeller-protocol-lib-testing` environment setup
You can follow this section anytime you want to setup the testing environment.
If it's the first time you are setting up the repo, please follow the [first time setup](#first-time-propeller-protocol-lib-testing-environment-setup)

1. Head to `propeller-protocol-lib/testing` directory.
2. Run `source .env` to load the environment variables.
3. Run `./setup_testing_env.sh` to setup the testing environment.
4. Run `conda activate propeller-protocol-lib-testing` to activate the testing environment.
5. Run `cd ..` to go back to the root directory.
6. Run `python ./testing/src/runner/cli.py --package "<your-package-name>"` to run the integration tests.
  - If you want to run the test and check vm-traces, run `python ./testing/src/runner/cli.py --package "<your-package-name>" --vm-traces`
   If you want to run the test and debug, run `python ./testing/src/runner/cli.py --package "<your-package-name>" --tycho-logs`

## Test Configuration

Tests are defined in a `yaml` file. A documented template can be found at
`substreams/ethereum-template/integration_test.tycho.yaml`. The configuration file should include:

- The target Substreams config file.
- The corresponding SwapAdapter and args to build it.
- The expected protocol types.
- The tests to be run.

Each test will index all blocks between `start-block` and `stop-block`, verify that the indexed state matches the
expected state and optionally simulate transactions using `SwapAdapter` interface.

You will also need the VM Runtime file for the adapter contract.
Our testing script should be able to build it using your test config.
The script to generate this file manually is available under `evm/scripts/buildRuntime.sh`.

## Setup testing environment

### Step 1: Export Environment Variables

**DOMAIN_OWNER**

- **Description**: The domain owner identifier for Propellerhead's AWS account, used for authenticating on the private
  PyPI repository.
- **Example**: `export DOMAIN_OWNER=123456789`

### Step 2: Create python virtual environment for testing

Run setup env script. It will create a conda virtual env and install all dependencies.
This script must be run from within the `propeller-protocol-lib/testing` directory.

Please note that some dependencies require access to our private PyPI repository.

```
setup_env.sh
```

## Running Tests

### Prerequisites

This section requires a testing environment setup. If you don’t have it yet, please refer to the [setup testing
environment section](#setup-testing-environment)

### Step 1: Export Environment Variables

Export the required environment variables for the execution. You can find the available environment variables in the
`.env.default` file.
Please create a `.env` file in the `testing` directory and set the required environment variables.

#### Environment Variables

**RPC_URL**

- **Description**: The URL for the Ethereum RPC endpoint. This is used to fetch the storage data. The node needs to be
  an archive node, and support [debug_storageRangeAt](https://www.quicknode.com/docs/ethereum/debug_storageRangeAt).
- **Example**: `export RPC_URL="https://ethereum-mainnet.core.chainstack.com/123123123123"`

**SUBSTREAMS_API_TOKEN**

- **Description**: The API token for accessing Substreams services. This token is required for authentication.
- **Example**: `export SUBSTREAMS_API_TOKEN=eyJhbGci...`

### Step 2: Run tests

Run local postgres database using docker compose

```bash
docker compose up -d db
```

Run tests for your package.

```bash
python ./testing/src/runner/cli.py --package "your-package-name"
```

#### Example

If you want to run tests for `ethereum-balancer-v2`, use:

```bash
conda activate propeller-protocol-lib-testing
export RPC_URL="https://ethereum-mainnet.core.chainstack.com/123123123123"
export SUBSTREAMS_API_TOKEN=eyJhbGci...
docker compose up -d db
python ./testing/src/runner/cli.py --package "ethereum-balancer-v2"
```

#### Testing CLI args

A list and description of all available CLI args can be found using:

```
python ./testing/src/runner/cli.py --help
```
