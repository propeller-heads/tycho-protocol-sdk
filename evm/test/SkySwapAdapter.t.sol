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

    struct TokenPair {
        address token0;
        address token1;
        address converter; // Contract responsible for conversion
    }

    mapping(uint256 => TokenPair) pairs;

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

    uint256 constant PRECISION = 10 ** 18;
    uint256 constant MKR_TO_SKY_RATE = 24000;
    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 21678075;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new SkySwapAdapter(
            SDAI_ADDRESS,
            DAI_LITE_PSM_ADDRESS,
            DAI_USDS_CONVERTER_ADDRESS,
            USDS_PSM_WRAPPER_ADDRESS,
            SUSDS_ADDRESS,
            MKR_SKY_CONVERTER_ADDRESS,
            DAI_ADDRESS,
            USDS_ADDRESS,
            USDC_ADDRESS,
            MKR_ADDRESS,
            SKY_ADDRESS
        );

        vm.label(address(adapter), "SkySwapAdapter");
        vm.label(DAI_LITE_PSM_ADDRESS, "DaiLitePSM");
        vm.label(DAI_USDS_CONVERTER_ADDRESS, "DaiUsdsConverter");
        vm.label(USDS_PSM_WRAPPER_ADDRESS, "UsdsPsmWrapper");
        vm.label(MKR_SKY_CONVERTER_ADDRESS, "MkrSkyConverter");
        vm.label(DAI_ADDRESS, "DAI");
        vm.label(SDAI_ADDRESS, "sDAI");
        vm.label(USDS_ADDRESS, "USDS");
        vm.label(USDC_ADDRESS, "USDC");
        vm.label(MKR_ADDRESS, "MKR");
        vm.label(SKY_ADDRESS, "SKY");

        // Initialize pairs mapping
        pairs[0] = TokenPair({
            token0: DAI_ADDRESS,
            token1: SDAI_ADDRESS,
            converter: SDAI_ADDRESS // sDAI contract handles DAI<->sDAI
        });

        pairs[1] = TokenPair({
            token0: DAI_ADDRESS,
            token1: USDC_ADDRESS,
            converter: DAI_LITE_PSM_ADDRESS // PSM handles DAI<->USDC
        });

        pairs[2] = TokenPair({
            token0: DAI_ADDRESS,
            token1: USDS_ADDRESS,
            converter: DAI_USDS_CONVERTER_ADDRESS // Converter handles
            // DAI<->USDS
        });

        pairs[3] = TokenPair({
            token0: USDS_ADDRESS,
            token1: USDC_ADDRESS,
            converter: USDS_PSM_WRAPPER_ADDRESS // PSM wrapper handles
            // USDS<->USDC
        });

        pairs[4] = TokenPair({
            token0: USDS_ADDRESS,
            token1: SUSDS_ADDRESS,
            converter: SUSDS_ADDRESS // sUSDS contract handles USDS<->sUSDS
        });

        pairs[5] = TokenPair({
            token0: MKR_ADDRESS,
            token1: SKY_ADDRESS,
            converter: MKR_SKY_CONVERTER_ADDRESS // Converter handles MKR<->SKY
        });
    }

    function setupTest(
        address sellToken,
        address buyToken,
        uint256 specifiedAmount,
        OrderSide side
    ) internal {
        uint256[] memory limits = adapter.getLimits(PAIR, sellToken, buyToken);
        uint256 limitIndex = side == OrderSide.Buy ? 1 : 0;
        vm.assume(specifiedAmount < limits[limitIndex]);

        // Handle different decimal tokens
        if (sellToken == USDC_ADDRESS) {
            vm.assume(specifiedAmount > 10e6);
        } else {
            vm.assume(specifiedAmount > 1);
        }

        uint256 dealAmount = side == OrderSide.Buy
            ? type(uint256).max
            : specifiedAmount;
        deal(sellToken, address(this), dealAmount);
        IERC20(sellToken).approve(address(adapter), dealAmount);
    }

    function executeSwap(
        address sellToken,
        address buyToken,
        uint256 specifiedAmount,
        OrderSide side
    ) internal returns (SwapResult memory result) {
        result.sellBalanceBefore = IERC20(sellToken).balanceOf(address(this));
        result.buyBalanceBefore = IERC20(buyToken).balanceOf(address(this));

        result.trade = adapter.swap(
            PAIR,
            sellToken,
            buyToken,
            side,
            specifiedAmount
        );

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
        bool needsApprox = sellToken == USDC_ADDRESS ||
            buyToken == USDC_ADDRESS ||
            sellToken == MKR_ADDRESS ||
            buyToken == MKR_ADDRESS ||
            sellToken == SKY_ADDRESS ||
            buyToken == SKY_ADDRESS;

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

    // DAI <-> sDAI (Pair 0)
    function testSwapFuzzDaiSDai(uint256 specifiedAmount, bool isBuy) public {
        TokenPair memory pair = pairs[0];
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        setupTest(pair.token0, pair.token1, specifiedAmount, side);
        SwapResult memory result = executeSwap(
            pair.token0,
            pair.token1,
            specifiedAmount,
            side
        );
        verifySwap(pair.token0, pair.token1, result, specifiedAmount, side);
    }

    // DAI <-> USDC (Pair 1)
    function testSwapFuzzDaiUsdc(uint256 specifiedAmount, bool isBuy) public {
        TokenPair memory pair = pairs[1];
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        setupTest(pair.token0, pair.token1, specifiedAmount, side);
        SwapResult memory result = executeSwap(
            pair.token0,
            pair.token1,
            specifiedAmount,
            side
        );
        verifySwap(pair.token0, pair.token1, result, specifiedAmount, side);
    }

    // DAI <-> USDS (Pair 2)
    function testSwapFuzzDaiUsds(uint256 specifiedAmount, bool isBuy) public {
        TokenPair memory pair = pairs[2];
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        setupTest(pair.token0, pair.token1, specifiedAmount, side);
        SwapResult memory result = executeSwap(
            pair.token0,
            pair.token1,
            specifiedAmount,
            side
        );
        verifySwap(pair.token0, pair.token1, result, specifiedAmount, side);
    }

    // USDS <-> USDC (Pair 3)
    function testSwapFuzzUsdsUsdc(uint256 specifiedAmount, bool isBuy) public {
        TokenPair memory pair = pairs[3];
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        setupTest(pair.token0, pair.token1, specifiedAmount, side);
        SwapResult memory result = executeSwap(
            pair.token0,
            pair.token1,
            specifiedAmount,
            side
        );
        verifySwap(pair.token0, pair.token1, result, specifiedAmount, side);
    }

    // USDS <-> sUSDS (Pair 4)
    function testSwapFuzzUsdsSUsds(uint256 specifiedAmount, bool isBuy) public {
        TokenPair memory pair = pairs[4];
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        setupTest(pair.token0, pair.token1, specifiedAmount, side);
        SwapResult memory result = executeSwap(
            pair.token0,
            pair.token1,
            specifiedAmount,
            side
        );
        verifySwap(pair.token0, pair.token1, result, specifiedAmount, side);
    }

    // MKR <-> SKY (Pair 5)
    function testSwapFuzzMkrSky(uint256 specifiedAmount, bool isBuy) public {
        TokenPair memory pair = pairs[5];
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        setupTest(pair.token0, pair.token1, specifiedAmount, side);
        SwapResult memory result = executeSwap(
            pair.token0,
            pair.token1,
            specifiedAmount,
            side
        );
        verifySwap(pair.token0, pair.token1, result, specifiedAmount, side);
    }

    function testPriceKeepDaiSDai() public {
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        uint256 amountConstant_ = 10 ** 18;

        for (uint256 i = 1; i < TEST_ITERATIONS; i++) {
            amounts[i] = amountConstant_ * i;
        }

        Fraction[] memory prices = adapter.price(
            PAIR,
            DAI_ADDRESS,
            SDAI_ADDRESS,
            amounts
        );

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertEq(
                FractionMath.compareFractions(prices[i], prices[i + 1]),
                0
            );
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    function testPriceKeepSDaiDai() public {
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        uint256 amountConstant_ = 10 ** 18;

        for (uint256 i = 1; i < TEST_ITERATIONS; i++) {
            amounts[i] = amountConstant_ * i;
        }

        Fraction[] memory prices = adapter.price(
            PAIR,
            SDAI_ADDRESS,
            DAI_ADDRESS,
            amounts
        );

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertEq(
                FractionMath.compareFractions(prices[i], prices[i + 1]),
                0
            );
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    function testPriceAfterSwapEqPriceBeforeSwapSellDaiSDai(
        uint256 specifiedAmount
    ) public {
        testPriceAfterSwapEqPriceBeforeSwap(
            DAI_ADDRESS,
            SDAI_ADDRESS,
            OrderSide.Sell,
            specifiedAmount
        );
    }

    function testPriceAfterSwapEqPriceBeforeSwapSellSDaiForDai(
        uint256 specifiedAmount
    ) public {
        testPriceAfterSwapEqPriceBeforeSwap(
            SDAI_ADDRESS,
            DAI_ADDRESS,
            OrderSide.Sell,
            specifiedAmount
        );
    }

    function testPriceAfterSwapEqPriceBeforeSwap(
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) internal {
        uint256[] memory limits = adapter.getLimits(PAIR, sellToken, buyToken);

        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 1);

        uint256[] memory specifiedAmount_ = new uint256[](1);

        Fraction[] memory priceBeforeSwap = adapter.price(
            PAIR,
            sellToken,
            buyToken,
            specifiedAmount_
        );

        deal(sellToken, address(this), specifiedAmount);
        IERC20(sellToken).approve(address(adapter), specifiedAmount);

        Trade memory trade = adapter.swap(
            PAIR,
            sellToken,
            buyToken,
            side,
            specifiedAmount
        );

        assertEq(
            FractionMath.compareFractions(priceBeforeSwap[0], trade.price),
            0
        );
    }

    // Price Keep tests for DAI-USDC
    function testPriceKeepDaiUsdc() public {
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        uint256 amountConstant_ = 10 ** 18;

        for (uint256 i = 1; i < TEST_ITERATIONS; i++) {
            amounts[i] = amountConstant_ * i;
        }

        Fraction[] memory prices = adapter.price(
            PAIR,
            DAI_ADDRESS,
            USDC_ADDRESS,
            amounts
        );

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertEq(
                FractionMath.compareFractions(prices[i], prices[i + 1]),
                0
            );
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    function testPriceKeepUsdcDai() public {
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        uint256 amountConstant_ = 10 ** 18;

        for (uint256 i = 1; i < TEST_ITERATIONS; i++) {
            amounts[i] = amountConstant_ * i;
        }

        Fraction[] memory prices = adapter.price(
            PAIR,
            USDC_ADDRESS,
            DAI_ADDRESS,
            amounts
        );

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertEq(
                FractionMath.compareFractions(prices[i], prices[i + 1]),
                0
            );
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    // Price Keep tests for DAI-USDS
    function testPriceKeepDaiUsds() public {
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        uint256 amountConstant_ = 10 ** 18;

        for (uint256 i = 1; i < TEST_ITERATIONS; i++) {
            amounts[i] = amountConstant_ * i;
        }

        Fraction[] memory prices = adapter.price(
            PAIR,
            DAI_ADDRESS,
            USDS_ADDRESS,
            amounts
        );

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertEq(
                FractionMath.compareFractions(prices[i], prices[i + 1]),
                0
            );
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    function testPriceKeepUsdsDai() public {
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        uint256 amountConstant_ = 10 ** 18;

        for (uint256 i = 1; i < TEST_ITERATIONS; i++) {
            amounts[i] = amountConstant_ * i;
        }

        Fraction[] memory prices = adapter.price(
            PAIR,
            USDS_ADDRESS,
            DAI_ADDRESS,
            amounts
        );

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertEq(
                FractionMath.compareFractions(prices[i], prices[i + 1]),
                0
            );
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    // Price Keep tests for USDS-USDC
    function testPriceKeepUsdsUsdc() public {
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        uint256 amountConstant_ = 10 ** 18;

        for (uint256 i = 1; i < TEST_ITERATIONS; i++) {
            amounts[i] = amountConstant_ * i;
        }

        Fraction[] memory prices = adapter.price(
            PAIR,
            USDS_ADDRESS,
            USDC_ADDRESS,
            amounts
        );

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertEq(
                FractionMath.compareFractions(prices[i], prices[i + 1]),
                0
            );
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    // Price Keep tests for USDS-sUSDS
    function testPriceKeepUsdsSUsds() public {
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        uint256 amountConstant_ = 10 ** 18;

        for (uint256 i = 1; i < TEST_ITERATIONS; i++) {
            amounts[i] = amountConstant_ * i;
        }

        Fraction[] memory prices = adapter.price(
            PAIR,
            USDS_ADDRESS,
            SUSDS_ADDRESS,
            amounts
        );

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertEq(
                FractionMath.compareFractions(prices[i], prices[i + 1]),
                0
            );
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    // Price Keep tests for MKR-SKY
    function testPriceKeepMkrSky() public {
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        uint256 amountConstant_ = 10 ** 18;

        for (uint256 i = 1; i < TEST_ITERATIONS; i++) {
            amounts[i] = amountConstant_ * i;
        }

        Fraction[] memory prices = adapter.price(
            PAIR,
            MKR_ADDRESS,
            SKY_ADDRESS,
            amounts
        );

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertEq(
                FractionMath.compareFractions(prices[i], prices[i + 1]),
                0
            );
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    // Price before/after swap tests for each pair
    function testPriceAfterSwapEqPriceBeforeSwapDaiUsdc(
        uint256 specifiedAmount
    ) public {
        testPriceAfterSwapEqPriceBeforeSwap(
            DAI_ADDRESS,
            USDC_ADDRESS,
            OrderSide.Sell,
            specifiedAmount
        );
    }

    function testPriceAfterSwapEqPriceBeforeSwapDaiUsds(
        uint256 specifiedAmount
    ) public {
        testPriceAfterSwapEqPriceBeforeSwap(
            DAI_ADDRESS,
            USDS_ADDRESS,
            OrderSide.Sell,
            specifiedAmount
        );
    }

    function testPriceAfterSwapEqPriceBeforeSwapUsdsUsdc(
        uint256 specifiedAmount
    ) public {
        testPriceAfterSwapEqPriceBeforeSwap(
            USDS_ADDRESS,
            USDC_ADDRESS,
            OrderSide.Sell,
            specifiedAmount
        );
    }

    function testPriceAfterSwapEqPriceBeforeSwapUsdsSUsds(
        uint256 specifiedAmount
    ) public {
        testPriceAfterSwapEqPriceBeforeSwap(
            USDS_ADDRESS,
            SUSDS_ADDRESS,
            OrderSide.Sell,
            specifiedAmount
        );
    }

    function testPriceAfterSwapEqPriceBeforeSwapMkrSky(
        uint256 specifiedAmount
    ) public {
        testPriceAfterSwapEqPriceBeforeSwap(
            MKR_ADDRESS,
            SKY_ADDRESS,
            OrderSide.Sell,
            specifiedAmount
        );
    }

    // Add increasing swap tests for remaining pairs
    function testSwapSellIncreasingDaiUsds() public {
        executeIncreasingSwaps(DAI_ADDRESS, USDS_ADDRESS, OrderSide.Sell, 10 ** 18);
    }

    function testSwapBuyIncreasingDaiUsds() public {
        executeIncreasingSwaps(DAI_ADDRESS, USDS_ADDRESS, OrderSide.Buy, 10 ** 18);
    }

    function testSwapSellIncreasingUsdsUsdc() public {
        executeIncreasingSwaps(USDS_ADDRESS, USDC_ADDRESS, OrderSide.Sell, 10 ** 18);
    }

    function testSwapBuyIncreasingUsdsUsdc() public {
        executeIncreasingSwaps(USDS_ADDRESS, USDC_ADDRESS, OrderSide.Buy, 10 ** 6);
    }

    function testSwapSellIncreasingUsdsSUsds() public {
        executeIncreasingSwaps(USDS_ADDRESS, SUSDS_ADDRESS, OrderSide.Sell, 10 ** 18);
    }

    function testSwapBuyIncreasingUsdsSUsds() public {
        executeIncreasingSwaps(USDS_ADDRESS, SUSDS_ADDRESS, OrderSide.Buy, 10 ** 18);
    }

    function testSwapSellIncreasingMkrSky() public {
        executeIncreasingSwaps(MKR_ADDRESS, SKY_ADDRESS, OrderSide.Sell, 10 ** 18);
    }

    function testSwapBuyIncreasingMkrSky() public {
        executeIncreasingSwaps(MKR_ADDRESS, SKY_ADDRESS, OrderSide.Buy, 10 ** 18);
    }

    /// @notice price is constant for any amount of DAI token we selling
    function executeIncreasingSwaps(address sellToken, address buyToken, OrderSide side, uint256 startingAmount) internal {

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        for (uint256 i = 1; i < TEST_ITERATIONS; i++) {
            amounts[i] = startingAmount * i;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 1; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(sellToken, address(this), type(uint256).max);
            IERC20(sellToken).approve(address(adapter), type(uint256).max);

            trades[i] =
                adapter.swap(PAIR, sellToken, buyToken, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
            assertEq(
                FractionMath.compareFractions(
                    trades[i].price, trades[i + 1].price
                ),
                0
            );
        }
    }
}
