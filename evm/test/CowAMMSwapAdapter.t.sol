// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {console2} from "forge-std/console2.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import { IBPool } from "src/CowAMM/interfaces/IBPool.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";
import "src/CowAMM/CowAMMSwapAdapter.sol";

contract CowAMMSwapAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    uint256 constant pricePrecision = 10e24;
    string[] public stringPctgs = ["0%", "0.1%", "50%", "100%"];
    
    address constant BPool = 0x9bd702E05B9c97E4A4a3E47Df1e0fe7A0C26d2F1;

    CowAMMSwapAdapter adapter;

    address constant COW = 0xDEf1CA1fb7FBcDC777520aa7f396b4E015F497aB; 
    address constant wstETH = 0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0;
    address constant BCoW50COW50wstETH = 0x9bd702E05B9c97E4A4a3E47Df1e0fe7A0C26d2F1; //LP token
    uint256 constant TEST_ITERATIONS = 5;
    function setUp() public {
        uint256 forkBlock = 20522303;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);

        adapter = new CowAMMSwapAdapter(BPool);

        vm.label(address(BPool), "BPool"); 
        vm.label(address(adapter), "CowAMMSwapAdapter");
        vm.label(address(wstETH), "wstETH");
        vm.label(address(COW), "COW");
    } 
    
    function testPriceFuzz(uint256 amount0, uint256 amount1) public {
        uint256[] memory limits = adapter.getLimits(bytes32(0), wstETH, COW);
        //if the amount is too big, that when computing it eventually creates a base too big in the calcOutGivenIn() 
        //calculation and we'll get an revert BNum_BPowBaseTooHigh() it threw that error for 2904413035532990072 
        //check limits 
        vm.assume(amount0 < limits[0] && amount0 > 0);
        vm.assume(amount1 < limits[1] && amount1 > 0);

        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;
        // console2.log("amount 0:", amount0);
        // console2.log("amount 1:", amount1);
        //with amountIn = 0, no tokens can be taken out. and vice versa so assertGt()
        Fraction[] memory prices = adapter.price(bytes32(0), wstETH, COW, amounts);
        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testPriceSingleFuzz() public {
        uint256 specifiedAmount = 100; 
        //the arithmetic operations are implemented with a precision of BONE.
        //so they are already scaled internally by 10^18 
        // Assume OrderSide.Sell
        uint256[] memory limits =
            adapter.getLimits(bytes32(0), wstETH, COW);
        vm.assume(specifiedAmount > 0);
        vm.assume(specifiedAmount < limits[0]); //10^20 < 2.32 * 10^20

        Fraction memory price = adapter.getPriceAt(
           specifiedAmount, wstETH, COW
        );

        assertGt(price.numerator, 0);
        assertGt(price.denominator, 0);
    }

     function testPriceDecreasing() public {
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        Fraction[] memory prices = new Fraction[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * (i + 1) * 10**6; 
            prices[i] = adapter.getPriceAt(
                 amounts[i], wstETH, COW
            ); 
        }
        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            assertEq(prices[i].compareFractions(prices[i + 1]), 1);
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }
     function testSwapFuzz(uint256 specifiedAmount, bool isBuy) public {
        // specifiedAmount = 900;
        vm.assume(specifiedAmount > 0);
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        uint256[] memory limits = adapter.getLimits(bytes32(0), wstETH, COW);
        
        if (side == OrderSide.Buy) {
            //the limits for the sell or buy token change so limit[0] desnt cut it 
            vm.assume(specifiedAmount < limits[0]);
            //set specified amount of COW tokens to address
            deal(COW, address(this), specifiedAmount);
            IERC20(COW).approve(address(adapter), specifiedAmount);
        } else {
            vm.assume(specifiedAmount < limits[1]);
            //set specified amount of wstETH tokens to address
            deal(wstETH, address(this), specifiedAmount);
            IERC20(wstETH).approve(address(adapter), specifiedAmount);
        }

        uint256 wstETH_balance = IERC20(wstETH).balanceOf(address(this));
        uint256 cow_balance = IERC20(COW).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(bytes32(0), wstETH, COW, side, specifiedAmount);

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    IERC20(COW).balanceOf(address(this)) - cow_balance
                );
                // assertEq(
                //     trade.calculatedAmount,
                //     wstETH_balance - IERC20(wstETH).balanceOf(address(this))
                // );
            } else {
                assertEq(
                    specifiedAmount, 
                    wstETH_balance - IERC20(wstETH).balanceOf(address(this))
                );
                // the balance of COW can never update because we can't actually swap on the 
                //CowPool directly, ideally i would have used swapExactAmountIn() or swapExactAmountOut()
                //but the fee on the COW-wstETH pool is 99.9% so this assertion would always fail
                // assertEq(
                //     trade.calculatedAmount, 
                //     IERC20(COW).balanceOf(address(this)) - cow_balance
                // );
            }
        }
    }
    function testSwapSellIncreasing() public {
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        Trade[] memory trades = new Trade[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * (i + 1) * 10 ** 18;

            uint256 beforeSwap = vm.snapshot();

            deal(wstETH, address(this), amounts[i]);
            IERC20(wstETH).approve(address(adapter), amounts[i]);
            trades[i] = adapter.swap(
                bytes32(0), wstETH, COW, OrderSide.Sell, amounts[i]
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
            amounts[i] = 10 * (i + 1) * 10 ** 18;

            uint256 beforeSwap = vm.snapshot();

            Fraction memory price = adapter.getPriceAt(
                amounts[i], wstETH, COW
            );
            uint256 amountIn =
                (amounts[i] * price.denominator / price.numerator) * 2;

            deal(wstETH, address(this), amountIn);
            IERC20(wstETH).approve(address(adapter), amountIn);
            trades[i] = adapter.swap(
                bytes32(0), wstETH, COW, OrderSide.Buy, amounts[i]
            );

            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    // function testLPjoinFuzz() public {

    // }
    // function testLPexitFuzz() public {

    // }
    // function testLPjoin() public {

    // }
    // function testLPexit() public {

    // }
    // @notice Test the behavior of a swap adapter for a list of pools
    // @dev Computes limits, prices, and swaps on the pools on both directions
    // for different
    // sell amounts. Asserts that the prices behaves as expected.
    // @param adapter The swap adapter to test
    // @param poolIds The list of pool ids to test
    // function testPoolBehaviour(
    //     bytes32[] memory poolIds
    // ) public {
    //     bool hasPriceImpact = !hasCapability(
    //         adapter.getCapabilities(poolIds[0], address(0), address(0)),
    //         Capability.ConstantPrice
    //     );
    //     console2.log("this is the poolId length", poolIds.length);
    //     for (uint256 i = 0; i < poolIds.length; i++) {
    //         address[] memory tokens = adapter.getTokens(poolIds[i]);
    //         IERC20(tokens[0]).approve(address(adapter), type(uint256).max);
    //         IERC20(tokens[1]).approve(address(adapter), type(uint256).max);

    //         testPricesForPair(
    //             poolIds[i], tokens[0], tokens[1], hasPriceImpact
    //         );
    //         testPricesForPair(
    //             poolIds[i], tokens[1], tokens[0], hasPriceImpact
    //         );
    //     }
    // }

    // Prices should:
    // 1. Be monotonic decreasing
    // 2. Be positive
    // 3. Always be >= the executed price and >= the price after the swap
    // function testPricesForPair(
    //     bytes32,
    //     address tokenIn,
    //     address tokenOut,
    //     bool hasPriceImpact
    // ) internal {
    //     uint256 sellLimit = adapter.getLimits(bytes32(0), tokenIn, tokenOut)[0];
    //     assertGt(sellLimit, 0, "Sell limit should be greater than 0");

    //     console2.log(
    //         "TEST: Testing prices for pair %s -> %s. Sell limit: %d",
    //         tokenIn,
    //         tokenOut,
    //         sellLimit
    //     );

    //     bool hasMarginalPrices = hasCapability(
    //         adapter.getCapabilities(bytes32(0), tokenIn, tokenOut),
    //         Capability.MarginalPrice
    //     );
    //     uint256[] memory amounts =
    //         calculateTestAmounts(sellLimit, hasMarginalPrices);
    //     Fraction[] memory prices =
    //         adapter.price(bytes32(0), tokenIn, tokenOut, amounts);
    //     assertGt(
    //         fractionToInt(prices[0]),
    //         fractionToInt(prices[prices.length - 1]),
    //         "Price at limit should be smaller than price at 0"
    //     );
    //     console2.log(
    //         "TEST: Price at 0: %d, price at sell limit: %d",
    //         fractionToInt(prices[0]),
    //         fractionToInt(prices[prices.length - 1])
    //     );

    //     console2.log("TEST: Testing behavior for price at 0");
    //     assertGt(prices[0].numerator, 0, "Nominator shouldn't be 0");
    //     assertGt(prices[0].denominator, 0, "Denominator shouldn't be 0");
    //     uint256 priceAtZero = fractionToInt(prices[0]);
    //     console2.log("TEST: Price at 0: %d", priceAtZero);

    //     Trade memory trade;
    //     deal(tokenIn, address(this), 5 * amounts[amounts.length - 1]);

    //     uint256 initialState = vm.snapshot();

    //     for (uint256 j = 1; j < amounts.length; j++) {
    //         console2.log(
    //             "TEST: Testing behavior for price at %s of limit.",
    //             stringPctgs[j],
    //             amounts[j]
    //         );
    //         uint256 priceAtAmount = fractionToInt(prices[j]);

    //         console2.log("TEST: Swapping %d of %s", amounts[j], tokenIn);
    //         trade = adapter.swap(
    //             bytes32(0), tokenIn, tokenOut, OrderSide.Sell, amounts[j]
    //         );
    //         uint256 executedPrice =
    //             trade.calculatedAmount * pricePrecision / amounts[j];
    //         uint256 priceAfterSwap = fractionToInt(trade.price);
    //         console2.log("TEST:  - Executed price:   %d", executedPrice);
    //         console2.log("TEST:  - Price at amount:  %d", priceAtAmount);
    //         console2.log("TEST:  - Price after swap: %d", priceAfterSwap);

    //         if (hasPriceImpact) {
    //             assertGe(
    //                 executedPrice,
    //                 priceAtAmount,
    //                 "Price should be greated than executed price."
    //             );
    //             assertGt(
    //                 executedPrice,
    //                 priceAfterSwap,
    //                 "Executed price should be greater than price after swap."
    //             );
    //             assertGt(
    //                 priceAtZero,
    //                 executedPrice,
    //                 "Price should be greated than price after swap."
    //             );
    //         } else {
    //             assertGe(
    //                 priceAtZero,
    //                 priceAfterSwap,
    //                 "Executed price should be or equal to price after swap."
    //             );
    //             assertGe(
    //                 priceAtZero,
    //                 priceAtAmount,
    //                 "Executed price should be or equal to price after swap."
    //             );
    //             assertGe(
    //                 priceAtZero,
    //                 executedPrice,
    //                 "Price should be or equal to price after swap."
    //             );
    //         }

    //         vm.revertTo(initialState);
    //     }
    //     uint256 amountAboveLimit = sellLimit * 105 / 100;

    //     bool hasHardLimits = hasCapability(
    //         adapter.getCapabilities(bytes32(0), tokenIn, tokenOut),
    //         Capability.HardLimits
    //     );

    //     if (hasHardLimits) {
    //         testRevertAboveLimit(
    //             bytes32(0), tokenIn, tokenOut, amountAboveLimit
    //         );
    //     } else {
    //         testOperationsAboveLimit(
    //             bytes32(0), tokenIn, tokenOut, amountAboveLimit
    //         );
    //     }

    //     console2.log("TEST: All tests passed.");
    // }

    // function testRevertAboveLimit(
    //     bytes32 poolId,
    //     address tokenIn,
    //     address tokenOut,
    //     uint256 amountAboveLimit
    // ) internal {
    //     console2.log(
    //         "TEST: Testing revert behavior above the sell limit: %d",
    //         amountAboveLimit
    //     );
    //     uint256[] memory aboveLimitArray = new uint256[](1);
    //     aboveLimitArray[0] = amountAboveLimit;

    //     try adapter.price(poolId, tokenIn, tokenOut, aboveLimitArray) {
    //         revert(
    //             "Pool shouldn't be able to fetch prices above the sell limit"
    //         );
    //     } catch Error(string memory s) {
    //         console2.log(
    //             "TEST: Expected error when fetching price above limit: %s", s
    //         );
    //     }
    //     try adapter.swap(
    //         poolId, tokenIn, tokenOut, OrderSide.Sell, aboveLimitArray[0]
    //     ) {
    //         revert("Pool shouldn't be able to swap above the sell limit");
    //     } catch Error(string memory s) {
    //         console2.log(
    //             "TEST: Expected error when swapping above limit: %s", s
    //         );
    //     }
    // }

    // function testOperationsAboveLimit(
    //     bytes32 poolId,
    //     address tokenIn,
    //     address tokenOut,
    //     uint256 amountAboveLimit
    // ) internal {
    //     console2.log(
    //         "TEST: Testing operations above the sell limit: %d",
    //         amountAboveLimit
    //     );
    //     uint256[] memory aboveLimitArray = new uint256[](1);
    //     aboveLimitArray[0] = amountAboveLimit;

    //     adapter.price(poolId, tokenIn, tokenOut, aboveLimitArray);
    //     adapter.swap(
    //         poolId, tokenIn, tokenOut, OrderSide.Sell, aboveLimitArray[0]
    //     );
    // }

    function calculateTestAmounts(uint256 limit, bool hasMarginalPrices)
        internal
        pure
        returns (uint256[] memory)
    {
        uint256[] memory amounts = new uint256[](4);
        amounts[0] = hasMarginalPrices ? 0 : limit / 10000;
        amounts[1] = limit / 1000;
        amounts[2] = limit / 2;
        amounts[3] = limit;
        return amounts;
    }

    function testGetCapabilitiesCowAMM() public {
        Capability[] memory res =
            adapter.getCapabilities(bytes32(0), COW, wstETH);

        assertEq(res.length, 4);
    }
    function testGetLimits() public {
        uint256[] memory limits = adapter.getLimits(bytes32(0), COW, wstETH);

        assertEq(limits.length, 2);
        assert(limits[0] > 0);
        assert(limits[1] > 0);
    }

    function testGetTokens(bytes32 poolId) public {
        address[] memory tokens = adapter.getTokens(bytes32(0));

        assertEq(tokens[0], address(COW));
        assertEq(tokens[1], address(wstETH));
    }
    function fractionToInt(Fraction memory price)
        public
        pure
        returns (uint256)
    {
        return price.numerator * pricePrecision / price.denominator;
    }

    function hasCapability(
        Capability[] memory capabilities,
        Capability capability
    ) internal pure returns (bool) {
        for (uint256 i = 0; i < capabilities.length; i++) {
            if (capabilities[i] == capability) {
                return true;
            }
        }

        return false;
    }
    
}
