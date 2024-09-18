# Instructions

```bash
bash run.bash
```

## Requirements

Python 3.8+ required (needed for pushing the params into the substream).

## Parameters

The `params.json` file is a list of objects where each object describes an anglecoin and the data we need in order to process and produce a `ProtocolComponent`. The fields are described as followed:

- `name`: This is purely for description
- `creation_block_no`: The block where the transmuter was created (see the proxy creation block)
- `creation_hash`: Said transaction hash from creation block
- `proxy`: The address of the transmuter proxy (diamond contract)
- `stablecoin`: The address of the stablecoin used in the transmuter
- `anglecoin`: The address of the cooresponding angle protocol coin that is connected with the stablecoin.
