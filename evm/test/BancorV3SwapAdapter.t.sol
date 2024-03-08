// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "forge-std/console.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";
import "src/bancor-v3/BancorV3SwapAdapter.sol";

/// @title TemplateSwapAdapterTest
/// @dev This is a template for a swap adapter test.
/// Test all functions that are implemented in your swap adapter, the two test included here are just an example.
/// Feel free to use UniswapV2SwapAdapterTest and BalancerV2SwapAdapterTest as a reference.
contract BancorV3SwapAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    BancorV3SwapAdapter adapter;
    // address constant POOL_COLLECTION = 0xde1B3CcfC45e3F5bff7f43516F2Cd43364D883E4;
    // address constant BANCOR_NETWORK_ADDRESS= 0x3006EB573bA4b6f28C36AAd49d2062C5e82Cfc75;
    address constant BANCOR_NETWORK_PROXY_ADDRESS = 0xeEF417e1D5CC832e619ae18D2F140De2999dD4fB;

    function setUp() public {
        uint256 forkBlock = 19361722;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);

        adapter = new BancorV3SwapAdapter(BANCOR_NETWORK_PROXY_ADDRESS);
    }

    function testGetPoolIdsBancor() public {
        bytes32[] memory ids = adapter.getPoolIds(1, 20);

        console.log(ids.length);
        console.logBytes32(ids[1]);

    }


}