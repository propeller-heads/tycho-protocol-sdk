// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "./AdapterTest.sol";
import "forge-std/Test.sol";
import "forge-std/console.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";
import "src/bancor-v3/BancorV3SwapAdapter.sol";

/**
 * @title BancorV3SwapAdapterTest
 */
contract BancorV3SwapAdapterTest is Test, ISwapAdapterTypes, AdapterTest {
    using FractionMath for Fraction;

    BancorV3SwapAdapter adapter;

    address constant BANCOR_NETWORK_INFO_V3 =
        0x8E303D296851B320e6a697bAcB979d13c9D6E760;
    address constant BANCOR_NETWORK_V3 =
        0xeEF417e1D5CC832e619ae18D2F140De2999dD4fB;

    address constant ETH = 0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE;
    address constant LINK = 0x514910771AF9Ca656af840dff83E8264EcF986CA;
    address constant BNT = 0x1F573D6Fb3F13d689FF844B4cE37794d79a7FF1C;
    address constant WBTC = 0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599;

    bytes32 constant PAIR = bytes32(0);
    uint256 constant TEST_ITERATIONS = 100;

    receive() external payable {}

    function setUp() public {
        uint256 forkBlock = 21127835;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);

        adapter =
            new BancorV3SwapAdapter(BANCOR_NETWORK_INFO_V3, BANCOR_NETWORK_V3);
    }

    function testPriceFuzzBancorV3LinkBnt(uint256 amount0, uint256 amount1)
        public
    {
        uint256[] memory limits = adapter.getLimits(PAIR, LINK, BNT);
        uint256 minAmount = 1;

        vm.assume(amount0 < limits[0]);
        vm.assume(amount0 > minAmount);
        vm.assume(amount1 < limits[0]);
        vm.assume(amount1 > minAmount);

        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;

        Fraction[] memory prices = adapter.price(PAIR, LINK, BNT, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testPriceFuzzBancorV3BntLink(uint256 amount0, uint256 amount1)
        public
    {
        uint256[] memory limits = adapter.getLimits(PAIR, BNT, LINK);
        uint256 minAmount = 100;

        vm.assume(amount0 < limits[0]);
        vm.assume(amount0 > minAmount);
        vm.assume(amount1 < limits[0]);
        vm.assume(amount1 > minAmount);

        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;

        Fraction[] memory prices = adapter.price(PAIR, BNT, LINK, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testPriceFuzzBancorV3WbtcLink(uint256 amount0, uint256 amount1)
        public
    {
        uint256[] memory limits = adapter.getLimits(PAIR, WBTC, LINK);
        uint256 minAmount = 1;

        vm.assume(amount0 < limits[0]);
        vm.assume(amount0 > minAmount);
        vm.assume(amount1 < limits[0]);
        vm.assume(amount1 > minAmount);

        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;

        Fraction[] memory prices = adapter.price(PAIR, WBTC, LINK, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testPriceDecreasingBancorV3LinkBnt() public {
        bytes32 pair = PAIR;
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * (i + 1) * 10 ** 6;
        }

        Fraction[] memory prices = adapter.price(pair, LINK, BNT, amounts);

        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            assertEq(prices[i].compareFractions(prices[i + 1]), 1);
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    function testPriceDecreasingBancorV3BntLink() public {
        bytes32 pair = PAIR;
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * (i + 1) * 10 ** 6;
        }

        Fraction[] memory prices = adapter.price(pair, BNT, LINK, amounts);

        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            assertEq(prices[i].compareFractions(prices[i + 1]), 1);
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    function testPriceDecreasingBancorV3LinkWbtc() public {
        bytes32 pair = PAIR;
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * (i + 1) * 10 ** 12;
        }

        Fraction[] memory prices = adapter.price(pair, LINK, WBTC, amounts);

        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            assertEq(prices[i].compareFractions(prices[i + 1]), 1);
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    function testPriceDecreasingBancorV3WbtcLink() public {
        bytes32 pair = PAIR;
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * (i + 1) * 10;
        }

        Fraction[] memory prices = adapter.price(pair, WBTC, LINK, amounts);

        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            assertEq(prices[i].compareFractions(prices[i + 1]), 1);
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    function testSwapFuzzBancorV3BntLink(uint256 specifiedAmount, bool isBuy)
        public
    {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        uint256[] memory limits = adapter.getLimits(PAIR, BNT, LINK);

        vm.assume(specifiedAmount > 1000);

        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1]);

            deal(address(BNT), address(this), type(uint256).max);
            IERC20(BNT).approve(address(adapter), type(uint256).max);
        } else {
            vm.assume(specifiedAmount < limits[0]);

            deal(address(BNT), address(this), specifiedAmount);
            IERC20(BNT).approve(address(adapter), specifiedAmount);
        }

        uint256 bnt_balance_before_swap = IERC20(BNT).balanceOf(address(this));
        uint256 link_balance_before_swap = IERC20(LINK).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(PAIR, BNT, LINK, side, specifiedAmount);

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    IERC20(LINK).balanceOf(address(this))
                        - link_balance_before_swap
                );
                assertEq(
                    trade.calculatedAmount,
                    bnt_balance_before_swap
                        - IERC20(BNT).balanceOf(address(this))
                );
            } else {
                assertEq(
                    specifiedAmount,
                    bnt_balance_before_swap
                        - IERC20(BNT).balanceOf(address(this))
                );
                assertEq(
                    trade.calculatedAmount,
                    IERC20(LINK).balanceOf(address(this))
                        - link_balance_before_swap
                );
            }
        }
    }

    function testSwapFuzzBancorV3BntEth(uint256 specifiedAmount, bool isBuy)
        public
    {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        uint256[] memory limits = adapter.getLimits(PAIR, BNT, ETH);

        vm.assume(specifiedAmount > 10 ** 8);

        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1]);

            deal(address(BNT), address(this), type(uint256).max);
            IERC20(BNT).approve(address(adapter), type(uint256).max);
        } else {
            vm.assume(specifiedAmount < limits[0]);

            deal(address(BNT), address(this), specifiedAmount);
            IERC20(BNT).approve(address(adapter), specifiedAmount);
        }

        uint256 bnt_balance_before_swap = IERC20(BNT).balanceOf(address(this));
        uint256 eth_balance_before_swap = address(this).balance;

        Trade memory trade = adapter.swap(PAIR, BNT, ETH, side, specifiedAmount);

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    address(this).balance - eth_balance_before_swap
                );
                assertEq(
                    trade.calculatedAmount,
                    bnt_balance_before_swap
                        - IERC20(BNT).balanceOf(address(this))
                );
            } else {
                assertEq(
                    specifiedAmount,
                    bnt_balance_before_swap
                        - IERC20(BNT).balanceOf(address(this))
                );
                assertEq(
                    trade.calculatedAmount,
                    address(this).balance - eth_balance_before_swap
                );
            }
        }
    }

    function testSwapFuzzBancorV3EthForBnt(uint256 specifiedAmount, bool isBuy)
        public
    {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        uint256 amountEth = 10 ** 25;
        uint256 eth_balance_before_swap;

        uint256[] memory limits = adapter.getLimits(PAIR, ETH, BNT);

        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1] && specifiedAmount > 10 ** 8);

            deal(address(this), type(uint256).max);

            (bool success,) = address(adapter).call{value: amountEth}("");

            eth_balance_before_swap = address(adapter).balance;
        } else {
            vm.assume(specifiedAmount < limits[0]);

            deal(address(this), specifiedAmount);
            eth_balance_before_swap = address(this).balance;
            (bool success,) = address(adapter).call{value: specifiedAmount}("");
        }

        uint256 bnt_balance_before_swap = IERC20(BNT).balanceOf(address(this));

        Trade memory trade = adapter.swap(PAIR, ETH, BNT, side, specifiedAmount);

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    IERC20(BNT).balanceOf(address(this))
                        - bnt_balance_before_swap
                );
                assertEq(
                    trade.calculatedAmount,
                    eth_balance_before_swap - address(adapter).balance
                );
            } else {
                assertEq(
                    specifiedAmount,
                    eth_balance_before_swap - address(this).balance
                );
                assertEq(
                    trade.calculatedAmount,
                    IERC20(BNT).balanceOf(address(this))
                        - bnt_balance_before_swap
                );
            }
        }
    }

    function testSwapSellIncreasingBancorV3() public {
        executeIncreasingSwaps(OrderSide.Sell);
    }

    function executeIncreasingSwaps(OrderSide side) internal {
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * (i + 1) * 10 ** 18;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(address(BNT), address(this), type(uint256).max);
            IERC20(BNT).approve(address(adapter), type(uint256).max);

            trades[i] = adapter.swap(PAIR, BNT, LINK, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    function testSwapBuyIncreasingBancorV3() public {
        executeIncreasingSwaps(OrderSide.Buy);
    }

    function testGetPoolIdsBancorV3() public view {
        bytes32[] memory ids = adapter.getPoolIds(1, 1000);
        assertGe(ids.length, 0);
    }

    function testGetLimitsBancorV3() public {
        uint256[] memory limits = adapter.getLimits(bytes32(0), BNT, LINK);
        assertEq(limits.length, 2);

        limits = adapter.getLimits(bytes32(0), ETH, BNT);
        assertEq(limits.length, 2);

        limits = adapter.getLimits(bytes32(0), LINK, ETH);
        assertEq(limits.length, 2);

        limits = adapter.getLimits(bytes32(0), ETH, LINK);
        assertEq(limits.length, 2);
    }

    function testPriceEqualPriceAfterSwapBancorV3(uint256 amountIn) public {
        uint256[] memory limits = adapter.getLimits(PAIR, LINK, WBTC);

        vm.assume(amountIn > 10 ** 14);
        vm.assume(amountIn < limits[0]);

        uint256[] memory amounts = new uint256[](1);

        amounts[0] = amountIn;

        Fraction[] memory prices = adapter.price(PAIR, LINK, WBTC, amounts);

        deal(address(LINK), address(this), amountIn);
        IERC20(LINK).approve(address(adapter), amountIn);

        Fraction memory priceSwap =
            adapter.swap(PAIR, LINK, WBTC, OrderSide.Sell, amountIn).price;

        assertEq(prices[0].numerator, priceSwap.numerator);
        assertEq(prices[0].denominator, priceSwap.denominator);
    }

    /**
     * @dev test fails with poolIds[0] =  bytes32(bytes20(address(ETH))) because
     * AdapterTest.sol
     * @dev doesn't handle selling eth
     * @dev And because allowance needs to be reset "using SafeERC20 for IERC20"
     */
    // function testPoolBehaviourBancorV3() public {
    //     bytes32[] memory poolIds = adapter.getPoolIds(0, 1000);
    //     runPoolBehaviourTest(adapter, poolIds);
    // }
}
