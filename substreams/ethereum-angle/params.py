import json
from typing import Any

PARAMETERS = "params.json"


def encode_json_to_query_params(params: list[dict[str, Any]]):
    creation_block_nos: list[str] = [param["creation_block_no"] for param in params]
    creation_hashes: list[str] = [param["creation_hash"] for param in params]
    proxies: list[str] = [param["proxy"] for param in params]
    stablecoins: list[str] = [param["stablecoin"] for param in params]
    anglecoins: list[str] = [param["anglecoin"] for param in params]

    encoded_block_nos = "&".join(
        f"creation_block_nos[]={no}" for no in creation_block_nos
    )
    encoded_hashes = "&".join(f"creation_hashes[]={hash}" for hash in creation_hashes)
    encoded_proxies = "&".join(f"proxies[]={proxy}" for proxy in proxies)
    encoded_stablecoins = "&".join(
        f"stablecoins[]={stablecoin}" for stablecoin in stablecoins
    )
    encoded_anglecoins = "&".join(
        f"anglecoins[]={anglecoin}" for anglecoin in anglecoins
    )

    return f"{encoded_block_nos}&{encoded_hashes}&{encoded_proxies}&{encoded_stablecoins}&{encoded_anglecoins}"


def main():
    with open(PARAMETERS, "r") as f:
        params = json.load(f)
    print(encode_json_to_query_params(params))


if __name__ == "__main__":
    main()
