// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";
import "src/camelot-v3/CamelotV3Adapter.sol";

contract CamelotV3AdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    CamelotV3Adapter adapter;
    address constant WETH = 0x82aF49447D8a07e3bd95BD0d56f35241523fBab1;
    address constant GRAIL = 0x3d9907F9a368ad0a51Be60f7Da3b97cf940982D8;
    address constant WETH_GRAIL_PAIR =
        0x60451B6aC55E3C5F0f3aeE31519670EcC62DC28f;
    address constant quoter = 0x0Fc73040b26E9bC8514fA028D998E73A254Fa76E;

    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 232411918;
        vm.createSelectFork(vm.rpcUrl("arb_mainnet"), forkBlock);
        adapter = new CamelotV3Adapter(quoter);

        vm.label(address(WETH), "WETH");
        vm.label(GRAIL, "GRAIL");
        vm.label(address(WETH_GRAIL_PAIR), "WETH_GRAIL_PAIR");
    }

    function testPriceFuzzCamelotV3(uint256 amount0, uint256 amount1) public {
        bytes32 pair = bytes32(bytes20(WETH_GRAIL_PAIR));
        uint256[] memory limits = adapter.getLimits(pair, GRAIL, WETH);
        vm.assume(amount0 < limits[0] && amount0 > 10 ** 2);
        vm.assume(amount1 < limits[1] && amount1 > 10 ** 2);

        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;

        Fraction[] memory prices = adapter.price(pair, GRAIL, WETH, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testSwapFuzzCamelotV3(uint256 specifiedAmount, bool isBuy)
        public
    {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(WETH_GRAIL_PAIR));
        uint256[] memory limits = adapter.getLimits(pair, GRAIL, WETH);

        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1] && specifiedAmount > 10);

            deal(GRAIL, address(this), type(uint256).max);
            IERC20(GRAIL).approve(address(adapter), type(uint256).max);
        } else {
            vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10);

            deal(GRAIL, address(this), type(uint256).max);
            IERC20(GRAIL).approve(address(adapter), specifiedAmount);
        }

        uint256 GRAIL_balance_before = IERC20(GRAIL).balanceOf(address(this));
        uint256 weth_balance_before = IERC20(WETH).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, GRAIL, WETH, side, specifiedAmount);

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    IERC20(WETH).balanceOf(address(this)) - weth_balance_before
                );

                assertEq(
                    trade.calculatedAmount,
                    GRAIL_balance_before
                        - IERC20(GRAIL).balanceOf(address(this))
                );
            } else {
                assertEq(
                    specifiedAmount,
                    GRAIL_balance_before
                        - IERC20(GRAIL).balanceOf(address(this))
                );

                assertEq(
                    trade.calculatedAmount,
                    IERC20(WETH).balanceOf(address(this)) - weth_balance_before
                );
            }
        }
    }

    function testSwapSellIncreasingCamelotV3() public {
        executeIncreasingSwapsCamelotV3(OrderSide.Sell);
    }

    function testSwapBuyIncreasing() public {
        executeIncreasingSwapsCamelotV3(OrderSide.Buy);
    }

    function executeIncreasingSwapsCamelotV3(OrderSide side) internal {
        bytes32 pair = bytes32(bytes20(WETH_GRAIL_PAIR));

        uint256[] memory limits = adapter.getLimits(pair, WETH, GRAIL);
        uint256 amountConstant_ =
            side == OrderSide.Sell ? limits[0] / 1000 : limits[1] / 1000;

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        amounts[0] = amountConstant_;
        for (uint256 i = 1; i < TEST_ITERATIONS; i++) {
            amounts[i] = amountConstant_ * i;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 1; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            if (side == OrderSide.Buy) {
                deal(GRAIL, address(this), type(uint256).max);
            } else {
                deal(GRAIL, address(this), amounts[i]);
            }
            IERC20(GRAIL).approve(address(adapter), type(uint256).max);

            trades[i] = adapter.swap(pair, GRAIL, WETH, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    function testGetCapabilitiesCamelotV3(bytes32 pair, address t0, address t1)
        public
    {
        Capability[] memory res = adapter.getCapabilities(pair, t0, t1);

        assertEq(res.length, 3);
    }

    function testGetTokensCamelotV3() public {
        bytes32 pair = bytes32(bytes20(WETH_GRAIL_PAIR));
        address[] memory tokens = adapter.getTokens(pair);

        assertEq(tokens.length, 2);
    }

    function testGetLimitsCamelotV3() public {
        bytes32 pair = bytes32(bytes20(WETH_GRAIL_PAIR));
        uint256[] memory limits = adapter.getLimits(pair, GRAIL, WETH);

        assertEq(limits.length, 2);
    }
}
