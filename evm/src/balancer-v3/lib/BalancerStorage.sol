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
    address immutable permit2;

    enum CUSTOM_WRAP_KIND {
        NONE,
        ERC20_TO_ERC20, // swap ERC20 to ERC20, passing through a ERC4626_4626 pool
            // pool
        ERC4626_TO_ERC4626 // swap ERC4626 to ERC4626, passing through a
            // ERC20_20_20 pool

    }

    enum ERC4626_SWAP_TYPE {
        ERC20_SWAP, // ERC20->ERC20->ERC4626
        ERC20_WRAP, // ERC20->ERC4626->ERC4626
        ERC4626_UNWRAP, // ERC4626->ERC20->ERC20
        ERC4626_SWAP // ERC4626->ERC4626->ERC20
    }
}
