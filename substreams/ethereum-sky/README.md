# ethereum_sky Substreams modules

This package was initialized via `substreams init`, using the `evm-events-calls` template.

## Usage

```bash
substreams build
substreams auth
substreams gui       			  # Get streaming!
substreams registry login         # Login to substreams.dev
substreams registry publish       # Publish your Substreams to substreams.dev
```

## Modules

All of these modules produce data filtered by these contracts:
- _sdai_ at **0x83f20f44975d03b1b09e64809b757c47f942beea**
- _dai_usds_converter_ at **0x3225737a9bbb6473cb4a45b7244aca2befdb276a**
- _dai_lite_psm_ at **0xf6e72db5454dd049d0788e411b06cfaf16853042**
- _usds_psm_wrapper_ at **0xa188eec8f81263234da3622a406892f3d630f98c**
- _susds_ at **0xa3931d71877c0e7a3148cb7eb4463524fec27fbd**
- _mkr_sky_converter_ at **0xbdcfca946b6cdd965f99a839e4435bcdc1bc470b**
- _usdc_ at **0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48**
### `map_events_calls`

This module gets you events _and_ calls


### `map_events`

This module gets you only events that matched.



### `map_calls`

This module gets you only calls that matched.

## Sky Substreams Events broke down by contract

- sdai
- dai_usds_converter
- dai_lite_psm
- usds_psm_wrapper
- susds
- mkr_sky_converter

# Contracts and their deployment details

## sdai
- address: 0x83F20F44975D03b1b09e64809B757c47f942BEeA
- initial_block_number: 16428133
- Age: Jan-17-2023 05:52:11 PM UTC
- creation_tx: 0xa2f51048265f2fe9ffaf69b94cb5a2a4113be49bdecd2040d530dd6f68facc42
- contract_creator: 0x3249936bDdF8bF739d3f06d26C40EEfC81029BD1 (MakerDAO: Deployer)

## dai_usds_converter
- address: 0x3225737a9Bbb6473CB4a45b7244ACa2BeFdB276A
- initial_block_number: 20663734
- Age: Sep-02-2024 03:19:35 PM UTC
- creation_tx: 0xb63d6f4cfb9945130ab32d914aaaafbad956be3718176771467b4154f9afab61
- contract_creator: 0x4Ec216c476175a236BD70026b984D4adECa0cfb8 (Sky: Deployer)

## dai_lite_psm
- address: 0xf6e72Db5454dd049d0788e411b06CfAF16853042
- initial_block_number: 20283666
- Age: Jul-11-2024 01:48:47 PM UTC
- creation_tx: 0x61e5d04f14d1fea9c505fb4dc9b6cf6e97bc83f2076b53cb7e92d0a2e88b6bbd
- contract_creator: 0xb27B6fa77D7FBf3C1BD34B0f7DA59b39D3DB0f7e

## usds_psm_wrapper
- address: 0xA188EEC8F81263234dA3622A406892F3D630f98c
- initial_block_number: 20668728
- Age: Sep-03-2024 08:02:59 AM UTC
- creation_tx: 0x43ddae74123936f6737b78fcf785547f7f6b7b27e280fe7fbf98c81b3c018585
- contract_creator: 0x4E65a603a9170fa572E276D1B70D6295D433bAc5

## susds
- address: 0xa3931d71877C0E7a3148CB7Eb4463524FEc27fbD
- initial_block_number: 20677434
- Age: Sep-03-2024 08:02:59 AM UTC
- creation_tx: 0xe1be00c4ea3c21cf536b98ac082a5bba8485cf75d6b2b94f4d6e3edd06472c00
- contract_creator: 0xD6ec7a1b1f4c42C5208fF68b2436Fab8CC593fB7 (Sky Deployer 6)

## mkr_sky_converter
- address: 0xBDcFCA946b6CDd965f99a839e4435Bcdc1bc470B
- initial_block_number: 20663740
- Age: Sep-02-2024 03:20:47 PM UTC
- creation_tx: 0xbd89595dadba76ffb243cb446a355cfb833c1ea3cefbe427349f5b4644d5fa02
- contract_creator: 0x4Ec216c476175a236BD70026b984D4adECa0cfb8 (Sky: Deployer)