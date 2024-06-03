// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import "src/curve/CurveAdapter.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";

contract CurveAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    CurveAdapter adapter;
    address constant USDT = 0xdAC17F958D2ee523a2206206994597C13D831ec7;
    address constant USDC = 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48;
    address constant DAI = 0x6B175474E89094C44Da98b954EedeAC495271d0F;
    address constant EURS = 0xdB25f211AB05b1c97D595516F45794528a807ad8;
    address constant sEUR = 0xD71eCFF9342A5Ced620049e616c5035F1dB98620;
    address constant WETH = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
    address constant WBTC = 0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599;
    address constant GUSD =  0x056Fd409E1d7A124BD7017459dFEa2F387b6d5Cd;
    address constant FRAX = 0x853d955aCEf822Db058eb8505911ED77F175b99e;
    address constant TRECRV = 0x6c3F90f043a72FA612cbac8115EE7e52BDe6E490;

    address constant TRIPOOL_DAI_USDC_USDT_STABLEPOOL = 0xbEbc44782C7dB0a1A60Cb6fe97d0b483032FF1C7;
    address constant EURSPOOL_EURS_sEUR_STABLEPOOL = 0x0Ce6a5fF5217e38315f87032CF90686C96627CAA;
    address constant GUSD_DAI_USDC_USDT_STABLEMETAPOOL = 0x056Fd409E1d7A124BD7017459dFEa2F387b6d5Cd;
    address constant FRAX_USDC_CRYPTOPOOL = 0xDcEF968d416a41Cdac0ED8702fAC8128A64241A2;
    address constant WETH_TRIPOOL = 0x80466c64868E1ab14a1Ddf27A676C3fcBE638Fe5;

    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 20010426;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new CurveAdapter(0x90E00ACe148ca3b23Ac1bC8C240C2a7Dd9c2d7f5);

        vm.label(address(adapter), "CurveAdapter");
        vm.label(USDT, "USDT");
        vm.label(USDC, "USDC");
        vm.label(DAI, "DAI");
        vm.label(WBTC, "WBTC");
        vm.label(renBTC, "renBTC");
        vm.label(sBTC, "sBTC");
        vm.label(GUSD, "GUSD");
        vm.label(FRAX, "FRAX");
        vm.label(TRIPOOL_DAI_USDC_USDT_STABLEPOOL, "TRIPOOL_DAI_USDC_USDT_STABLEPOOL");
        vm.label(SBTC_POOL, "SBTC_POOL");
        vm.label(GUSD_DAI_USDC_USDT_STABLEMETAPOOL, "GUSD_DAI_USDC_USDT_STABLEMETAPOOL");
        vm.label(FRAX_USDC_CRYPTOPOOL, "FRAX_USDC_CRYPTOPOOL");
        vm.label(WETH, "WETH");
        vm.label(WETH_TRIPOOL, "WETH_TRIPOOL");
    }

    function testSwapFuzzCurveStableSwap(uint256 specifiedAmount) public {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(TRIPOOL_DAI_USDC_USDT_STABLEPOOL));
        uint256[] memory limits = adapter.getLimits(pair, USDC, USDT);

        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 6);

        deal(USDC, address(this), specifiedAmount);
        IERC20(USDC).approve(address(adapter), specifiedAmount);

        uint256 usdc_balance = IERC20(USDC).balanceOf(address(this));
        uint256 usdt_balance = IERC20(USDT).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, USDC, USDT, side, specifiedAmount);

        assertEq(
            specifiedAmount,
            usdc_balance - IERC20(USDC).balanceOf(address(this))
        );
        assertEq(
            trade.calculatedAmount,
            IERC20(USDT).balanceOf(address(this)) - usdt_balance
        );
    }

    function testSwapFuzzCurveCryptoSwap(uint256 specifiedAmount) public {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(WETH_TRIPOOL));
        uint256[] memory limits = adapter.getLimits(pair, WETH, USDT);

        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 10);

        deal(WETH, address(this), specifiedAmount);
        IERC20(WETH).approve(address(adapter), specifiedAmount);

        uint256 weth_balance = IERC20(WETH).balanceOf(address(this));
        uint256 usdt_balance = IERC20(USDT).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, WETH, USDT, side, specifiedAmount);

        assertEq(
            specifiedAmount,
            weth_balance - IERC20(WETH).balanceOf(address(this))
        );
        assertEq(
            trade.calculatedAmount,
            IERC20(USDT).balanceOf(address(this)) - usdt_balance
        );
    }

    function testSwapFuzzCurveStableSwap3Pool(uint256 specifiedAmount) public {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(TRIPOOL_DAI_USDC_USDT_STABLEPOOL));
        uint256[] memory limits = adapter.getLimits(pair, DAI, USDT);

        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 6);

        deal(DAI, address(this), specifiedAmount);
        IERC20(DAI).approve(address(adapter), specifiedAmount);

        uint256 dai_balance = IERC20(DAI).balanceOf(address(this));
        uint256 usdt_balance = IERC20(USDT).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, DAI, USDT, side, specifiedAmount);

        assertEq(
            specifiedAmount, dai_balance - IERC20(DAI).balanceOf(address(this))
        );
        assertEq(
            trade.calculatedAmount,
            IERC20(USDT).balanceOf(address(this)) - usdt_balance
        );
    }

    function testSwapFuzzCurveCryptoSwapSBTC(uint256 specifiedAmount) public {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(SBTC_POOL));
        uint256[] memory limits = adapter.getLimits(pair, WBTC, renBTC);

        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 8);

        deal(WBTC, address(this), specifiedAmount);
        IERC20(WBTC).approve(address(adapter), specifiedAmount);

        uint256 wbtc_balance = IERC20(WBTC).balanceOf(address(this));
        uint256 renbtc_balance = IERC20(renBTC).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, WBTC, renBTC, side, specifiedAmount);

        assertEq(
            specifiedAmount,
            wbtc_balance - IERC20(WBTC).balanceOf(address(this))
        );
        assertEq(
            trade.calculatedAmount,
            IERC20(renBTC).balanceOf(address(this)) - renbtc_balance
        );
    }

    function testSwapFuzzCurveMetaPoolGUSD(uint256 specifiedAmount) public {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(GUSD_DAI_USDC_USDT_STABLEMETAPOOL));
        uint256[] memory limits = adapter.getLimits(pair, GUSD, USDC);

        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 6);

        deal(GUSD, address(this), specifiedAmount);
        IERC20(GUSD).approve(address(adapter), specifiedAmount);

        uint256 gusd_balance = IERC20(GUSD).balanceOf(address(this));
        uint256 usdc_balance = IERC20(USDC).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, GUSD, USDC, side, specifiedAmount);

        assertEq(
            specifiedAmount,
            gusd_balance - IERC20(GUSD).balanceOf(address(this))
        );
        assertEq(
            trade.calculatedAmount,
            IERC20(USDC).balanceOf(address(this)) - usdc_balance
        );
    }

    function testSwapFuzzCurveMetaPoolFRAX(uint256 specifiedAmount) public {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(FRAX_USDC_CRYPTOPOOL));
        uint256[] memory limits = adapter.getLimits(pair, FRAX, USDC);

        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 6);

        deal(FRAX, address(this), specifiedAmount);
        IERC20(FRAX).approve(address(adapter), specifiedAmount);

        uint256 frax_balance = IERC20(FRAX).balanceOf(address(this));
        uint256 usdc_balance = IERC20(USDC).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, FRAX, USDC, side, specifiedAmount);

        assertEq(
            specifiedAmount,
            frax_balance - IERC20(FRAX).balanceOf(address(this))
        );
        assertEq(
            trade.calculatedAmount,
            IERC20(USDC).balanceOf(address(this)) - usdc_balance
        );
    }

    /// Test Swap Sell Increasing Swaps

    function testSwapSellIncreasingSwapsForAllPools() public {

        ////// STABLE POOLS //////

        // Pool: TRIPOOL_DAI_USDC_USDT_STABLEPOOL
        // PoolAddress: 0xbEbc44782C7dB0a1A60Cb6fe97d0b483032FF1C7
        // Tokens Index: O = DAI, 1 = USDC, 2 = USDT
        // SellToken: USDC
        // BuyToken: USDT
        /// @dev passes
        executeIncreasingSwapsStablePool(
            OrderSide.Sell, TRIPOOL_DAI_USDC_USDT_STABLEPOOL, USDC, USDT
        );

        // Pool: EURSPOOL_EURS_sEUR_STABLEPOOL
        // PoolAddress: 0x0Ce6a5fF5217e38315f87032CF90686C96627CAA
        // Tokens Index: 0 = EURS, 1 = sEUR
        // SellToken: EURS
        // BuyToken: sEUR
        /// @dev reverts
        // executeIncreasingSwapsStablePool(
        //     OrderSide.Sell, EURSPOOL_EURS_sEUR_STABLEPOOL, EURS, sEUR
        // );

        ////// STABLE META POOLS //////

        // Pool: GUSD_DAI_USDC_USDT_STABLEMETAPOOL
        // PoolAddress: 0x0Ce6a5fF5217e38315f87032CF90686C96627CAA
        // Base Coins Index: 0 = DAI, 1 = USDC, 2 = USDT
        // SellToken: DAI
        // BuyToken: USDC
        ///@dev reverts 
        // executeIncreasingSwapsMetaPool(
        //     OrderSide.Sell, GUSD_DAI_USDC_USDT_STABLEMETAPOOL, USDT, DAI
        // );


        // ////// CRYPTO POOLS //////

        // Pool: FRAX_USDC_CRYPTOPOOL
        // PoolAddress: 0xDcEF968d416a41Cdac0ED8702fAC8128A64241A2
        // Tokens Index: 0 = FRAX, 1 = USDC
        // SellToken: FRAX
        // BuyToken: USDC
        /// @dev passes
        executeIncreasingSwapsCryptoPool(
            OrderSide.Sell, FRAX_USDC_CRYPTOPOOL, FRAX, USDC
        );

        // Pool: WETH_WBTC_USDT_CRYPTOPOOL
        // Pool Address: 0x80466c64868E1ab14a1Ddf27A676C3fcBE638Fe5
        // Tokens Index: 0 = USDT, 1 = WBTC, 2 = WETH
        // SellToken: WETH
        // BuyToken: WBTC
        /// @dev reverts
        executeIncreasingSwapsCryptoPool(
            OrderSide.Sell, WETH_TRIPOOL, WETH, WBTC
        );

    }

    function executeIncreasingSwapsStablePool(
        OrderSide side,
        address pool,
        address sellToken,
        address buyToken
    ) internal {
        bytes32 pair = bytes32(bytes20(pool));

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * i * 10 ** 12;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(sellToken, address(this), amounts[i]);
            IERC20(sellToken).approve(address(adapter), amounts[i]);

            trades[i] = adapter.swap(pair, sellToken, buyToken, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    function executeIncreasingSwapsCryptoPool(
        OrderSide side,
        address pool,
        address sellToken,
        address buyToken
    ) internal {
        bytes32 pair = bytes32(bytes20(pool));

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * i * 10 ** 8;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(sellToken, address(this), amounts[i]);
            IERC20(sellToken).approve(address(adapter), amounts[i]);

            trades[i] = adapter.swap(pair, sellToken, buyToken, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    function executeIncreasingSwapsMetaPool(
        OrderSide side,
        address pool,
        address sellToken,
        address buyToken
    ) internal {
        bytes32 pair = bytes32(bytes20(pool));

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * i * 10 ** 6;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(sellToken, address(this), amounts[i]);
            IERC20(sellToken).approve(address(adapter), amounts[i]);

            trades[i] = adapter.swap(pair, sellToken, buyToken, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    // function testSwapSellIncreasingSwapsCurve() public {
    //     executeIncreasingSwapsStableSwap(OrderSide.Sell);
    //     executeIncreasingSwapsCryptoSwap(OrderSide.Sell);
    // }

    // function executeIncreasingSwapsStableSwap(OrderSide side) internal {
    //     bytes32 pair = bytes32(bytes20(WETH_TRIPOOL));

    //     uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
    //     for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
    //         amounts[i] = 1000 * i * 10 ** 14;
    //     }

    //     Trade[] memory trades = new Trade[](TEST_ITERATIONS);
    //     uint256 beforeSwap;
    //     for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
    //         beforeSwap = vm.snapshot();

    //         deal(WETH, address(this), amounts[i]);
    //         IERC20(WETH).approve(address(adapter), amounts[i]);

    //         trades[i] = adapter.swap(pair, WETH, USDT, side, amounts[i]);
    //         vm.revertTo(beforeSwap);
    //     }

    //     for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
    //         assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
    //         assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
    //     }
    // }

    // function executeIncreasingSwapsCryptoSwap(OrderSide side) internal {
    //     bytes32 pair = bytes32(bytes20(WETH_TRIPOOL));

    //     uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
    //     for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
    //         amounts[i] = 1000 * i * 10 ** 6;
    //     }

    //     Trade[] memory trades = new Trade[](TEST_ITERATIONS);
    //     uint256 beforeSwap;
    //     for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
    //         beforeSwap = vm.snapshot();

    //         deal(WETH, address(this), amounts[i]);
    //         IERC20(WETH).approve(address(adapter), amounts[i]);

    //         trades[i] = adapter.swap(pair, WETH, USDT, side, amounts[i]);
    //         vm.revertTo(beforeSwap);
    //     }

    //     for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
    //         assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
    //         assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
    //     }
    // }

    function testGetTokensCurveStableSwap3Pool() public {
        bytes32 pair = bytes32(bytes20(TRIPOOL_DAI_USDC_USDT_STABLEPOOL));
        address[] memory tokens = adapter.getTokens(pair);

        assertGe(tokens.length, 3);
    }

    function testGetTokensCurveCryptoSwapSBTC() public {
        bytes32 pair = bytes32(bytes20(SBTC_POOL));
        address[] memory tokens = adapter.getTokens(pair);

        assertGe(tokens.length, 3);
    }

    function testGetTokensCurveMetaPoolGUSD() public {
        bytes32 pair = bytes32(bytes20(GUSD_DAI_USDC_USDT_STABLEMETAPOOL));
        address[] memory tokens = adapter.getTokens(pair);

        assertGe(tokens.length, 3);
    }

    function testGetTokensCurveMetaPoolFRAX() public {
        bytes32 pair = bytes32(bytes20(FRAX_USDC_CRYPTOPOOL));
        address[] memory tokens = adapter.getTokens(pair);

        assertGe(tokens.length, 3);
    }

    function testGetLimitsCurveCryptoSwapSBTC() public {
        bytes32 pair = bytes32(bytes20(SBTC_POOL));
        uint256[] memory limits = adapter.getLimits(pair, WBTC, renBTC);

        assertEq(limits.length, 2);
    }

    function testGetLimitsCurveMetaPoolGUSD() public {
        bytes32 pair = bytes32(bytes20(GUSD_DAI_USDC_USDT_STABLEMETAPOOL));
        uint256[] memory limits = adapter.getLimits(pair, GUSD, USDC);

        assertEq(limits.length, 2);
    }

    function testGetLimitsCurveMetaPoolFRAX() public {
        bytes32 pair = bytes32(bytes20(FRAX_USDC_CRYPTOPOOL));
        uint256[] memory limits = adapter.getLimits(pair, FRAX, USDC);

        assertEq(limits.length, 2);
    }

    function testGetCapabilitiesCurveSwap(bytes32 pair, address t0, address t1)
        public
    {
        Capability[] memory res = adapter.getCapabilities(pair, t0, t1);

        assertEq(res.length, 1);
    }

    function testGetTokensCurveStableSwap() public {
        bytes32 pair = bytes32(bytes20(TRIPOOL_DAI_USDC_USDT_STABLEPOOL));
        address[] memory tokens = adapter.getTokens(pair);

        assertGe(tokens.length, 2);
    }

    function testGetTokensCurveCryptoSwap() public {
        bytes32 pair = bytes32(bytes20(WETH_TRIPOOL));
        address[] memory tokens = adapter.getTokens(pair);

        assertGe(tokens.length, 2);
    }

    function testGetPoolIdsCurveSwap() public {
        bytes32[] memory poolIds = adapter.getPoolIds(0, 2);

        assertEq(poolIds.length, 2);
    }

    function testGetLimitsCurveStableSwap() public {
        bytes32 pair = bytes32(bytes20(TRIPOOL_DAI_USDC_USDT_STABLEPOOL));
        uint256[] memory limits = adapter.getLimits(pair, USDC, USDT);

        assertEq(limits.length, 2);
    }

    function testGetLimitsCurveCryptoSwap() public {
        bytes32 pair = bytes32(bytes20(WETH_TRIPOOL));
        uint256[] memory limits = adapter.getLimits(pair, WETH, USDT);

        assertEq(limits.length, 2);
    }
}
