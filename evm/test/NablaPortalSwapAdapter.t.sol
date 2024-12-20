// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "./AdapterTest.sol";
import {NablaPortalSwapAdapter, IERC20} from "src/nabla/NablaPortalSwapAdapter.sol";
import {FractionMath} from "src/libraries/FractionMath.sol";

contract NablaPortalSwapAdapterTest is AdapterTest {
    using FractionMath for Fraction;

    NablaPortalSwapAdapter adapter;

    address constant ORACLE_ADAPTER =
        0x1234567890123456789012345678901234567890; // Replace with actual address
    address constant GUARD_ORACLE = 0x2345678901234567890123456789012345678901; // Replace with actual address
    address constant WETH = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2; // mainnet wETH
    // address constant WETH = 0x82aF49447D8a07e3bd95BD0d56f35241523fBab1; // arbitrum wETH

    function setUp() public {
        uint256 forkBlock = 18710000;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);

        adapter = new NablaPortalSwapAdapter(
            ORACLE_ADAPTER,
            GUARD_ORACLE,
            WETH
        );

        vm.label(ORACLE_ADAPTER, "OracleAdapter");
        vm.label(GUARD_ORACLE, "GuardOracle");
        vm.label(WETH, "WETH");
        vm.label(address(adapter), "NablaPortalSwapAdapter");
    }

    function testPriceNotImplemented() public {
        uint256[] memory amounts = new uint256[](1);
        amounts[0] = 1e18;

        vm.expectRevert(
            abi.encodeWithSelector(
                NotImplemented.selector,
                "NablaPortalSwapAdapter.price"
            )
        );
        adapter.price(bytes32(0), address(0), address(0), amounts);
    }

    // function testPriceFuzz(uint256 amount0, uint256 amount1) public {}

    // function testSwapFuzz(uint256 specifiedAmount) public {}

    function testGetCapabilities(
        bytes32 pair,
        address t0,
        address t1
    ) public view {
        Capability[] memory res = adapter.getCapabilities(pair, t0, t1);

        assertEq(res.length, 4);
        assertEq(uint256(res[0]), uint256(Capability.SellOrder));
        assertEq(uint256(res[1]), uint256(Capability.BuyOrder));
        assertEq(uint256(res[2]), uint256(Capability.PriceFunction));
    }

    // function testGetLimits() public view {
    //     bytes32 pair = bytes32(bytes20(USDC_WETH_PAIR));
    //     uint256[] memory limits = adapter.getLimits(pair, WETH, WETH);

    //     assertEq(limits.length, 2);
    //     assertGt(limits[0], 0);
    //     assertGt(limits[1], 0);
    // }
}
