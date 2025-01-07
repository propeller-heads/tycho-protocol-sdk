// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "./AdapterTest.sol";
import {NablaPortalSwapAdapter, IERC20, INablaPortal} from "src/nabla/NablaPortalSwapAdapter.sol";
import {FractionMath} from "src/libraries/FractionMath.sol";

contract NablaPortalSwapAdapterTest is AdapterTest {
    using FractionMath for Fraction;

    INablaPortal constant nablaPortal =
        INablaPortal(payable(0xcB94Eee869a2041F3B44da423F78134aFb6b676B));
    NablaPortalSwapAdapter adapter;

    function setUp() public {
        uint256 forkBlock = 280000000;
        vm.createSelectFork(vm.rpcUrl("arbitrum"), forkBlock);

        adapter = new NablaPortalSwapAdapter(payable(address(nablaPortal)));

        vm.label(address(nablaPortal), "INablaPortal");
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
        assertEq(res.length, 5);
        assertEq(uint256(res[0]), uint256(Capability.SellOrder));
        assertEq(uint256(res[1]), uint256(Capability.BuyOrder));
        assertEq(uint256(res[2]), uint256(Capability.PriceFunction));
        assertEq(uint256(res[3]), uint256(Capability.ScaledPrices));
        assertEq(uint256(res[4]), uint256(Capability.HardLimits));
    }
}
