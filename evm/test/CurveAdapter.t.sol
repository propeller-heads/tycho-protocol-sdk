// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import "src/curve/CurveAdapter.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";
import "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";

contract CurveAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    CurveAdapter adapter;

    address constant DAI = 0x6B175474E89094C44Da98b954EedeAC495271d0F;
    address constant USDC = 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48;
    address constant USDT = 0xdAC17F958D2ee523a2206206994597C13D831ec7;
    address constant GUSD = 0x056Fd409E1d7A124BD7017459dFEa2F387b6d5Cd;
    address constant TRECRV = 0x6c3F90f043a72FA612cbac8115EE7e52BDe6E490;
    address constant WBTC = 0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599;
    address constant WETH = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
    address constant ETH = 0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE;
    address constant wBETH = 0xa2E3356610840701BDf5611a53974510Ae27E2e1;
    address constant EURS = 0xdB25f211AB05b1c97D595516F45794528a807ad8;

    address constant STABLE_POOL_3POOL = 0xbEbc44782C7dB0a1A60Cb6fe97d0b483032FF1C7;
    address constant STABLE_POOL_META_GUSD3CRV = 0x4f062658eaaf2c1ccf8c8e36d6824cdf41167956;
    address constant CRYPTO_POOL_EURSUSD = 0x98a7F18d4E56Cfe84E3D081B40001B3d5bD3eB8B;
    address constant STABLE_POOL_ETHwBETHCRV = 0xBfAb6FA95E0091ed66058ad493189D2cB29385E6;
    address constant CRYPTO_POOL_TRICRYPTO2 = 0xD51a44d3FaE010294C616388b506AcdA1bfAAE46;

    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 19910426;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new CurveAdapter(0xF98B45FA17DE75FB1aD0e7aFD971b0ca00e379fC);

        vm.label(address(adapter), "CurveAdapter");
        vm.label(USDT, "USDT");
        vm.label(USDC, "USDC");
        vm.label(DAI, "DAI");
        vm.label(WBTC, "WBTC");
        vm.label(GUSD, "GUSD");
        vm.label(FRAX, "FRAX");
        vm.label(TRIPOOL_DAI_USDC_USDT_STABLEPOOL, "TRIPOOL_DAI_USDC_USDT_STABLEPOOL");
        vm.label(GUSD_DAI_USDC_USDT_STABLEMETAPOOL, "GUSD_DAI_USDC_USDT_STABLEMETAPOOL");
        vm.label(FRAX_USDC_CRYPTOPOOL, "FRAX_USDC_CRYPTOPOOL");
        vm.label(WETH, "WETH");
        vm.label(WETH_TRIPOOL, "WETH_TRIPOOL");
    }

    function testScemo() public {
        /**
            For each pool:
            If supports int128 -> do a swap(incl. mint token etc.)
            If does not support int128(uint256) -> do a swap
         */
        
        /**
            Step 3 example:
            for (i = ogni pool) {
                // get coin address at index 0
                // 
                // execute a swap with amount of 10**4
            }
         */

        ICurveRegistry registry = adapter.registry();

        uint256 poolLength = registry.pool_count();

        for (uint256 i = 0; i < poolLength; i++ ) {
            ICurveStableSwapPool pool = ICurveStableSwapPool(registry.pool_list(i));
            address[8] memory coins = registry.get_coins(address(pool));
            uint256 assetType = registry.get_pool_asset_type(address(pool));
            bool isStable = assetType == 0;

            if(isStable) {
                console.log("The pool: ", address(pool), "is STABLE");
            }
            else {
                console.log("The pool: ", address(pool), "should not be stable");
                console.log("With asset type", assetType);
            }
            for(uint256 j = 0; j < 3; j++) {
                if(coins[j] != address(0)) {
                    console.log("Coin:", coins[j]);
                }
            }
            console.log("");
        }

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
        // executeIncreasingSwapsStablePool(
        //     OrderSide.Sell, TRIPOOL_DAI_USDC_USDT_STABLEPOOL, USDC, USDT
        // );

        // Pool: EURSPOOL_EURS_sEUR_STABLEPOOL
        // PoolAddress: 0x0Ce6a5fF5217e38315f87032CF90686C96627CAA
        // Tokens Index: 0 = EURS, 1 = sEUR
        // SellToken: EURS
        // BuyToken: sEUR
        /// @dev reverts
        // try ICurveCryptoSwapPool(EURSPOOL_EURS_sEUR_STABLEPOOL).get_dy(0, 1, 10 ** 6) returns (
        //     uint256
        // ) {
        //     return;
        // } catch {
        //     return;
        // }
        executeIncreasingSwapsStablePool(
            OrderSide.Sell, EURSPOOL_EURS_sEUR_STABLEPOOL, EURS, sEUR
        );


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
        // executeIncreasingSwapsCryptoPool(
        //     OrderSide.Sell, FRAX_USDC_CRYPTOPOOL, FRAX, USDC
        // );

        // Pool: WETH_WBTC_USDT_CRYPTOPOOL
        // Pool Address: 0x80466c64868E1ab14a1Ddf27A676C3fcBE638Fe5
        // Tokens Index: 0 = USDT, 1 = WBTC, 2 = WETH
        // SellToken: WETH
        // BuyToken: WBTC
        /// @dev reverts
        // executeIncreasingSwapsCryptoPool(
        //     OrderSide.Sell, WETH_TRIPOOL, WETH, WBTC
        // );

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
