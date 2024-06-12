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
    address constant THREE_CRV_TOKEN = 0x6c3F90f043a72FA612cbac8115EE7e52BDe6E490;
    address constant DAI = 0x6B175474E89094C44Da98b954EedeAC495271d0F;

    // pools
    address constant STABLE_POOL = 0xbEbc44782C7dB0a1A60Cb6fe97d0b483032FF1C7;
    address constant CRYPTO_POOL = 0x80466c64868E1ab14a1Ddf27A676C3fcBE638Fe5;
    address constant STABLE_META_POOL = 0x5a6A4D54456819380173272A5E8E9B9904BdF41B;
    address constant ETH_POOL = 0xBfAb6FA95E0091ed66058ad493189D2cB29385E6;

    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 19719570;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new CurveAdapter(0x90E00ACe148ca3b23Ac1bC8C240C2a7Dd9c2d7f5);

        vm.label(address(adapter), "CurveAdapter");
        vm.label(USDT, "USDT");
        vm.label(USDC, "USDC");
        vm.label(STABLE_POOL, "STABLE_POOL");
        vm.label(WETH, "WETH");
        vm.label(CRYPTO_POOL, "CRYPTO_POOL");
    }

    receive() external payable {}

    function testSwapFuzzCurveStableSwap(uint256 specifiedAmount) public {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(STABLE_POOL));
        uint256[] memory limits = adapter.getLimits(pair, USDC, USDT);

        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 4);

        deal(USDC, address(this), specifiedAmount);
        IERC20(USDC).approve(address(adapter), specifiedAmount);

        uint256 usdc_balance = IERC20(USDC).balanceOf(address(this));
        uint256 USDT_balance = IERC20(USDT).balanceOf(address(this));

        Trade memory trade = adapter.swap(
            pair,
            USDC,
            USDT,
            side,
            specifiedAmount
        );

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

        Trade memory trade = adapter.swap(
            pair,
            WETH,
            USDT,
            side,
            specifiedAmount
        );

        assertEq(
            specifiedAmount,
            WETH_balance - IERC20(WETH).balanceOf(address(this))
        );
        assertEq(
            trade.calculatedAmount,
            IERC20(USDT).balanceOf(address(this)) - USDT_balance
        );
    }

    function testSwapFuzzCurveCryptoSwapUsingEth(uint256 specifiedAmount) public {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(CRYPTO_POOL));
        uint256[] memory limits = adapter.getLimits(pair, ETH, USDT);

        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 6);

        deal(address(adapter), specifiedAmount);

        uint256 ETH_balance = address(adapter).balance;
        uint256 USDT_balance = IERC20(USDT).balanceOf(address(this));

        Trade memory trade = adapter.swap(
            pair,
            ETH,
            USDT,
            side,
            specifiedAmount
        );

        assertEq(
            specifiedAmount,
            ETH_balance - address(adapter).balance
        );
        assertEq(
            trade.calculatedAmount,
            IERC20(USDT).balanceOf(address(this)) - USDT_balance
        );
    }

    function testSwapFuzzCurveStableMetaSwapTokenVsUnderlying(uint256 specifiedAmount) public {
        address coin0 = MIM;
        address coin1 = USDT;

        bytes32 pair = bytes32(bytes20(STABLE_META_POOL));
        uint256[] memory limits = adapter.getLimits(pair, coin0, coin1);

        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 16);

        executeSwapFuzzMeta(pair, coin0, coin1, specifiedAmount);
    }

    function testSwapFuzzCurveStableMetaSwapTokenVsToken(uint256 specifiedAmount) public {
        address coin0 = MIM;
        address coin1 = THREE_CRV_TOKEN;

        bytes32 pair = bytes32(bytes20(STABLE_META_POOL));
        uint256[] memory limits = adapter.getLimits(pair, coin0, coin1);

        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 16);
        executeSwapFuzzMeta(pair, coin0, coin1, specifiedAmount);

    }

    function testSwapFuzzCurveStableMetaSwapUnderlyingVsUnderlying(uint256 specifiedAmount) public {
        address coin0 = DAI;
        address coin1 = USDC;

        bytes32 pair = bytes32(bytes20(STABLE_META_POOL));
        uint256[] memory limits = adapter.getLimits(pair, coin0, coin1);

        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 16);

        executeSwapFuzzMeta(pair, coin0, coin1, specifiedAmount);
    }

    function executeSwapFuzzMeta(bytes32 pair, address coin0, address coin1, uint256 specifiedAmount) internal {

        deal(coin0, address(this), specifiedAmount);
        IERC20(coin0).approve(address(adapter), specifiedAmount);

        uint256 coin0_balance = IERC20(coin0).balanceOf(address(this));
        uint256 coin1_balance = IERC20(coin1).balanceOf(address(this));

        Trade memory trade = adapter.swap(
            pair,
            coin0,
            coin1,
            OrderSide.Sell,
            specifiedAmount
        );

        assertEq(
            specifiedAmount,
            coin0_balance - IERC20(coin0).balanceOf(address(this))
        );
        assertEq(
            trade.calculatedAmount,
            IERC20(coin1).balanceOf(address(this)) - coin1_balance
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

        assertEq(
            specifiedAmount,
            eth_balance - address(adapter).balance
        );
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

        assertEq(
            trade.calculatedAmount,
            address(this).balance - eth_balance
        );
        assertEq(
            specifiedAmount,
            WBETH_balance - IERC20(WBETH).balanceOf(address(this))
        );
    }

    function testSwapSellIncreasingSwapsCurve() public {
        executeIncreasingSwapsStableSwap(OrderSide.Sell);
        executeIncreasingSwapsCryptoSwap(OrderSide.Sell);
    }

    function testSwapSellIncreasingCurveMeta() public {
        executeIncreasingSwapsStableSwapMeta(MIM, USDT);
        executeIncreasingSwapsStableSwapMeta(MIM, THREE_CRV_TOKEN);
        executeIncreasingSwapsStableSwapMeta(DAI, USDT);
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
            assertLe(
                trades[i].calculatedAmount,
                trades[i + 1].calculatedAmount
            );
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
            assertLe(
                trades[i].calculatedAmount,
                trades[i + 1].calculatedAmount
            );
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    function executeIncreasingSwapsStableSwapMeta(address coin0, address coin1) internal {
        bytes32 pair = bytes32(bytes20(STABLE_META_POOL));

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * i * 10 ** 10;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(coin0, address(this), amounts[i]);
            IERC20(coin0).approve(address(adapter), amounts[i]);

            trades[i] = adapter.swap(pair, coin0, coin1, OrderSide.Sell, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(
                trades[i].calculatedAmount,
                trades[i + 1].calculatedAmount
            );
        }
    }

    function testGetCapabilitiesCurveSwap(
        bytes32 pair,
        address t0,
        address t1
    ) public {
        Capability[] memory res = adapter.getCapabilities(pair, t0, t1);

        assertEq(res.length, 1);
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

    function testGetPoolIdsCurveSwap() public {
        bytes32[] memory poolIds = adapter.getPoolIds(0, 2);

        assertEq(poolIds.length, 2);
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
