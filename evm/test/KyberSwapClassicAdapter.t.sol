// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import "src/kyberswap-classic/KyberSwapClassicAdapter.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";

contract KyberSwapClassicAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    KyberSwapClassicAdapter adapter;
    address constant WETH = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
    address constant WBTC = 0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599;
    address constant WBTC_WETH_PAIR = 0x1cf68Bbc2b6D3C6CfE1BD3590CF0E10b06a05F17;

    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 19689725;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new KyberSwapClassicAdapter(
            0x1c87257F5e8609940Bc751a07BB085Bb7f8cDBE6
        );

        vm.label(address(adapter), "KyberSwapClassicAdapter");
        vm.label(WETH, "WETH");
        vm.label(WBTC, "WBTC");
        vm.label(WBTC_WETH_PAIR, "WBTC_WETH_PAIR");
    }

    function testPriceFuzzKyberSwapWbtcForWeth(uint256 amount0, uint256 amount1)
        public
    {
        bytes32 pair = bytes32(bytes20(WBTC_WETH_PAIR));
        uint256[] memory limits = adapter.getLimits(pair, WBTC, WETH);
        vm.assume(amount0 > 0 && amount0 < limits[0]);
        vm.assume(amount1 > 0 && amount1 < limits[0]);

        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;

        Fraction[] memory prices = adapter.price(pair, WBTC, WETH, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testPriceFuzzKyberSwapWethForWbtc(uint256 amount0, uint256 amount1)
        public
    {
        bytes32 pair = bytes32(bytes20(WBTC_WETH_PAIR));
        uint256[] memory limits = adapter.getLimits(pair, WETH, WBTC);
        vm.assume(amount0 > 0 && amount0 < limits[0]);
        vm.assume(amount1 > 0 && amount1 < limits[0]);

        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;

        Fraction[] memory prices = adapter.price(pair, WETH, WBTC, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testPriceDecreasingKyberSwapWbtcForWeth() public {
        bytes32 pair = bytes32(bytes20(WBTC_WETH_PAIR));
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = i * 10 ** 2;
        }

        Fraction[] memory prices = adapter.price(pair, WBTC, WETH, amounts);

        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            assertEq(prices[i].compareFractions(prices[i + 1]), 1);
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    function testPriceDecreasingKyberSwapWethForWbtc() public {
        bytes32 pair = bytes32(bytes20(WBTC_WETH_PAIR));
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * i * 10 ** 6;
        }

        Fraction[] memory prices = adapter.price(pair, WETH, WBTC, amounts);

        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            assertEq(prices[i].compareFractions(prices[i + 1]), 1);
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    function testSwapFuzzKyberSwapWbtcForWeth(
        uint256 specifiedAmount,
        bool isBuy
    ) public {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(WBTC_WETH_PAIR));

        uint256[] memory limits = adapter.getLimits(pair, WBTC, WETH);

        Fraction[] memory priceBefore;

        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1]);
            vm.assume(specifiedAmount > 10 ** 12);

            deal(WBTC, address(this), type(uint256).max);
            IERC20(WBTC).approve(address(adapter), type(uint256).max);
        } else {
            vm.assume(specifiedAmount < limits[0]);
            vm.assume(specifiedAmount > 10 ** 2);

            deal(WBTC, address(this), specifiedAmount);
            IERC20(WBTC).approve(address(adapter), specifiedAmount);

            uint256[] memory specifiedAmounts = new uint256[](1);
            specifiedAmounts[0] = specifiedAmount;
            priceBefore = adapter.price(pair, WBTC, WETH, specifiedAmounts);
        }

        uint256 wbtc_balance = IERC20(WBTC).balanceOf(address(this));
        uint256 weth_balance = IERC20(WETH).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, WBTC, WETH, side, specifiedAmount);

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    IERC20(WETH).balanceOf(address(this)) - weth_balance
                );
                assertEq(
                    trade.calculatedAmount,
                    wbtc_balance - IERC20(WBTC).balanceOf(address(this))
                );
            } else {
                assertEq(
                    specifiedAmount,
                    wbtc_balance - IERC20(WBTC).balanceOf(address(this))
                );
                assertEq(
                    trade.calculatedAmount,
                    IERC20(WETH).balanceOf(address(this)) - weth_balance
                );
                assertEq(trade.price.compareFractions(priceBefore[0]), 0);
            }
        }
    }

    function testSwapFuzzKyberSwapWethForWbtc(
        uint256 specifiedAmount,
        bool isBuy
    ) public {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(WBTC_WETH_PAIR));
        uint256[] memory limits = adapter.getLimits(pair, WETH, WBTC);

        Fraction[] memory priceBefore;

        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1]);
            vm.assume(specifiedAmount > 10 ** 2);

            deal(WETH, address(this), type(uint256).max);
            IERC20(WETH).approve(address(adapter), type(uint256).max);
        } else {
            vm.assume(specifiedAmount < limits[0]);
            vm.assume(specifiedAmount > 10 ** 12);

            deal(WETH, address(this), specifiedAmount);
            IERC20(WETH).approve(address(adapter), specifiedAmount);

            uint256[] memory specifiedAmounts = new uint256[](1);
            specifiedAmounts[0] = specifiedAmount;
            priceBefore = adapter.price(pair, WETH, WBTC, specifiedAmounts);
        }

        uint256 weth_balance = IERC20(WETH).balanceOf(address(this));
        uint256 wbtc_balance = IERC20(WBTC).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, WETH, WBTC, side, specifiedAmount);

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    IERC20(WBTC).balanceOf(address(this)) - wbtc_balance
                );
                assertEq(
                    trade.calculatedAmount,
                    weth_balance - IERC20(WETH).balanceOf(address(this))
                );
            } else {
                assertEq(
                    specifiedAmount,
                    weth_balance - IERC20(WETH).balanceOf(address(this))
                );
                assertEq(
                    trade.calculatedAmount,
                    IERC20(WBTC).balanceOf(address(this)) - wbtc_balance
                );
                assertEq(trade.price.compareFractions(priceBefore[0]), 0);
            }
        }
    }

    function testSwapSellIncreasingKyberSwapWbtcForWeth() public {
        executeIncreasingSwapsKyberSwapWbtcForWeth(OrderSide.Sell);
    }

    function executeIncreasingSwapsKyberSwapWbtcForWeth(OrderSide side)
        internal
    {
        bytes32 pair = bytes32(bytes20(WBTC_WETH_PAIR));

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = side == OrderSide.Sell ? i * 10 ** 2 : i * 10 ** 12;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(WBTC, address(this), type(uint256).max);
            IERC20(WBTC).approve(address(adapter), type(uint256).max);

            trades[i] = adapter.swap(pair, WBTC, WETH, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    function testSwapBuyIncreasingKyberSwapWbtcForWeth() public {
        executeIncreasingSwapsKyberSwapWbtcForWeth(OrderSide.Buy);
    }

    function testSwapSellIncreasingKyberSwapWethForWbtc() public {
        executeIncreasingSwapsKyberSwapWethForWbtc(OrderSide.Sell);
    }

    function executeIncreasingSwapsKyberSwapWethForWbtc(OrderSide side)
        internal
    {
        bytes32 pair = bytes32(bytes20(WBTC_WETH_PAIR));

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = side == OrderSide.Sell ? i * 10 ** 12 : i * 10 ** 2;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(WETH, address(this), type(uint256).max);
            IERC20(WETH).approve(address(adapter), type(uint256).max);

            trades[i] = adapter.swap(pair, WETH, WBTC, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    function testSwapBuyIncreasingKyberSwapWethForWbtc() public {
        executeIncreasingSwapsKyberSwapWethForWbtc(OrderSide.Buy);
    }

    function testGetCapabilities(bytes32 pair, address t0, address t1) public {
        Capability[] memory res = adapter.getCapabilities(pair, t0, t1);

        assertEq(res.length, 3);
    }

    function testGetLimits() public {
        bytes32 pair = bytes32(bytes20(WBTC_WETH_PAIR));
        uint256[] memory limits = adapter.getLimits(pair, WBTC, WETH);

        assertEq(limits.length, 2);
    }
}
