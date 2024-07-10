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

    // tokens
    address constant USDT = 0xdAC17F958D2ee523a2206206994597C13D831ec7;
    address constant USDC = 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48;
    address constant WETH = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
    address constant ETH = address(0);
    address constant WBETH = 0xa2E3356610840701BDf5611a53974510Ae27E2e1;
    address constant MIM = 0x99D8a9C45b2ecA8864373A26D1459e3Dff1e17F3;
    address constant THREE_CRV_TOKEN =
        0x6c3F90f043a72FA612cbac8115EE7e52BDe6E490;
    address constant DAI = 0x6B175474E89094C44Da98b954EedeAC495271d0F;

    // pools
    address constant STABLE_POOL = 0xbEbc44782C7dB0a1A60Cb6fe97d0b483032FF1C7;
    address constant CRYPTO_POOL = 0x80466c64868E1ab14a1Ddf27A676C3fcBE638Fe5;
    address constant STABLE_META_POOL =
        0x5a6A4D54456819380173272A5E8E9B9904BdF41B;
    address constant ETH_POOL = 0xBfAb6FA95E0091ed66058ad493189D2cB29385E6;
    address[] ADDITIONAL_POOLS_FOR_PRICE;

    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 20234346;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new CurveAdapter();

        // Additional pools that include custom Int128 pools
        ADDITIONAL_POOLS_FOR_PRICE = [
            0xEcd5e75AFb02eFa118AF914515D6521aaBd189F1,
            0xEd279fDD11cA84bEef15AF5D39BB4d4bEE23F0cA,
            0x43b4FdFD4Ff969587185cDB6f0BD875c5Fc83f8c,
            0x87650D7bbfC3A9F10587d7778206671719d9910D,
            0x9EfE1A1Cbd6Ca51Ee8319AFc4573d253C3B732af,
            0x4807862AA8b2bF68830e4C8dc86D0e9A998e085a,
            0xd632f22692FaC7611d2AA1C0D552930D43CAEd3B,
            0xA5407eAE9Ba41422680e2e00537571bcC53efBfD,
            0x5a6A4D54456819380173272A5E8E9B9904BdF41B,
            0x3211C6cBeF1429da3D0d58494938299C92Ad5860,
            0x50f3752289e1456BfA505afd37B241bca23e685d,
            0x4eBdF703948ddCEA3B11f675B4D1Fba9d2414A14,
            0xDB6925eA42897ca786a045B252D95aA7370f44b4,
            0xf861483fa7E511fbc37487D91B6FAa803aF5d37c,
            0x0E9B5B092caD6F1c5E6bc7f89Ffe1abb5c95F1C2,
            0x1e098B32944292969fB58c85bDC85545DA397117
        ];

        vm.label(address(adapter), "CurveAdapter");
        vm.label(USDT, "USDT");
        vm.label(USDC, "USDC");
        vm.label(STABLE_POOL, "STABLE_POOL");
        vm.label(WETH, "WETH");
        vm.label(CRYPTO_POOL, "CRYPTO_POOL");
    }

    receive() external payable {}

    function testPricesForAdditionalPools() public {
        uint256 len = ADDITIONAL_POOLS_FOR_PRICE.length;
        for (uint256 i = 0; i < len; i++) {
            bytes32 pair = bytes32(bytes20(ADDITIONAL_POOLS_FOR_PRICE[i]));
            address[] memory tokens = adapter.getTokens(pair);
            uint256[] memory amounts = new uint256[](1);

            try ICurveStableSwapPool(ADDITIONAL_POOLS_FOR_PRICE[i]).balances(0)
            returns (uint256 bal) {
                amounts[0] = bal / 10;
            } catch {
                amounts[0] = ICurveCustomInt128Pool(
                    ADDITIONAL_POOLS_FOR_PRICE[i]
                ).balances(int128(0)) / 10;
            }

            // Test Price
            Fraction[] memory prices =
                adapter.price(pair, tokens[0], tokens[1], amounts);

            // Test Limits
            uint256[] memory limits =
                adapter.getLimits(pair, tokens[0], tokens[1]);

            assertGt(prices[0].numerator, 0);
            assertGt(prices[0].denominator, 0);
            assertGt(limits[0], 0);
            assertGt(limits[1], 0);
        }
    }

    function testPriceFuzzCurveStableSwap(uint256 amount0, uint256 amount1)
        public
    {
        bytes32 pair = bytes32(bytes20(STABLE_POOL));
        uint256[] memory limits = adapter.getLimits(pair, USDC, USDT);
        vm.assume(amount0 < limits[0]);
        vm.assume(amount1 < limits[0]);

        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;

        Fraction[] memory prices = adapter.price(pair, USDC, USDT, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testPriceFuzzCurveCryptoSwap(uint256 amount0, uint256 amount1)
        public
    {
        bytes32 pair = bytes32(bytes20(CRYPTO_POOL));
        uint256[] memory limits = adapter.getLimits(pair, USDT, WETH);
        vm.assume(amount0 < limits[0]);
        vm.assume(amount1 < limits[0]);

        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;

        Fraction[] memory prices = adapter.price(pair, USDT, WETH, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testSwapFuzzCurveStableSwap(uint256 specifiedAmount) public {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(STABLE_POOL));
        uint256[] memory limits = adapter.getLimits(pair, USDC, USDT);

        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 4);

        deal(USDC, address(this), specifiedAmount);
        IERC20(USDC).approve(address(adapter), specifiedAmount);

        uint256 usdc_balance = IERC20(USDC).balanceOf(address(this));
        uint256 USDT_balance = IERC20(USDT).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, USDC, USDT, side, specifiedAmount);

        if (side == OrderSide.Buy) {
            assertEq(
                specifiedAmount,
                IERC20(USDT).balanceOf(address(this)) - USDT_balance
            );
            assertEq(
                trade.calculatedAmount,
                usdc_balance - IERC20(USDC).balanceOf(address(this))
            );
        } else {
            assertEq(
                specifiedAmount,
                usdc_balance - IERC20(USDC).balanceOf(address(this))
            );
            assertEq(
                trade.calculatedAmount,
                IERC20(USDT).balanceOf(address(this)) - USDT_balance
            );
        }
    }

    function testSwapFuzzCurveCryptoSwap(uint256 specifiedAmount) public {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(CRYPTO_POOL));
        uint256[] memory limits = adapter.getLimits(pair, WETH, USDT);

        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 6);

        deal(WETH, address(this), specifiedAmount);
        IERC20(WETH).approve(address(adapter), specifiedAmount);

        uint256 WETH_balance = IERC20(WETH).balanceOf(address(this));
        uint256 USDT_balance = IERC20(USDT).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, WETH, USDT, side, specifiedAmount);

        assertEq(
            specifiedAmount,
            WETH_balance - IERC20(WETH).balanceOf(address(this))
        );
        assertEq(
            trade.calculatedAmount,
            IERC20(USDT).balanceOf(address(this)) - USDT_balance
        );
    }

    function testSwapFuzzCurveCryptoSwapUsingEth(uint256 specifiedAmount)
        public
    {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(CRYPTO_POOL));
        uint256[] memory limits = adapter.getLimits(pair, ETH, USDT);

        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 6);

        deal(address(adapter), specifiedAmount);

        uint256 ETH_balance = address(adapter).balance;
        uint256 USDT_balance = IERC20(USDT).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, ETH, USDT, side, specifiedAmount);

        assertEq(specifiedAmount, ETH_balance - address(adapter).balance);
        assertEq(
            trade.calculatedAmount,
            IERC20(USDT).balanceOf(address(this)) - USDT_balance
        );
    }

    function testSwapFuzzCurveStablePoolEthWithEth(uint256 specifiedAmount)
        public
    {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(ETH_POOL));
        uint256[] memory limits = adapter.getLimits(pair, ETH, WBETH);

        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 14);

        deal(address(adapter), specifiedAmount);

        uint256 eth_balance = address(adapter).balance;
        uint256 WBETH_balance = IERC20(WBETH).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, ETH, WBETH, side, specifiedAmount);

        assertEq(specifiedAmount, eth_balance - address(adapter).balance);
        assertEq(
            trade.calculatedAmount,
            IERC20(WBETH).balanceOf(address(this)) - WBETH_balance
        );
    }

    function testSwapFuzzCurveStablePoolEthWithToken(uint256 specifiedAmount)
        public
    {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(ETH_POOL));
        uint256[] memory limits = adapter.getLimits(pair, WBETH, ETH);

        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 14);

        deal(address(WBETH), address(this), specifiedAmount);
        IERC20(WBETH).approve(address(adapter), specifiedAmount);

        uint256 eth_balance = address(this).balance;
        uint256 WBETH_balance = IERC20(WBETH).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, WBETH, ETH, side, specifiedAmount);

        assertEq(trade.calculatedAmount, address(this).balance - eth_balance);
        assertEq(
            specifiedAmount,
            WBETH_balance - IERC20(WBETH).balanceOf(address(this))
        );
    }

    function testSwapSellIncreasingSwapsCurve() public {
        executeIncreasingSwapsStableSwap(OrderSide.Sell);
        executeIncreasingSwapsCryptoSwap(OrderSide.Sell);
    }

    function executeIncreasingSwapsStableSwap(OrderSide side) internal {
        bytes32 pair = bytes32(bytes20(CRYPTO_POOL));

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * i * 10 ** 14;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(WETH, address(this), amounts[i]);
            IERC20(WETH).approve(address(adapter), amounts[i]);

            trades[i] = adapter.swap(pair, WETH, USDT, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    function executeIncreasingSwapsCryptoSwap(OrderSide side) internal {
        bytes32 pair = bytes32(bytes20(CRYPTO_POOL));

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * i * 10 ** 6;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(WETH, address(this), amounts[i]);
            IERC20(WETH).approve(address(adapter), amounts[i]);

            trades[i] = adapter.swap(pair, WETH, USDT, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    function testGetCapabilitiesCurveSwap(bytes32 pair, address t0, address t1)
        public
    {
        Capability[] memory res = adapter.getCapabilities(pair, t0, t1);

        assertEq(res.length, 2);
    }

    function testGetTokensCurveStableSwap() public {
        bytes32 pair = bytes32(bytes20(STABLE_POOL));
        address[] memory tokens = adapter.getTokens(pair);

        assertGe(tokens.length, 2);
    }

    function testGetTokensCurveCryptoSwap() public {
        bytes32 pair = bytes32(bytes20(CRYPTO_POOL));
        address[] memory tokens = adapter.getTokens(pair);

        assertGe(tokens.length, 2);
    }

    function testGetLimitsCurveStableSwap() public {
        bytes32 pair = bytes32(bytes20(STABLE_POOL));
        uint256[] memory limits = adapter.getLimits(pair, USDC, USDT);

        assertEq(limits.length, 2);
    }

    function testGetLimitsCurveCryptoSwap() public {
        bytes32 pair = bytes32(bytes20(CRYPTO_POOL));
        uint256[] memory limits = adapter.getLimits(pair, WETH, USDT);

        assertEq(limits.length, 2);
    }
}
