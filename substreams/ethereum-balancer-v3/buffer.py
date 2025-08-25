# Tool script to fetch all initialized liquidity buffer tokens from Balancer V3 Vault
# This is not main application logic - it's a utility to discover buffer token mappings
# and organize them as a mapping structure for configuration purposes.

import os
from web3 import Web3

rpc_url = os.environ.get("ETH_RPC_URL")
if not rpc_url:
    raise ValueError("Environment variable ETH_RPC_URL is not set")

w3 = Web3(Web3.HTTPProvider(rpc_url))

vault_address = "0xbA1333333333a1BA1108E8412f11850A5C319bA9"
vault_abi = '[{"anonymous":false,"inputs":[{"indexed":true,"name":"wrappedToken","type":"address"},{"indexed":false,"name":"amountUnderlying","type":"uint256"},{"indexed":false,"name":"amountWrapped","type":"uint256"},{"indexed":false,"name":"bufferBalances","type":"bytes32"}],"name":"LiquidityAddedToBuffer","type":"event"}]'
vault_explorer_address = "0xFc2986feAB34713E659da84F3B1FA32c1da95832"
vault_explorer_abi = '[{"constant":true,"inputs":[{"name":"wrappedToken","type":"address"}],"name":"getBufferAsset","outputs":[{"name":"underlyingToken","type":"address"}],"payable":false,"stateMutability":"view","type":"function"}]'

vault_contract = w3.eth.contract(address=vault_address, abi=vault_abi)
vault_explorer_contract = w3.eth.contract(address=vault_explorer_address, abi=vault_explorer_abi)


start_block = 21416204
batch_size = 2000
end_block = w3.eth.block_number
print(f"[INFO] Latest Ethereum block: {end_block}")

wrapped_tokens = set()
print(f"[INFO] Starting event scan from block {start_block} to {end_block} in batches of {batch_size} blocks.")
for from_block in range(start_block, end_block + 1, batch_size):
    to_block = min(from_block + batch_size - 1, end_block)
    print(f"[INFO] Querying blocks {from_block} to {to_block}...")

    try:
        logs = vault_contract.events.LiquidityAddedToBuffer.get_logs(
            from_block=from_block,
            to_block=to_block
        )
        for log in logs:
            wrapped_token = log.args.wrappedToken
            wrapped_tokens.add(wrapped_token)
    except Exception as e:
        print(f"[WARN] Failed to fetch logs for blocks {from_block}-{to_block}: {e}")

print(f"[INFO] Total unique wrapped tokens found: {len(wrapped_tokens)}")

underlying_tokens = {}
for wrapped_token in wrapped_tokens:
    try:
        underlying_token = vault_explorer_contract.functions.getBufferAsset(wrapped_token).call()
        underlying_tokens[wrapped_token] = underlying_token
        print(f"[INFO] Wrapped: {wrapped_token} -> Underlying: {underlying_token}")
    except Exception as e:
        print(f"[WARN] Failed to fetch underlying token for {wrapped_token}: {e}")


result = "&".join([f"{wrapped_token}={underlying_token}" for wrapped_token, underlying_token in underlying_tokens.items()])
print(result)
