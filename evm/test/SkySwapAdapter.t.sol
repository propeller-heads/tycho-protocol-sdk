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

    struct TokenPair {
        address token0;
        address token1;
        address converter;  // Contract responsible for conversion
    }

    mapping(uint256 => TokenPair) pairs;

    SkySwapAdapter adapter;
    ISavingsDai savingsDai;
    IDssLitePSM daiLitePSM;
    IDaiUsdsConverter daiUsdsConverter;
    IUsdsPsmWrapper usdsPsmWrapper;
    ISUsds sUsds;
    IMkrSkyConverter mkrSkyConverter;

    address constant SDAI_ADDRESS = 0x83F20F44975D03b1b09e64809B757c47f942BEeA;
    address constant DAI_LITE_PSM_ADDRESS = 0xf6e72Db5454dd049d0788e411b06CfAF16853042;
    address constant DAI_USDS_CONVERTER_ADDRESS = 0x3225737a9Bbb6473CB4a45b7244ACa2BeFdB276A;
    address constant USDS_PSM_WRAPPER_ADDRESS = 0xA188EEC8F81263234dA3622A406892F3D630f98c;
    address constant SUSDS_ADDRESS = 0xa3931d71877C0E7a3148CB7Eb4463524FEc27fbD;
    address constant MKR_SKY_CONVERTER_ADDRESS = 0xBDcFCA946b6CDd965f99a839e4435Bcdc1bc470B;


    address constant DAI_ADDRESS = 0x6B175474E89094C44Da98b954EedeAC495271d0F;
    address constant USDS_ADDRESS = 0xdC035D45d973E3EC169d2276DDab16f1e407384F;
    address constant USDC_ADDRESS = 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48;
    address constant MKR_ADDRESS = 0x9f8F72aA9304c8B593d555F12eF6589cC3A579A2;
    address constant SKY_ADDRESS = 0x56072C95FAA701256059aa122697B133aDEd9279;

    IERC20 constant DAI = IERC20(DAI_ADDRESS);
    IERC20 constant SDAI = IERC20(SDAI_ADDRESS);
    IERC20 constant USDS = IERC20(USDS_ADDRESS);
    IERC20 constant MKR = IERC20(MKR_ADDRESS);
    IERC20 constant SKY = IERC20(SKY_ADDRESS);
    IERC20 constant USDC = IERC20(USDC_ADDRESS);
    bytes32 constant PAIR = bytes32(0);
    uint256 constant NUM_PAIRS = 6;  // Total number of token pairs

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
            converter: SDAI_ADDRESS         // sDAI contract handles DAI<->sDAI
        });
        
        pairs[1] = TokenPair({
            token0: DAI_ADDRESS,
            token1: USDC_ADDRESS,
            converter: DAI_LITE_PSM_ADDRESS // PSM handles DAI<->USDC
        });
        
        pairs[2] = TokenPair({
            token0: DAI_ADDRESS,
            token1: USDS_ADDRESS,
            converter: DAI_USDS_CONVERTER_ADDRESS // Converter handles DAI<->USDS
        });
        
        pairs[3] = TokenPair({
            token0: USDS_ADDRESS,
            token1: USDC_ADDRESS,
            converter: USDS_PSM_WRAPPER_ADDRESS  // PSM wrapper handles USDS<->USDC
        });
        
        pairs[4] = TokenPair({
            token0: USDS_ADDRESS,
            token1: SUSDS_ADDRESS,
            converter: SUSDS_ADDRESS        // sUSDS contract handles USDS<->sUSDS
        });
        
        pairs[5] = TokenPair({
            token0: MKR_ADDRESS,
            token1: SKY_ADDRESS,
            converter: MKR_SKY_CONVERTER_ADDRESS // Converter handles MKR<->SKY
        });
    }

    ////////////////////////////////// DAI-sDAI ///////////////////////////////////////

    // DAI -> sDAI | PASS
    function testSwapFuzzDaiForSDai(uint256 specifiedAmount, bool isBuy)
        public
    {
        vm.assume(specifiedAmount > 1);

        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        uint256[] memory limits =
            adapter.getLimits(PAIR, DAI_ADDRESS, SDAI_ADDRESS);

        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1]);

            deal(DAI_ADDRESS, address(this), type(uint256).max);
            DAI.approve(address(adapter), type(uint256).max);
        } else {
            vm.assume(specifiedAmount < limits[0]);

            deal(DAI_ADDRESS, address(this), specifiedAmount);
            DAI.approve(address(adapter), specifiedAmount);
        }

        uint256 dai_balance_before = DAI.balanceOf(address(this));
        uint256 sDai_balance_before = SDAI.balanceOf(address(this));

        Trade memory trade =
            adapter.swap(PAIR, DAI_ADDRESS, SDAI_ADDRESS, side, specifiedAmount);

        uint256 dai_balance_after = DAI.balanceOf(address(this));
        uint256 sDai_balance_after = SDAI.balanceOf(address(this));

        if (side == OrderSide.Buy) {
            assertEq(specifiedAmount, sDai_balance_after - sDai_balance_before);
            assertEq(
                trade.calculatedAmount, dai_balance_before - dai_balance_after
            );
        } else {
            assertEq(specifiedAmount, dai_balance_before - dai_balance_after);
            assertEq(
                trade.calculatedAmount, sDai_balance_after - sDai_balance_before
            );
        }
    }

    // sDAI -> DAI | PASS
    function testSwapFuzzSDaiForDai(uint256 specifiedAmount, bool isBuy)
        public
    {
        vm.assume(specifiedAmount > 1);

        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        uint256[] memory limits =
            adapter.getLimits(PAIR, SDAI_ADDRESS, DAI_ADDRESS);

        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1]);

            deal(SDAI_ADDRESS, address(this), type(uint256).max);
            SDAI.approve(address(adapter), type(uint256).max);
        } else {
            vm.assume(specifiedAmount < limits[0]);

            deal(SDAI_ADDRESS, address(this), specifiedAmount);
            SDAI.approve(address(adapter), specifiedAmount);
        }

        uint256 sDai_balance_before = SDAI.balanceOf(address(this));
        uint256 dai_balance_before = DAI.balanceOf(address(this));

        Trade memory trade =
            adapter.swap(PAIR, SDAI_ADDRESS, DAI_ADDRESS, side, specifiedAmount);

        uint256 sDai_balance_after = SDAI.balanceOf(address(this));
        uint256 dai_balance_after = DAI.balanceOf(address(this));

        if (side == OrderSide.Buy) {
            assertEq(specifiedAmount, dai_balance_after - dai_balance_before);
            assertEq(
                trade.calculatedAmount, sDai_balance_before - sDai_balance_after
            );
        } else {
            assertEq(specifiedAmount, sDai_balance_before - sDai_balance_after);
            assertEq(
                trade.calculatedAmount, dai_balance_after - dai_balance_before
            );
        }
    }

    ////////////////////////////////// DAI-USDC ///////////////////////////////////////

    // USDC-DAI | SELL SIDE PASS | BUY SIDE PASS
    function testSwapFuzzUsdcForDai(uint256 specifiedAmount, bool isBuy)
        public
    {
        vm.assume(specifiedAmount > 10e6);

        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        uint256[] memory limits =
            adapter.getLimits(PAIR, USDC_ADDRESS, DAI_ADDRESS);

        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1]);

            deal(USDC_ADDRESS, address(this), 10e24);
            USDC.approve(address(adapter), 10e24);
        } else {
            vm.assume(specifiedAmount < limits[0]);

            deal(USDC_ADDRESS, address(this), specifiedAmount);
            USDC.approve(address(adapter), specifiedAmount);
        }
        
        uint256 usdc_balance_before = USDC.balanceOf(address(this));
        console.log("usdc_balance_before", usdc_balance_before);
        uint256 dai_balance_before = DAI.balanceOf(address(this));
        console.log("dai_balance_before", dai_balance_before);

        Trade memory trade =
            adapter.swap(PAIR, USDC_ADDRESS, DAI_ADDRESS, side, specifiedAmount);

        uint256 usdc_balance_after = USDC.balanceOf(address(this));
        console.log("usdc_balance_after", usdc_balance_after);
        uint256 dai_balance_after = DAI.balanceOf(address(this));
        console.log("dai_balance_after", dai_balance_after);

        if (side == OrderSide.Buy) {
            // Allow for small rounding errors (up to 0.001 DAI or 10^15 wei)
            uint256 daiReceived = dai_balance_after - dai_balance_before;
            assertApproxEqAbs(
                specifiedAmount,
                daiReceived,
                1e15,
                "DAI amount received differs too much from specified"
            );
            assertEq(
                trade.calculatedAmount, usdc_balance_before - usdc_balance_after
            );
        } else {
            assertEq(specifiedAmount, usdc_balance_before - usdc_balance_after);
            assertApproxEqAbs(
                trade.calculatedAmount,
                dai_balance_after - dai_balance_before,
                1e15,
                "DAI calculation differs too much from actual"
            );
        }
    }

    // DAI-USDC | BUY SIDE | SELL SIDE
    function testSwapFuzzDaiForUsdc(uint256 specifiedAmount, bool isBuy)
        public
    {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        uint256[] memory limits =
            adapter.getLimits(PAIR, DAI_ADDRESS, USDC_ADDRESS);

        if (side == OrderSide.Buy) {
            // When buying USDC, specifiedAmount is in USDC (6 decimals)
            vm.assume(specifiedAmount < limits[1]);
            
            deal(DAI_ADDRESS, address(this), 10e30);
            DAI.approve(address(adapter), 10e30);
        } else {
            // When selling DAI, specifiedAmount is in DAI (18 decimals)
            vm.assume(specifiedAmount < limits[0]);
            
            deal(DAI_ADDRESS, address(this), specifiedAmount);
            DAI.approve(address(adapter), specifiedAmount);
        }
        
        uint256 dai_balance_before = DAI.balanceOf(address(this));
        uint256 usdc_balance_before = USDC.balanceOf(address(this));

        Trade memory trade =
            adapter.swap(PAIR, DAI_ADDRESS, USDC_ADDRESS, side, specifiedAmount);

        uint256 dai_balance_after = DAI.balanceOf(address(this));
        uint256 usdc_balance_after = USDC.balanceOf(address(this));

        if (side == OrderSide.Buy) {
            // When buying USDC, specified amount is in USDC (6 decimals)
            assertEq(specifiedAmount, usdc_balance_after - usdc_balance_before);
            assertEq(
                trade.calculatedAmount, dai_balance_before - dai_balance_after
            );
        } else {
            // When selling DAI, specified amount is in DAI (18 decimals)
            console.log("specifiedAmount", specifiedAmount);
            console.log("dai_balance_before", dai_balance_before);
            console.log("dai_balance_after", dai_balance_after);
            console.log("trade.calculatedAmount", trade.calculatedAmount);
            console.log("usdc_balance_before", usdc_balance_before);
            console.log("usdc_balance_after", usdc_balance_after);
            assertEq(specifiedAmount, dai_balance_before - dai_balance_after);
            assertEq(
                trade.calculatedAmount, usdc_balance_after - usdc_balance_before
            );
        }
    }

    ////////////////////////////////////////// Get Limits ///////////////////////////////////////

    function testGetLimitsSkyProtocolPairs() public view {
        for (uint256 i = 0; i < NUM_PAIRS; i++) {
            TokenPair memory pair = pairs[i];
            uint256[] memory limits = adapter.getLimits(PAIR, pair.token0, pair.token1);
            console.log("pair", pair.token0, pair.token1);
            console.log("limits", limits[0], limits[1]);
            assertEq(limits.length, 2);
        }
    }

    function testGetLimitsSDai() public view {
        uint256[] memory limits =
            adapter.getLimits(PAIR, DAI_ADDRESS, SDAI_ADDRESS);
        console.log("limits", limits[0], limits[1]);
        assertEq(limits.length, 2);
    }

    function testGetLimitsUsdcDaiPair() public view {
        uint256[] memory limits =
            adapter.getLimits(PAIR, USDC_ADDRESS, DAI_ADDRESS);
        console.log("limits", limits[0], limits[1]);
        assertEq(limits.length, 2);
    }

    // This test is currently broken due to a bug in runPoolBehaviour
    // with constant price pools.
    // The conversion between DAI <-> sDAI is linear, meaning the underlying
    // relationship
    // between them is determined by the yield accrued by the DAI in the
    // SavingsDai (sDAI) contract (which is constant in a given block).
    // and it is consistent regardless of the amount being swapped.
    // function testPoolBehaviourSDai() public {
    //     bytes32[] memory poolIds = new bytes32[](1);
    //     poolIds[0] = PAIR;
    //     runPoolBehaviourTest(adapter, poolIds);
    // }
}