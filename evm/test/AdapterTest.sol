// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";
import "src/libraries/EfficientERC20.sol";

contract AdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;
    using EfficientERC20 for IERC20;

    string[] public stringPctgs = ["0%", "0.1%", "50%", "100%"];

    // @notice Test the behavior of a swap adapter for a list of pools
    // @dev Computes limits, prices, and swaps on the pools on both directions
    // for different
    // sell amounts. Asserts that the prices behaves as expected.
    // @param adapter The swap adapter to test
    // @param poolIds The list of pool ids to test
    function runPoolBehaviourTest(
        ISwapAdapter adapter,
        bytes32[] memory poolIds
    ) public {
        bool hasPriceImpact = !hasCapability(
            adapter.getCapabilities(poolIds[0], address(0), address(0)),
            Capability.ConstantPrice
        );
        for (uint256 i = 0; i < poolIds.length; i++) {
            address[] memory tokens = adapter.getTokens(poolIds[i]);
            IERC20(tokens[0]).forceApprove(address(adapter), type(uint256).max);
            IERC20(tokens[1]).forceApprove(address(adapter), type(uint256).max);

            testPricesForPair(
                adapter, poolIds[i], tokens[0], tokens[1], hasPriceImpact
            );
            testPricesForPair(
                adapter, poolIds[i], tokens[1], tokens[0], hasPriceImpact
            );
        }
    }

    // Prices should:
    // 1. Be monotonic decreasing (within rounding tolerance)
    // 2. Be positive
    // 3. Always be >= the executed price and >= the price after the swap
    // (within rounding tolerance)
    function testPricesForPair(
        ISwapAdapter adapter,
        bytes32 poolId,
        address tokenIn,
        address tokenOut,
        bool hasPriceImpact
    ) internal {
        uint256 sellLimit = adapter.getLimits(poolId, tokenIn, tokenOut)[0];
        assertGt(sellLimit, 0, "Sell limit should be greater than 0");

        console2.log(
            "TEST: Testing prices for pair %s -> %s. Sell limit: %d",
            tokenIn,
            tokenOut,
            sellLimit
        );

        bool hasMarginalPrices = hasCapability(
            adapter.getCapabilities(poolId, tokenIn, tokenOut),
            Capability.MarginalPrice
        );
        uint256[] memory amounts =
            calculateTestAmounts(sellLimit, hasMarginalPrices);

        // TODO: What if the price function is not available? Do we still want
        // to run this test?
        Fraction[] memory prices =
            adapter.price(poolId, tokenIn, tokenOut, amounts);
        assertGt(
            fractionToInt(prices[0])
                // within rounding tolerance
                * (amounts[amounts.length - 1] + 1)
                / amounts[amounts.length - 1],
            fractionToInt(prices[prices.length - 1]),
            "Price at limit should be smaller than price at 0"
        );
        console2.log(
            "TEST: Price at 0: %d, price at sell limit: %d",
            fractionToInt(prices[0]),
            fractionToInt(prices[prices.length - 1])
        );

        console2.log("TEST: Testing behavior for price at 0");
        assertGt(prices[0].numerator, 0, "Nominator shouldn't be 0");
        assertGt(prices[0].denominator, 0, "Denominator shouldn't be 0");
        uint256 priceAtZero = fractionToInt(prices[0]);
        console2.log("TEST: Price at 0: %d", priceAtZero);

        deal(tokenIn, address(this), 5 * amounts[amounts.length - 1]);

        uint256 initialState = vm.snapshot();

        for (uint256 j = 1; j < amounts.length; j++) {
            console2.log(
                "TEST: Testing behavior for price at %s of limit.",
                stringPctgs[j],
                amounts[j]
            );
            uint256 priceAtAmount = fractionToInt(prices[j]);

            console2.log("TEST: Swapping %d of %s", amounts[j], tokenIn);
            try adapter.swap(
                poolId, tokenIn, tokenOut, OrderSide.Sell, amounts[j]
            ) returns (
                Trade memory trade
            ) {
                uint256 executedPrice = Fraction(
                        trade.calculatedAmount, amounts[j]
                    ).toQ128x128();
                uint256 priceAfterSwap = fractionToInt(trade.price);
                console2.log("TEST:  - Executed price:   %d", executedPrice);
                console2.log("TEST:  - Price at amount:  %d", priceAtAmount);
                console2.log("TEST:  - Price after swap: %d", priceAfterSwap);

                if (hasPriceImpact) {
                    assertGe(
                        executedPrice
                            // within rounding tolerance
                            * (amounts[j] + 1) / amounts[j],
                        priceAtAmount,
                        "Price should be greater than executed price."
                    );
                    assertGt(
                        executedPrice
                            // within rounding tolerance
                            * (amounts[j] + 1) / amounts[j],
                        priceAfterSwap,
                        "Executed price should be greater than price after swap."
                    );
                    assertGt(
                        priceAtZero
                            // within rounding tolerance
                            * (amounts[j] + 1) / amounts[j],
                        executedPrice,
                        "Price should be greater than price after swap."
                    );
                } else {
                    assertGe(
                        priceAtZero
                            // within rounding tolerance
                            * (amounts[j] + 1) / amounts[j],
                        priceAfterSwap,
                        "Executed price should be or equal to price after swap."
                    );
                    assertGe(
                        priceAtZero
                            // within rounding tolerance
                            * (amounts[j] + 1) / amounts[j],
                        priceAtAmount,
                        "Executed price should be or equal to price after swap."
                    );
                    assertGe(
                        priceAtZero
                            // within rounding tolerance
                            * (amounts[j] + 1) / amounts[j],
                        executedPrice,
                        "Price should be or equal to price after swap."
                    );
                }
            } catch (bytes memory reason) {
                (bool isTooSmall, uint256 lowerLimit) =
                    decodeTooSmallError(reason);
                (bool isLimitExceeded, uint256 limit) =
                    decodeLimitExceededError(reason);

                if (isTooSmall) {
                    // We allow a TooSmall exception to occur for the smallest
                    // amount only.
                    if (j == 1) {
                        console2.log(
                            "TEST: TooSmall exception tolerated for smallest amount"
                        );
                    } else {
                        revert(
                            "TEST: TooSmall thrown for a significantly sized amount"
                        );
                    }
                } else if (isLimitExceeded) {
                    // We never allow LimitExceeded to be thrown, since all
                    // amounts should be within the stated limits.
                    revert(
                        "TEST: LimitExceeded thrown for an amount within limits"
                    );
                } else {
                    // any other revert reason bubbles up
                    assembly {
                        revert(add(reason, 32), mload(reason))
                    }
                }
            }

            vm.revertTo(initialState);
        }
        uint256 amountAboveLimit = sellLimit * 105 / 100;

        bool hasHardLimits = hasCapability(
            adapter.getCapabilities(poolId, tokenIn, tokenOut),
            Capability.HardLimits
        );

        if (hasHardLimits) {
            testRevertAboveLimit(
                adapter, poolId, tokenIn, tokenOut, amountAboveLimit
            );
        } else {
            testOperationsAboveLimit(
                adapter, poolId, tokenIn, tokenOut, amountAboveLimit
            );
        }

        console2.log("TEST: All tests passed.");
    }

    function testRevertAboveLimit(
        ISwapAdapter adapter,
        bytes32 poolId,
        address tokenIn,
        address tokenOut,
        uint256 amountAboveLimit
    ) internal {
        console2.log(
            "TEST: Testing revert behavior above the sell limit: %d",
            amountAboveLimit
        );
        uint256[] memory aboveLimitArray = new uint256[](1);
        aboveLimitArray[0] = amountAboveLimit;
        bool supportsLimitExceeded = false;

        try adapter.price(poolId, tokenIn, tokenOut, aboveLimitArray) {
            revert(
                "Pool shouldn't be able to fetch prices above the sell limit"
            );
        } catch (bytes memory reason) {
            (bool isTooSmall, uint256 lowerLimit) = decodeTooSmallError(reason);
            (bool isLimitExceeded, uint256 limit) =
                decodeLimitExceededError(reason);

            if (isLimitExceeded) {
                supportsLimitExceeded = true;
                console2.log(
                    "TEST: LimitExceeded supported! Thrown when fetching price above limit: %i",
                    limit
                );
            } else if (isTooSmall) {
                console2.log(
                    "TEST: UNEXPECTED TooSmall error when fetching price below limit: %i",
                    lowerLimit
                );
                revert TooSmall(lowerLimit);
            } else if (
                reason.length >= 4
                    && bytes4(reason) == bytes4(keccak256("Error(string)"))
            ) {
                string memory s = abi.decode(
                    sliceBytes(reason, 4, reason.length - 4), (string)
                );
                console2.log(
                    "TEST: Expected error when fetching price above limit: %s",
                    s
                );
            } else {
                // Unexpected error type: re-raise.
                assembly {
                    revert(add(reason, 32), mload(reason))
                }
            }
        }
        try adapter.swap(
            poolId, tokenIn, tokenOut, OrderSide.Sell, aboveLimitArray[0]
        ) {
            revert("Pool shouldn't be able to swap above the sell limit");
        } catch (bytes memory reason) {
            (bool isTooSmall, uint256 lowerLimit) = decodeTooSmallError(reason);
            (bool isLimitExceeded, uint256 limit) =
                decodeLimitExceededError(reason);

            if (isLimitExceeded) {
                supportsLimitExceeded = true;
                console2.log(
                    "TEST: LimitExceeded supported! Thrown when swapping above limit: %i",
                    limit
                );
            } else if (isTooSmall) {
                console2.log(
                    "TEST: UNEXPECTED TooSmall error when swapping above limit: %i",
                    lowerLimit
                );
                revert TooSmall(lowerLimit);
            } else if (
                reason.length >= 4
                    && bytes4(reason) == bytes4(keccak256("Error(string)"))
            ) {
                string memory s = abi.decode(
                    sliceBytes(reason, 4, reason.length - 4), (string)
                );
                console2.log(
                    "TEST: Expected error when swapping above limit: %s", s
                );
            } else {
                // Unexpected error type: re-raise.
                assembly {
                    revert(add(reason, 32), mload(reason))
                }
            }
        }
        if (supportsLimitExceeded) {
            console.log(unicode"Adapter supports LimitExceeded ✓");
        }
    }

    function testOperationsAboveLimit(
        ISwapAdapter adapter,
        bytes32 poolId,
        address tokenIn,
        address tokenOut,
        uint256 amountAboveLimit
    ) internal {
        console2.log(
            "TEST: Testing operations above the sell limit: %d",
            amountAboveLimit
        );
        uint256[] memory aboveLimitArray = new uint256[](1);
        aboveLimitArray[0] = amountAboveLimit;

        adapter.price(poolId, tokenIn, tokenOut, aboveLimitArray);
        adapter.swap(
            poolId, tokenIn, tokenOut, OrderSide.Sell, aboveLimitArray[0]
        );
    }

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

    function fractionToInt(Fraction memory price)
        public
        pure
        returns (uint256)
    {
        return price.toQ128x128();
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

    //
    // Custom Error Helper Functions
    // TODO should we expose these in a better location / library for solvers to
    // also leverage?

    // Helper function to check if error is TooSmall and decode it
    function decodeTooSmallError(bytes memory reason)
        internal
        pure
        returns (bool, uint256)
    {
        if (reason.length >= 4 && bytes4(reason) == TooSmall.selector) {
            if (reason.length == 36) {
                uint256 lowerLimit =
                    abi.decode(sliceBytes(reason, 4, 32), (uint256));
                return (true, lowerLimit);
            }
        }
        return (false, 0);
    }

    // Helper function to check if error is LimitExceeded and decode it
    function decodeLimitExceededError(bytes memory reason)
        internal
        pure
        returns (bool, uint256)
    {
        if (reason.length >= 4 && bytes4(reason) == LimitExceeded.selector) {
            if (reason.length == 36) {
                uint256 limit = abi.decode(sliceBytes(reason, 4, 32), (uint256));
                return (true, limit);
            }
        }
        return (false, 0);
    }

    // Helper function to slice bytes
    function sliceBytes(bytes memory data, uint256 start, uint256 length)
        internal
        pure
        returns (bytes memory)
    {
        bytes memory result = new bytes(length);
        for (uint256 i = 0; i < length; i++) {
            result[i] = data[start + i];
        }
        return result;
    }
}
