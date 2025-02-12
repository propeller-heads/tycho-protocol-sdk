// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "./AdapterTest.sol";
import "src/libraries/FractionMath.sol";
import "src/sky/SkySwapAdapter.sol";

/// @title SkySwapAdapterTest

contract SkySwapAdapterTest is Test, ISwapAdapterTypes, AdapterTest {
    using FractionMath for Fraction;

    struct SwapResult {
        uint256 sellBalanceBefore;
        uint256 buyBalanceBefore;
        uint256 sellBalanceAfter;
        uint256 buyBalanceAfter;
        Trade trade;
    }

    struct PairInfo {
        bytes32 id;
        address token0;
        address token1;
        string name;
    }

    SkySwapAdapter adapter;
    ISavingsDai savingsDai;
    IDssLitePSM daiLitePSM;
    IDaiUsdsConverter daiUsdsConverter;
    IUsdsPsmWrapper usdsPsmWrapper;
    ISUsds sUsds;
    IMkrSkyConverter mkrSkyConverter;

    address constant DAI_LITE_PSM_ADDRESS =
        0xf6e72Db5454dd049d0788e411b06CfAF16853042;
    address constant DAI_USDS_CONVERTER_ADDRESS =
        0x3225737a9Bbb6473CB4a45b7244ACa2BeFdB276A;
    address constant USDS_PSM_WRAPPER_ADDRESS =
        0xA188EEC8F81263234dA3622A406892F3D630f98c;
    address constant MKR_SKY_CONVERTER_ADDRESS =
        0xBDcFCA946b6CDd965f99a839e4435Bcdc1bc470B;

    address constant DAI_ADDRESS = 0x6B175474E89094C44Da98b954EedeAC495271d0F;
    address constant SDAI_ADDRESS = 0x83F20F44975D03b1b09e64809B757c47f942BEeA;
    address constant USDS_ADDRESS = 0xdC035D45d973E3EC169d2276DDab16f1e407384F;
    address constant USDC_ADDRESS = 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48;
    address constant SUSDS_ADDRESS = 0xa3931d71877C0E7a3148CB7Eb4463524FEc27fbD;
    address constant MKR_ADDRESS = 0x9f8F72aA9304c8B593d555F12eF6589cC3A579A2;
    address constant SKY_ADDRESS = 0x56072C95FAA701256059aa122697B133aDEd9279;

    IERC20 constant DAI = IERC20(DAI_ADDRESS);
    IERC20 constant SDAI = IERC20(SDAI_ADDRESS);
    IERC20 constant USDS = IERC20(USDS_ADDRESS);
    IERC20 constant MKR = IERC20(MKR_ADDRESS);
    IERC20 constant SKY = IERC20(SKY_ADDRESS);
    IERC20 constant USDC = IERC20(USDC_ADDRESS);
    IERC20 constant SUSDS = IERC20(SUSDS_ADDRESS);
    bytes32 constant PAIR = bytes32(0);
    uint256 constant NUM_PAIRS = 6; // Total number of token pairs

    bytes32 constant DAI_SDAI_PAIR = bytes32(bytes20(SDAI_ADDRESS));
    bytes32 constant DAI_USDC_PAIR = bytes32(bytes20(DAI_LITE_PSM_ADDRESS));
    bytes32 constant DAI_USDS_PAIR =
        bytes32(bytes20(DAI_USDS_CONVERTER_ADDRESS));
    bytes32 constant USDS_USDC_PAIR = bytes32(bytes20(USDS_PSM_WRAPPER_ADDRESS));
    bytes32 constant USDS_SUSDS_PAIR = bytes32(bytes20(SUSDS_ADDRESS));
    bytes32 constant MKR_SKY_PAIR = bytes32(bytes20(MKR_SKY_CONVERTER_ADDRESS));

    uint256 constant PRECISION = 10 ** 18;
    uint256 constant MKR_TO_SKY_RATE = 24000;
    uint256 constant TEST_ITERATIONS = 100;

    PairInfo[] pairs;

    function setUp() public {
        uint256 forkBlock = 21678075;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new SkySwapAdapter(
            SDAI_ADDRESS,
            DAI_USDS_CONVERTER_ADDRESS,
            USDS_PSM_WRAPPER_ADDRESS,
            SUSDS_ADDRESS,
            MKR_SKY_CONVERTER_ADDRESS,
            DAI_LITE_PSM_ADDRESS,
            USDC_ADDRESS,
            MKR_ADDRESS,
            SKY_ADDRESS,
            DAI_ADDRESS,
            USDS_ADDRESS
        );

        // Initialize pairs array
        pairs.push(
            PairInfo(DAI_SDAI_PAIR, DAI_ADDRESS, SDAI_ADDRESS, "DAI-sDAI")
        );
        pairs.push(
            PairInfo(DAI_USDC_PAIR, DAI_ADDRESS, USDC_ADDRESS, "DAI-USDC")
        );
        pairs.push(
            PairInfo(DAI_USDS_PAIR, DAI_ADDRESS, USDS_ADDRESS, "DAI-USDS")
        );
        pairs.push(
            PairInfo(USDS_USDC_PAIR, USDS_ADDRESS, USDC_ADDRESS, "USDS-USDC")
        );
        pairs.push(
            PairInfo(USDS_SUSDS_PAIR, USDS_ADDRESS, SUSDS_ADDRESS, "USDS-sUSDS")
        );
        pairs.push(PairInfo(MKR_SKY_PAIR, MKR_ADDRESS, SKY_ADDRESS, "MKR-SKY"));

        vm.label(address(adapter), "SkySwapAdapter");
        vm.label(SDAI_ADDRESS, "sDAI");
        vm.label(DAI_USDS_CONVERTER_ADDRESS, "DaiUsdsConverter");
        vm.label(USDS_PSM_WRAPPER_ADDRESS, "UsdsPsmWrapper");
        vm.label(SUSDS_ADDRESS, "sUSDS");
        vm.label(MKR_SKY_CONVERTER_ADDRESS, "MkrSkyConverter");
        vm.label(DAI_ADDRESS, "DAI");
        vm.label(USDS_ADDRESS, "USDS");
        vm.label(USDC_ADDRESS, "USDC");
        vm.label(MKR_ADDRESS, "MKR");
        vm.label(SKY_ADDRESS, "SKY");
    }

    function setupTest(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        uint256 specifiedAmount,
        OrderSide side
    ) internal {
        uint256[] memory limits = adapter.getLimits(poolId, sellToken, buyToken);
        uint256 limitIndex = side == OrderSide.Buy ? 1 : 0;
        vm.assume(specifiedAmount < limits[limitIndex]);

        // Handle different decimal tokens
        if (sellToken == USDC_ADDRESS) {
            vm.assume(specifiedAmount > 10e6);
        } else {
            vm.assume(specifiedAmount > 1);
        }

        uint256 dealAmount = side == OrderSide.Buy ? 10 ** 50 : specifiedAmount;
        deal(sellToken, address(this), dealAmount);
        IERC20(sellToken).approve(address(adapter), dealAmount);
    }

    function executeSwap(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        uint256 specifiedAmount,
        OrderSide side
    ) internal returns (SwapResult memory result) {
        result.sellBalanceBefore = IERC20(sellToken).balanceOf(address(this));
        result.buyBalanceBefore = IERC20(buyToken).balanceOf(address(this));

        result.trade =
            adapter.swap(poolId, sellToken, buyToken, side, specifiedAmount);

        result.sellBalanceAfter = IERC20(sellToken).balanceOf(address(this));
        result.buyBalanceAfter = IERC20(buyToken).balanceOf(address(this));
        return result;
    }

    function verifySwap(
        address sellToken,
        address buyToken,
        SwapResult memory result,
        uint256 specifiedAmount,
        OrderSide side
    ) internal {
        bool needsApprox = sellToken == USDC_ADDRESS || buyToken == USDC_ADDRESS
            || sellToken == MKR_ADDRESS || buyToken == MKR_ADDRESS
            || sellToken == SKY_ADDRESS || buyToken == SKY_ADDRESS;

        uint256 tolerance = needsApprox ? 1e15 : 0;

        if (side == OrderSide.Buy) {
            if (needsApprox) {
                assertApproxEqAbs(
                    specifiedAmount,
                    result.buyBalanceAfter - result.buyBalanceBefore,
                    tolerance,
                    "Buy amount mismatch"
                );
            } else {
                assertEq(
                    specifiedAmount,
                    result.buyBalanceAfter - result.buyBalanceBefore,
                    "Buy amount mismatch"
                );
            }
            assertEq(
                result.trade.calculatedAmount,
                result.sellBalanceBefore - result.sellBalanceAfter,
                "Sell calculation mismatch"
            );
        } else {
            assertEq(
                specifiedAmount,
                result.sellBalanceBefore - result.sellBalanceAfter,
                "Sell amount mismatch"
            );
            if (needsApprox) {
                assertApproxEqAbs(
                    result.trade.calculatedAmount,
                    result.buyBalanceAfter - result.buyBalanceBefore,
                    tolerance,
                    "Buy calculation mismatch"
                );
            } else {
                assertEq(
                    result.trade.calculatedAmount,
                    result.buyBalanceAfter - result.buyBalanceBefore,
                    "Buy calculation mismatch"
                );
            }
        }
    }

    function testSwapAllPairs(uint256 specifiedAmount, bool isBuy) public {
        for (uint256 i = 0; i < pairs.length; i++) {
            OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

            // Forward direction
            setupTest(
                pairs[i].id,
                pairs[i].token0,
                pairs[i].token1,
                specifiedAmount,
                side
            );
            SwapResult memory result = executeSwap(
                pairs[i].id,
                pairs[i].token0,
                pairs[i].token1,
                specifiedAmount,
                side
            );
            verifySwap(
                pairs[i].token0, pairs[i].token1, result, specifiedAmount, side
            );

            // Reverse direction
            setupTest(
                pairs[i].id,
                pairs[i].token1,
                pairs[i].token0,
                specifiedAmount,
                side
            );
            result = executeSwap(
                pairs[i].id,
                pairs[i].token1,
                pairs[i].token0,
                specifiedAmount,
                side
            );
            verifySwap(
                pairs[i].token1, pairs[i].token0, result, specifiedAmount, side
            );
        }
    }

    function testPriceKeepAllPairs() public {
        for (uint256 i = 0; i < pairs.length; i++) {
            priceKeeping(pairs[i].id);
        }
    }

    function priceKeeping(bytes32 pairId) internal {
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        (address token0, address token1) = adapter.pairs(pairId);

        // Set initial amount based on token decimals
        uint256 initialAmount = (
            token0 == USDC_ADDRESS || token1 == USDC_ADDRESS
        ) ? 10 ** 6 : 10 ** 18;

        for (uint256 i = 1; i < TEST_ITERATIONS; i++) {
            amounts[i] = initialAmount * i;
        }

        Fraction[] memory prices =
            adapter.price(pairId, token0, token1, amounts);
        Fraction[] memory pricesInverse =
            adapter.price(pairId, token1, token0, amounts);

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
            assertGt(pricesInverse[i].denominator, 0);
            assertGt(pricesInverse[i + 1].denominator, 0);

            assertEq(
                FractionMath.compareFractions(prices[i], prices[i + 1]),
                0,
                "Forward price not constant"
            );
            assertEq(
                FractionMath.compareFractions(
                    pricesInverse[i], pricesInverse[i + 1]
                ),
                0,
                "Inverse price not constant"
            );
        }
    }

    function testPriceAfterSwapEqPriceBeforeSwapAllPairs() public {
        for (uint256 i = 0; i < pairs.length; i++) {
            // Forward direction
            testPriceAfterSwapEqPriceBeforeSwap(
                pairs[i].token0, pairs[i].token1, OrderSide.Buy, 10 ** 18
            );
            testPriceAfterSwapEqPriceBeforeSwap(
                pairs[i].token0, pairs[i].token1, OrderSide.Sell, 10 ** 18
            );

            // Reverse direction
            testPriceAfterSwapEqPriceBeforeSwap(
                pairs[i].token1, pairs[i].token0, OrderSide.Buy, 10 ** 18
            );
            testPriceAfterSwapEqPriceBeforeSwap(
                pairs[i].token1, pairs[i].token0, OrderSide.Sell, 10 ** 18
            );
        }
    }

    function testPriceAfterSwapEqPriceBeforeSwap(
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) internal {
        // Find the correct poolId
        bytes32 poolId;
        for (uint256 i = 0; i < pairs.length; i++) {
            if (
                (pairs[i].token0 == sellToken && pairs[i].token1 == buyToken)
                    || (pairs[i].token1 == sellToken && pairs[i].token0 == buyToken)
            ) {
                poolId = pairs[i].id;
                break;
            }
        }
        require(poolId != bytes32(0), "Pool not found");

        // Ensure amount is within limits
        uint256[] memory limits = adapter.getLimits(poolId, sellToken, buyToken);
        uint256 limitIndex = side == OrderSide.Buy ? 1 : 0;
        specifiedAmount = specifiedAmount % limits[limitIndex];
        if (specifiedAmount == 0) specifiedAmount = 1;

        uint256[] memory amounts = new uint256[](1);
        amounts[0] = specifiedAmount;

        Fraction[] memory priceBeforeSwap =
            adapter.price(poolId, sellToken, buyToken, amounts);

        // Deal tokens based on token type
        uint256 dealAmount = side == OrderSide.Buy ? 10 ** 50 : specifiedAmount;
        deal(sellToken, address(this), dealAmount);
        IERC20(sellToken).approve(address(adapter), dealAmount);

        Trade memory trade =
            adapter.swap(poolId, sellToken, buyToken, side, specifiedAmount);

        assertEq(
            FractionMath.compareFractions(priceBeforeSwap[0], trade.price),
            0,
            "Price changed after swap"
        );
    }

    function testGetTokens() public {
        for (uint256 i = 0; i < pairs.length; i++) {
            address[] memory tokens = adapter.getTokens(pairs[i].id);
            assertEq(tokens.length, 2);
            assertFalse(tokens[0] == address(0));
            assertFalse(tokens[1] == address(0));
        }
    }

    function testGetCapabilities() public {
        for (uint256 i = 0; i < pairs.length; i++) {
            Capability[] memory capabilities =
                adapter.getCapabilities(pairs[i].id, address(0), address(0));
            assertTrue(capabilities.length == 5 || capabilities.length == 6);
        }
    }

    // This test is currently broken due to a bug in runPoolBehaviour
    // with constant price pools.
    // function testPoolBehaviourSkyAdapter() public {
    //     bytes32[] memory pairs = new bytes32[](6);
    //     pairs[0] = DAI_SDAI_PAIR;
    //     pairs[1] = DAI_USDC_PAIR;
    //     pairs[2] = DAI_USDS_PAIR;
    //     pairs[3] = USDS_USDC_PAIR;
    //     pairs[4] = USDS_SUSDS_PAIR;
    //     pairs[5] = MKR_SKY_PAIR;
    //     for (uint256 i = 0; i < pairs.length; i++) {
    //         bytes32[] memory poolIds = new bytes32[](1);
    //         poolIds[0] = pairs[i];
    //         runPoolBehaviourTest(adapter, poolIds);
    //     }
    // }
}
