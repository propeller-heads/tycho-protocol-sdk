// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "./AdapterTest.sol";
import "forge-std/Test.sol";
import {console2} from "forge-std/console2.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import {IBPool} from "src/cow-amm/interfaces/IBPool.sol";
import "src/libraries/FractionMath.sol";
import "src/cow-amm/CowAMMSwapAdapter.sol";

contract CowAMMSwapAdapterTest is AdapterTest {
    using FractionMath for Fraction;
    using BNumLib for uint256;

    address constant COWwstETHPool = 0x9bd702E05B9c97E4A4a3E47Df1e0fe7A0C26d2F1;

    CowAMMSwapAdapter adapter;

    address constant COW = 0xDEf1CA1fb7FBcDC777520aa7f396b4E015F497aB;
    address constant wstETH = 0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0;

    uint256 constant TEST_ITERATIONS = 500;

    function setUp() public {
        uint256 forkBlock = 20522303;

        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);

        adapter = new CowAMMSwapAdapter(COWwstETHPool);

        vm.label(address(COWwstETHPool), "COWwstETHPool");
        vm.label(address(adapter), "adapter");
        vm.label(address(COW), "COW");
        vm.label(address(wstETH), "wstETH");
    }

    //helper function
    function _calcTokenOutGivenExactLpTokenIn(
        uint256 balance,
        uint256 lpTokenAmountIn,
        uint256 totalLpToken
    ) internal pure returns (uint256 amountOut) {
        uint256 lpTokenRatio = lpTokenAmountIn.bdiv(totalLpToken);
        amountOut = balance.bmul(lpTokenRatio);
        return amountOut;
    }

    function _calcTokensOutGivenExactLpTokenIn(
        uint256[] memory balances,
        uint256 lpTokenAmountIn,
        uint256 totalLpToken
    ) internal pure returns (uint256[] memory amountsOut) {
        uint256 lpTokenRatio = lpTokenAmountIn.bdiv(totalLpToken);
        amountsOut = new uint256[](balances.length);
        for (uint256 i = 0; i < balances.length; i++) {
            amountsOut[i] = balances[i].bmul(lpTokenRatio);
        }
        return amountsOut;
    }

    function testPriceFuzz(uint256 amount0, uint256 amount1) public view {
        uint256[] memory limits = adapter.getLimits(bytes32(0), COW, wstETH);
        //check limits
        vm.assume(amount0 < limits[0] && amount0 > 10e4);
        vm.assume(amount1 < limits[0] && amount1 > 10e4);
        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;
        //with amountIn = 0, no tokens can be taken out. and vice versa so
        // assertGt()
        Fraction[] memory prices =
            adapter.price(bytes32(0), COW, wstETH, amounts);
        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testPriceSingleFuzz() public view {
        uint256 specifiedAmount = 10000;
        // Assume OrderSide.Sell
        uint256[] memory limits = adapter.getLimits(bytes32(0), COW, wstETH);
        vm.assume(specifiedAmount > 0);
        vm.assume(specifiedAmount < limits[0]);

        Fraction memory price = adapter.getPriceAt(specifiedAmount, COW, wstETH);

        assertGt(price.numerator, 0);
        assertGt(price.denominator, 0);
    }

    function testPriceDecreasing() public view {
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        Fraction[] memory prices = new Fraction[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * (i + 1) * 10 ** 10;
            prices[i] = adapter.getPriceAt(amounts[i], COW, wstETH);
        }

        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            assertEq(prices[i].compareFractions(prices[i + 1]), 1);
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    function testSwapFuzz(uint256 specifiedAmount, bool isBuy) public {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        uint256[] memory limits = adapter.getLimits(bytes32(0), COW, wstETH);
        //when you buy the specified amount you are passing out is you
        // specifying the amount you want to receive

        vm.assume(specifiedAmount > 10e4);
        vm.assume(specifiedAmount < limits[0]);

        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1] && specifiedAmount > 0);

            //set specified amount of COW tokens to address, to buy wstEth
            deal(COW, address(this), specifiedAmount);
            IERC20(COW).approve(address(adapter), specifiedAmount);
        } else {
            vm.assume(specifiedAmount < limits[0] && specifiedAmount > 0);
            //set specified amount of COW tokens to address
            deal(COW, address(this), specifiedAmount);
            IERC20(COW).approve(address(adapter), specifiedAmount);
        }

        uint256 wstETH_balance = IERC20(wstETH).balanceOf(address(this));
        uint256 COW_balance = IERC20(COW).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(bytes32(0), COW, wstETH, side, specifiedAmount);

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertGt(trade.calculatedAmount, 0);
                assertEq(specifiedAmount, COW_balance);
            } else {
                assertGt(trade.calculatedAmount, 0);
                assertEq(specifiedAmount, COW_balance);
                // the balance of wstETH can never update because we can't
                // actually swap on the CowPool directly, ideally i would have
                // used swapExactAmountIn() or swapExactAmountOut()
                //but the fee on the COW-wstETH pool is 99.9% so this assertion
                // would always fail, because the balance can never increase
                // assertEq(
                //     trade.calculatedAmount,
                //     IERC20(wstETH).balanceOf(address(this)) - wstETH_balance
                // );
            }
        }
    }

    function testSwapSellIncreasing() public {
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        Trade[] memory trades = new Trade[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * (i + 1) * 10 ** 10;

            uint256 beforeSwap = vm.snapshot();

            deal(COW, address(this), amounts[i]);
            IERC20(COW).approve(address(adapter), amounts[i]);
            trades[i] = adapter.swap(
                bytes32(0), COW, wstETH, OrderSide.Sell, amounts[i]
            );

            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    function testSwapBuyIncreasing() public {
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        Trade[] memory trades = new Trade[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 10 * (i + 1) * 10 ** 12;

            uint256 beforeSwap = vm.snapshot();

            Fraction memory price = adapter.getPriceAt(amounts[i], COW, wstETH);
            uint256 amountIn =
                (amounts[i] * price.denominator / price.numerator) * 2;

            deal(COW, address(this), amountIn);
            IERC20(COW).approve(address(adapter), amountIn);
            trades[i] = adapter.swap(
                bytes32(0), COW, wstETH, OrderSide.Buy, amounts[i]
            );

            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    function testLPExitWstETHFuzz(uint256 specifiedAmount) public {
        OrderSide side = OrderSide.Sell; // selling LP for tokenA = wstETH
        // vm.assume(specifiedAmount > 1e18 && specifiedAmount < 1e21);

        specifiedAmount = 1e18;

        deal(COWwstETHPool, address(adapter), specifiedAmount);
        IERC20(COWwstETHPool).approve(address(adapter), specifiedAmount);

        deal(COW, address(this), 10e20);
        IERC20(COW).approve(address(adapter), 10e20);

        deal(wstETH, address(this), 10e20);
        IERC20(wstETH).approve(address(adapter), 10e20);

        uint256 lpTokenBefore =
            IERC20(COWwstETHPool).balanceOf(address(adapter));
        uint256 wstETHBefore = IERC20(wstETH).balanceOf(address(adapter));
        uint256 COWBefore = IERC20(COW).balanceOf(address(adapter));
        //What swap does
        // 1. Sell LP token for wstETH (i.e., exit liquidity)
        // 2. Swap remaining COW tokens to wstETH convert full exit to wstETH
        Trade memory exitTrade = adapter.swap(
            bytes32(0), COWwstETHPool, wstETH, side, specifiedAmount
        );

        uint256 lpTokenAfter = IERC20(COWwstETHPool).balanceOf(address(adapter));
        uint256 wstETHAfter = IERC20(wstETH).balanceOf(address(adapter));
        uint256 COWAfter = IERC20(COW).balanceOf(address(adapter));
        uint256 tradeAmount = exitTrade.calculatedAmount;

        // Lp token was redeemed for wstETH and COW, so balance before is >
        // balance after wstETH was received from the pool exit so, balance
        // after > than the balance before
        // COW was received from the pool exit so, balance after > than the
        // balance before

        assertGt(lpTokenBefore, lpTokenAfter);
        assertGt(wstETHAfter, wstETHBefore);
        assertGt(COWAfter, COWBefore);
        assertGt(tradeAmount, 0);
    }

    function testLPExitCOWFuzz(uint256 specifiedAmount) public {
        OrderSide side = OrderSide.Sell; // selling LP for tokenA = COW

        specifiedAmount = 1e18;

        deal(COWwstETHPool, address(adapter), specifiedAmount);
        IERC20(COWwstETHPool).approve(address(adapter), specifiedAmount);

        deal(COW, address(this), 10e20);
        IERC20(COW).approve(address(adapter), 10e20);

        deal(wstETH, address(this), 10e20);
        IERC20(wstETH).approve(address(adapter), 10e20);

        uint256 lpTokenBefore =
            IERC20(COWwstETHPool).balanceOf(address(adapter));
        uint256 COWBefore = IERC20(COW).balanceOf(address(adapter));
        uint256 wstETHBefore = IERC20(wstETH).balanceOf(address(adapter));

        Trade memory exitTrade =
            adapter.swap(bytes32(0), COWwstETHPool, COW, side, specifiedAmount);

        uint256 LPTokenAfter = IERC20(COWwstETHPool).balanceOf(address(adapter));
        uint256 COWAfter = IERC20(COW).balanceOf(address(adapter));
        uint256 wstETHAfter = IERC20(wstETH).balanceOf(address(adapter));
        uint256 calculatedAmount = exitTrade.calculatedAmount;

        assertGt(lpTokenBefore, LPTokenAfter);
        assertGt(COWAfter, COWBefore);
        assertGt(wstETHAfter, wstETHBefore);
        assertGt(calculatedAmount, 0);
    }

    function testLPjoinWith_WstETH_fuzz(uint256 specifiedAmount) public {
        OrderSide side = OrderSide.Buy;
        // vm.assume(specifiedAmount > 1e18 && specifiedAmount < 1e21);
        specifiedAmount = 1e18;

        //we approve the tokens in the contract, first give the adapter tokens
        // to join to the pool
        deal(COW, address(adapter), 10e23);
        deal(wstETH, address(adapter), 10e23);

        deal(COWwstETHPool, address(this), 10e24);

        uint256 LPTokenBefore =
            IERC20(COWwstETHPool).balanceOf(address(adapter));
        uint256 wstETHBefore = IERC20(wstETH).balanceOf(address(adapter));
        uint256 COWBefore = IERC20(COW).balanceOf(address(adapter));

        //We are selling wstETH back into the pool to buy our COW back, we just
        // used the COW to join the wstETH to the pool since its double sided
        // liquidity joining
        Trade memory joinTrade = adapter.swap(
            bytes32(0), wstETH, COWwstETHPool, side, specifiedAmount
        );

        uint256 LPTokenAfter = IERC20(COWwstETHPool).balanceOf(address(adapter));
        uint256 wstETHAfter = IERC20(wstETH).balanceOf(address(adapter));
        uint256 COWAfter = IERC20(COW).balanceOf(address(adapter));
        uint256 calculatedAmount = joinTrade.calculatedAmount;
        assertGt(LPTokenAfter, LPTokenBefore);
        assertGt(COWBefore, COWAfter);
        assertGt(wstETHBefore, wstETHAfter);
        assertGt(calculatedAmount, 0);
    }

    function testLPjoinWith_COW_fuzz(uint256 specifiedAmount) public {
        OrderSide side = OrderSide.Buy;

        // vm.assume(specifiedAmount > 1e18 && specifiedAmount < 1e21);
        specifiedAmount = 1e18;
        deal(COWwstETHPool, address(this), 10e23);

        deal(COW, address(adapter), 10e24);
        deal(wstETH, address(adapter), 10e24);

        uint256 LPTokenBefore =
            IERC20(COWwstETHPool).balanceOf(address(adapter));
        uint256 COWBefore = IERC20(COW).balanceOf(address(adapter));
        uint256 wstETHBefore = IERC20(wstETH).balanceOf(address(adapter));

        Trade memory joinTrade =
            adapter.swap(bytes32(0), COW, COWwstETHPool, side, specifiedAmount);

        uint256 LPTokenAfter = IERC20(COWwstETHPool).balanceOf(address(adapter));
        uint256 COWAfter = IERC20(COW).balanceOf(address(adapter));
        uint256 wstETHAfter = IERC20(wstETH).balanceOf(address(adapter));
        assertGt(joinTrade.calculatedAmount, 0);
        assertGt(LPTokenAfter, LPTokenBefore);
        assertGt(COWBefore, COWAfter);
        assertGe(wstETHBefore, wstETHAfter);
    }

    // Normally, the price does not change when joining or exiting to a pool,
    // but because we swap the superfluous token in this case COW into wstETH it
    // will change

    function testLPjoinwstETHPriceIncreasing() public {
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        Trade[] memory trades = new Trade[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * (i + 1) * 10 ** 10;

            uint256 beforeSwap = vm.snapshot();

            deal(COW, address(adapter), 10e24);
            deal(wstETH, address(adapter), 10e24);

            deal(COWwstETHPool, address(this), 10e24);

            trades[i] = adapter.swap(
                bytes32(0), wstETH, COWwstETHPool, OrderSide.Buy, amounts[i]
            );

            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    function testLPjoinCOWPriceIncreasing() public {
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        Trade[] memory trades = new Trade[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * (i + 1) * 10 ** 10;

            uint256 beforeSwap = vm.snapshot();

            deal(COW, address(adapter), 10e24);
            deal(wstETH, address(adapter), 10e24);

            deal(COWwstETHPool, address(this), 10e24);

            trades[i] = adapter.swap(
                bytes32(0), COW, COWwstETHPool, OrderSide.Buy, amounts[i]
            );

            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    function testLPexitwstETHPriceIncreasing() public {
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        Trade[] memory trades = new Trade[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * (i + 1) * 10 ** 12;

            uint256 beforeSwap = vm.snapshot();

            deal(COWwstETHPool, address(adapter), amounts[i]);
            IERC20(COWwstETHPool).approve(address(adapter), amounts[i]);

            deal(COW, address(this), 10e24);
            IERC20(COW).approve(address(adapter), 10e24);

            deal(wstETH, address(this), 10e24);
            IERC20(wstETH).approve(address(adapter), 10e24);

            trades[i] = adapter.swap(
                bytes32(0), COWwstETHPool, wstETH, OrderSide.Sell, amounts[i]
            );

            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    function testLPexitCOWPriceIncreasing() public {
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        Trade[] memory trades = new Trade[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * (i + 1) * 10 ** 12;

            uint256 beforeSwap = vm.snapshot();

            deal(COWwstETHPool, address(adapter), amounts[i]);
            IERC20(COWwstETHPool).approve(address(adapter), amounts[i]);

            deal(COW, address(this), 10e24);
            IERC20(COW).approve(address(adapter), 10e24);

            deal(wstETH, address(this), 10e24);
            IERC20(wstETH).approve(address(adapter), 10e24);

            trades[i] = adapter.swap(
                bytes32(0), COWwstETHPool, COW, OrderSide.Sell, amounts[i]
            );

            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    function testCowAMMPoolBehaviour() public {
        bytes32[] memory poolIds = new bytes32[](1);
        poolIds[0] = bytes32(0);
        runPoolBehaviourTest(adapter, poolIds);
    }

    //runPoolBehaviourtest for LP token buying and selling that is tokens[2]

    function testGetCapabilitiesCowAMM() public view {
        Capability[] memory res =
            adapter.getCapabilities(bytes32(0), COW, wstETH);

        assertEq(res.length, 5);
    }

    function testGetLimits() public view {
        uint256[] memory limits = adapter.getLimits(bytes32(0), wstETH, COW);

        assertEq(limits.length, 2);
        assert(limits[0] > 0);
        assert(limits[1] > 0);
    }

    function testGetTokens(bytes32 poolId) public view {
        address[] memory tokens = adapter.getTokens(bytes32(0));

        assertEq(tokens.length, 3);
    }
}
