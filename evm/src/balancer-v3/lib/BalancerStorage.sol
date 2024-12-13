//SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.26;

import "./BalancerInterfaces.sol";

/**
 * @title Balancer V3 Storage
 */
abstract contract BalancerStorage {
    // Balancer V3 constants
    uint256 constant RESERVE_LIMIT_FACTOR = 7; // 70% as being divided by 10
    uint256 constant SWAP_DEADLINE_SEC = 1000;

    // Balancer V3 contracts
    IVault immutable vault;
    IBatchRouter immutable router;

    // ETH and Wrapped ETH addresses, using ETH as address(0)
    address constant WETH_ADDRESS = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
    address constant ETH_ADDRESS = address(0);

    // permit2 address
    address permit2;
}
