#!/usr/bin/env python3
# Tool to generate encoding strings for MappingToken structures used in Balancer V3
# This utility helps encode/decode token mappings for wrapped, underlying, and none-type tokens
# Examples of usage are provided at the bottom of this file

import argparse
import json
import binascii
import sys
from enum import Enum
from typing import List

class TokenType(str, Enum):
    NoneType = "None"
    Wrapped = "Wrapped"
    Underlying = "Underlying"

class MappingToken:
    def __init__(self, addresses: List[bytes], token_type: TokenType):
        self.addresses = addresses
        self.token_type = token_type

    def to_dict(self):
        return {
            "addresses": [[b for b in a] for a in self.addresses],
            "token_type": self.token_type.value
        }

    @staticmethod
    def from_dict(d):
        addresses = [bytes(a) for a in d["addresses"]]
        token_type = TokenType(d["token_type"])
        return MappingToken(addresses, token_type)

def json_serialize_mapping_tokens(tokens: List[MappingToken]) -> str:
    json_bytes = json.dumps([t.to_dict() for t in tokens], separators=(",", ":")).encode("utf-8")
    return binascii.hexlify(json_bytes).decode("utf-8")

def json_deserialize_mapping_tokens(hex_str: str) -> List[MappingToken]:
    json_bytes = binascii.unhexlify(hex_str)
    lst = json.loads(json_bytes)
    return [MappingToken.from_dict(d) for d in lst]

def main():
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="cmd", required=True)

    encode_parser = subparsers.add_parser("encode")
    encode_parser.add_argument("--wrapped", nargs="*", default=[], help="Wrapped token addresses (hex, 0x...)")
    encode_parser.add_argument("--underlying", nargs="*", default=[], help="Underlying token addresses (hex, 0x...)")
    encode_parser.add_argument("--none", nargs="*", default=[], help="None type token addresses (hex, 0x...). Uses zero address (0x0000...0000) if no addresses provided")
    encode_parser.add_argument("--none-count", type=int, default=0, help="Number of none type tokens with zero address to create")

    decode_parser = subparsers.add_parser("decode")
    decode_parser.add_argument("hex_str", help="Encoded hex string to decode")

    args = parser.parse_args()

    if args.cmd == "encode":
        tokens = []

        # Handle wrapped tokens
        for addr in args.wrapped:
            addr_clean = addr.lower().removeprefix("0x")
            tokens.append(MappingToken([bytes.fromhex(addr_clean)], TokenType.Wrapped))

        # Handle underlying tokens
        for addr in args.underlying:
            addr_clean = addr.lower().removeprefix("0x")
            tokens.append(MappingToken([bytes.fromhex(addr_clean)], TokenType.Underlying))

        # Handle none type tokens
        none_flag_explicitly_used = '--none' in sys.argv

        if none_flag_explicitly_used:
            if len(args.none) == 0:  # --none flag used without addresses
                zero_address = bytes(20)  # 20 bytes of zeros (0x0000...0000)
                tokens.append(MappingToken([zero_address], TokenType.NoneType))
            else:  # Specific addresses provided
                for addr in args.none:
                    addr_clean = addr.lower().removeprefix("0x")
                    tokens.append(MappingToken([bytes.fromhex(addr_clean)], TokenType.NoneType))

        # Handle none-count for multiple zero address tokens
        if args.none_count > 0:
            zero_address = bytes(20)  # 20 bytes of zeros (0x0000...0000)
            for _ in range(args.none_count):
                tokens.append(MappingToken([zero_address], TokenType.NoneType))

        encoded = json_serialize_mapping_tokens(tokens)
        print(encoded)

    elif args.cmd == "decode":
        tokens = json_deserialize_mapping_tokens(args.hex_str)
        for t in tokens:
            addresses_hex = ["0x" + a.hex() for a in t.addresses]
            print(f"TokenType: {t.token_type}, Addresses: {addresses_hex}")

if __name__ == "__main__":
    main()

# encode examples:
# python codec.py encode --underlying c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2 7f39c581f595b53c5cb19bd0b3f8da6c935e2ca0
# python codec.py encode --none  # Uses zero address
# python codec.py encode --none 1234567890abcdef1234567890abcdef12345678  # Uses specific address
# python codec.py encode --wrapped abc123 --underlying def456 --none  # Mixed types with zero address for none