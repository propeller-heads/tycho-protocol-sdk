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