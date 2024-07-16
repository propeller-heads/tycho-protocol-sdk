// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";

contract TestAdapter is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    uint256 constant pricePrecision = 10e24;

    // @notice Test the behavior of a swap adapter for a list of pools
    // @dev Computes limits, prices, and swaps on the pools on both directions
    // for different
    // sell amounts. Asserts that the prices behaves as expected.
    // @param adapter The swap adapter to test
    // @param poolIds The list of pool ids to test
    function testPoolBehaviour(
        ISwapAdapter adapter,
        bytes32[] memory poolIds
    ) public {
        for (uint256 i = 0; i < poolIds.length; i++) {
            address[] memory tokens = adapter.getTokens(poolIds[i]);
            IERC20(tokens[0]).approve(address(adapter), type(uint256).max);

            string[] memory pctgs = new string[](4);
            pctgs[0] = "0%";
            pctgs[1] = "0.01%";
            pctgs[2] = "50%";
            pctgs[3] = "100%";

            console2.log(
                "TEST: Testing prices at 0, 0.01%, 50% and 100% of the sell limit"
            );

            testPricesForPair(adapter, poolIds[i], tokens[0], tokens[1], pctgs);
            testPricesForPair(adapter, poolIds[i], tokens[1], tokens[0], pctgs);
        }
    }

    // Prices should:
    // 1. Be monotonic decreasing
    // 2. Be positive
    // 3. Always be >= the executed price and >= the price after the swap
    function testPricesForPair(
        ISwapAdapter adapter,
        bytes32 poolId,
        address tokenIn,
        address tokenOut,
        string[] memory pctgs
    ) internal {
        uint256[] memory amounts =
            calculateAmounts(adapter.getLimits(poolId, tokenIn, tokenOut)[0]);
        Fraction[] memory prices =
            adapter.price(poolId, tokenIn, tokenOut, amounts);
        assertGt(
            getPrice(prices[0]),
            getPrice(prices[1]),
            "Price at limit should be smaller than price at 0"
        );

        console2.log("TEST: Testing behavior for price at 0.");
        assertGt(prices[0].numerator, 0, "Nominator shouldn't be 0");
        assertGt(prices[0].denominator, 0, "Denominator shouldn't be 0");

        Trade memory trade;
        deal(tokenIn, address(this), 2 * amounts[amounts.length - 1]);

        for (uint256 j = 1; j < amounts.length; j++) {
            console2.log(
                "TEST: Testing behavior for price at %s limit: %d",
                pctgs[j],
                amounts[j]
            );
            uint256 priceAtAmount = getPrice(prices[j]);
            assertGt(prices[j].numerator, 0, "Nominator shouldn't be 0");
            assertGt(prices[j].denominator, 0, "Denominator shouldn't be 0");

            console2.log("TEST: Swapping %d of %s", amounts[j], tokenIn);
            trade = adapter.swap(
                poolId, tokenIn, tokenIn, OrderSide.Sell, amounts[j]
            );
            uint256 executedPrice =
                trade.calculatedAmount * pricePrecision / amounts[j];
            uint256 priceAfterSwap = getPrice(trade.price);
            console2.log("TEST: Pool price:       %d", priceAtAmount);
            console2.log("TEST: Executed price:   %d", executedPrice);
            console2.log("TEST: Price after swap: %d", priceAfterSwap);

            assertGt(
                priceAtAmount,
                executedPrice,
                "Price should be greated than executed price."
            );
            assertGt(
                executedPrice,
                priceAfterSwap,
                "Executed price should be greater than price after swap."
            );
            assertGt(
                priceAtAmount,
                priceAfterSwap,
                "Price should be greated than price after swap."
            );
        }
    }

    function calculateAmounts(uint256 limit)
        internal
        pure
        returns (uint256[] memory)
    {
        uint256[] memory amounts = new uint256[](4);
        amounts[0] = 0;
        amounts[1] = limit / 10000;
        amounts[2] = limit / 2;
        amounts[3] = limit;
        return amounts;
    }

    function getPrice(Fraction memory price) public pure returns (uint256) {
        return price.numerator * pricePrecision / price.denominator;
    }
}
