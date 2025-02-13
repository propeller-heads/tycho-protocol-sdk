# ethereum_renzo Substreams modules

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
- _restake_manager_ at **0x74a09653a083691711cf8215a6ab074bb4e99ef5**
- _withdrawal_contract_ at **0x5efc9d10e42fb517456f4ac41eb5e2ebe42c8918**
### `map_events_calls`

This module gets you events _and_ calls


### `map_events`

This module gets you only events that matched.



### `map_calls`

This module gets you only calls that matched.


