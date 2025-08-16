#!/usr/bin/env python3
import argparse
import json
import binascii
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

    decode_parser = subparsers.add_parser("decode")
    decode_parser.add_argument("hex_str", help="Encoded hex string to decode")

    args = parser.parse_args()

    if args.cmd == "encode":
        tokens = []
        for addr in args.wrapped:
            addr_clean = addr.lower().removeprefix("0x")
            tokens.append(MappingToken([bytes.fromhex(addr_clean)], TokenType.Wrapped))
        for addr in args.underlying:
            addr_clean = addr.lower().removeprefix("0x")
            tokens.append(MappingToken([bytes.fromhex(addr_clean)], TokenType.Underlying))
        encoded = json_serialize_mapping_tokens(tokens)
        print(encoded)

    elif args.cmd == "decode":
        tokens = json_deserialize_mapping_tokens(args.hex_str)
        for t in tokens:
            addresses_hex = ["0x" + a.hex() for a in t.addresses]
            print(f"TokenType: {t.token_type}, Addresses: {addresses_hex}")

if __name__ == "__main__":
    main()

# encode example: python codec.py encode --underlying c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2 7f39c581f595b53c5cb19bd0b3f8da6c935e2ca0