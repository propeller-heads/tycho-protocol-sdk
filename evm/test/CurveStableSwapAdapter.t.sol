// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import "src/curve-stableswap/CurveStableSwapAdapter.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";

contract CurveStableSwapAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    CurveStableSwapAdapter adapter;
    address constant USDT = 0xdAC17F958D2ee523a2206206994597C13D831ec7;
    address constant USDC = 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48;
    address constant USDT_TRIPOOL = 0xbEbc44782C7dB0a1A60Cb6fe97d0b483032FF1C7;

    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 19719570;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new CurveStableSwapAdapter(
            0x90E00ACe148ca3b23Ac1bC8C240C2a7Dd9c2d7f5
        );

        vm.label(address(adapter), "CurveStableSwapAdapter");
        vm.label(USDT, "USDT");
        vm.label(USDC, "USDC");
        vm.label(USDT_TRIPOOL, "USDT_TRIPOOL");
    }

    function testSwapFuzzCurveStableSwap(uint256 specifiedAmount) public {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(USDT_TRIPOOL));
        uint256[] memory limits = adapter.getLimits(pair, USDC, USDT);

        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 6);

        deal(USDC, address(this), specifiedAmount);
        IERC20(USDC).approve(address(adapter), specifiedAmount);

        uint256 usdc_balance = IERC20(USDC).balanceOf(address(this));
        uint256 USDT_balance = IERC20(USDT).balanceOf(address(this));

        Trade memory trade = adapter.swap(
            pair,
            USDC,
            USDT,
            side,
            specifiedAmount
        );

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    IERC20(USDT).balanceOf(address(this)) - USDT_balance
                );
                assertEq(
                    trade.calculatedAmount,
                    usdc_balance - IERC20(USDC).balanceOf(address(this))
                );
            } else {
                assertEq(
                    specifiedAmount,
                    usdc_balance - IERC20(USDC).balanceOf(address(this))
                );
                assertEq(
                    trade.calculatedAmount,
                    IERC20(USDT).balanceOf(address(this)) - USDT_balance
                );
            }
        }
    }

    function testSwapSellIncreasingCurveStableSwap() public {
        executeIncreasingSwaps(OrderSide.Sell);
    }

    function executeIncreasingSwaps(OrderSide side) internal {
        bytes32 pair = bytes32(bytes20(USDT_TRIPOOL));

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * i * 10 ** 6;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(USDC, address(this), amounts[i]);
            IERC20(USDC).approve(address(adapter), amounts[i]);

            trades[i] = adapter.swap(pair, USDC, USDT, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(
                trades[i].calculatedAmount,
                trades[i + 1].calculatedAmount
            );
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    function testGetCapabilitiesCurveStableSwap(
        bytes32 pair,
        address t0,
        address t1
    ) public {
        Capability[] memory res = adapter.getCapabilities(pair, t0, t1);

        assertEq(res.length, 1);
    }

    function testGetTokensCurveStableSwap() public {
        bytes32 pair = bytes32(bytes20(USDT_TRIPOOL));
        address[] memory tokens = adapter.getTokens(pair);

        assertGe(tokens.length, 2);
    }

    function testGetPoolIdsCurveStableSwap() public {
        bytes32 pair = bytes32(bytes20(USDT_TRIPOOL));
        bytes32[] memory poolIds = adapter.getPoolIds(0, 2);

        assertEq(poolIds.length, 2);
    }

    function testGetLimitsCurveStableSwap() public {
        bytes32 pair = bytes32(bytes20(USDT_TRIPOOL));
        uint256[] memory limits = adapter.getLimits(pair, USDC, USDT);

        assertEq(limits.length, 2);
    }
}
