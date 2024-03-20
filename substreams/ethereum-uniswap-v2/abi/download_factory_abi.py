import json
import os
import re
import time
import urllib.request

# Exports contract ABI in JSON
abis = {
    "UniswapV2Factory": "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f",
}

ABI_ENDPOINT = (
    "https://api.etherscan.io/api?module=contract&action=getabi&address={address}"
)

if etherscan_key := os.environ.get("ETHERSCAN_API_TOKEN"):
    print("API KEY Loaded!")
    ABI_ENDPOINT += f"&apikey={etherscan_key}"

def __main__():
    for name, addr in abis.items():
        normalized_name = "_".join(re.findall(r"[A-Z]+[a-z]*", name)).lower()
        print(f"Getting ABI for {name} at {addr} ({normalized_name})")
        try:
            with urllib.request.urlopen(ABI_ENDPOINT.format(address=addr)) as response:
                response_json = json.loads(response.read().decode())
                abi_json = json.loads(response_json["result"])
                result = json.dumps(abi_json, indent=4, sort_keys=True)
                with open(f"{normalized_name}.json", "w") as f:
                    f.write(result)
        except Exception as err:
            print(response.content)
            raise err
        time.sleep(0.25)

if __name__ == "__main__":
    __main__()