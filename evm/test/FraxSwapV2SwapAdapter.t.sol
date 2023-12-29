// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";
import "src/fraxswap-v2/FraxSwapV2SwapAdapter.sol";

///// Ethereum Network
//// factory Address: 0x43eC799eAdd63848443E2347C49f5f52e8Fe0F6f
/// FRAX-WETH Pair Address: 0x31351Bf3fba544863FBff44DDC27bA880916A199
// t0 - FRAX Token Address: 0x853d955aCEf822Db058eb8505911ED77F175b99e
// t1 - WETH Token Address: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2

contract FraxSwapV2SwapAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    FraxSwapV2SwapAdapter adapter;
    IERC20 constant FRAX = IERC20(0x853d955aCEf822Db058eb8505911ED77F175b99e);
    IERC20 constant WETH = IERC20(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2);
    address constant FRAX_WETH_PAIR = 0x31351Bf3fba544863FBff44DDC27bA880916A199;
    address constant factoryAddress = 0x43eC799eAdd63848443E2347C49f5f52e8Fe0F6f;

    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 18892426;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new FraxSwapV2SwapAdapter(factoryAddress);

        vm.label(address(FRAX), "FRAX");
        vm.label(address(WETH), "WETH");
        vm.label(address(FRAX_WETH_PAIR), "FRAX_WETH_PAIR");
    }

    function testGetCapabilitiesFrax(bytes32 pair, address t0, address t1) public {
        Capability[] memory res =
            adapter.getCapabilities(pair, IERC20(t0), IERC20(t1));

        assertEq(res.length, 3);
    }

    function testGetLimitsFrax() public {
        bytes32 pair = bytes32(bytes20(FRAX_WETH_PAIR));
        uint256[] memory limits = adapter.getLimits(pair, FRAX, WETH);

        assertEq(limits.length, 2);
        console.logString("Sell FRAX Limit");
        console.logUint(limits[0]);
        console.logString("Buy WETH Limit");
        console.logUint(limits[1]);
    }   

}