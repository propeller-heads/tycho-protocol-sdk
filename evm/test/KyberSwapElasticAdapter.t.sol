// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import "src/kyberswap-elastic/KyberSwapElasticAdapter.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";

contract KyberSwapElasticAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    KyberSwapElasticAdapter adapter;
    address constant DAI = 0x6B175474E89094C44Da98b954EedeAC495271d0F;
    address constant USDC = 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48;
    address constant DAI_USDC_PAIR = 0xa847a0DDc1eAB013D2C8f6D8b37766DF99776BC6;

    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 18384222;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new KyberSwapElasticAdapter(
            0xC7a590291e07B9fe9E64b86c58fD8fC764308C4A
        );

        vm.label(address(adapter), "KyberSwapElasticAdapter");
        vm.label(USDC, "USDC");
        vm.label(DAI, "DAI");
        vm.label(DAI_USDC_PAIR, "DAI_USDC_PAIR");
    }

    function testSwapFuzzKyberSwapElasticDaiForUsdc(
        uint256 specifiedAmount,
        bool isBuy
    ) public {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(DAI_USDC_PAIR));
        uint256[] memory limits = adapter.getLimits(pair, DAI, USDC);

        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1] && specifiedAmount > 1);

            deal(DAI, address(this), type(uint256).max);
            IERC20(DAI).approve(address(adapter), type(uint256).max);
        } else {
            vm.assume(specifiedAmount < limits[0] && specifiedAmount > 1);

            deal(DAI, address(this), specifiedAmount);
            IERC20(DAI).approve(address(adapter), specifiedAmount);
        }

        uint256 DAI_balance = IERC20(DAI).balanceOf(address(this));
        uint256 USDC_balance = IERC20(USDC).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, DAI, USDC, side, specifiedAmount);

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    IERC20(USDC).balanceOf(address(this)) - USDC_balance
                );
                assertEq(
                    trade.calculatedAmount,
                    DAI_balance - IERC20(DAI).balanceOf(address(this))
                );
            } else {
                assertEq(
                    specifiedAmount,
                    DAI_balance - IERC20(DAI).balanceOf(address(this))
                );
                assertEq(
                    trade.calculatedAmount,
                    IERC20(USDC).balanceOf(address(this)) - USDC_balance
                );
            }
        }
    }

    function testSwapFuzzKyberSwapElasticUsdcForDai(
        uint256 specifiedAmount,
        bool isBuy
    ) public {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(DAI_USDC_PAIR));
        uint256[] memory limits = adapter.getLimits(pair, USDC, DAI);

        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1] && specifiedAmount > 1);

            deal(USDC, address(this), type(uint256).max);
            IERC20(USDC).approve(address(adapter), type(uint256).max);
        } else {
            vm.assume(specifiedAmount < limits[0] && specifiedAmount > 1);

            deal(USDC, address(this), specifiedAmount);
            IERC20(USDC).approve(address(adapter), specifiedAmount);
        }

        uint256 DAI_balance = IERC20(DAI).balanceOf(address(this));
        uint256 USDC_balance = IERC20(USDC).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, USDC, DAI, side, specifiedAmount);

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    IERC20(DAI).balanceOf(address(this)) - DAI_balance
                );
                assertEq(
                    trade.calculatedAmount,
                    USDC_balance - IERC20(USDC).balanceOf(address(this))
                );
            } else {
                assertEq(
                    specifiedAmount,
                    USDC_balance - IERC20(USDC).balanceOf(address(this))
                );
                assertEq(
                    trade.calculatedAmount,
                    IERC20(DAI).balanceOf(address(this)) - DAI_balance
                );
            }
        }
    }

    function testSwapSellIncreasingKyberSwapElasticDaiForUsdc() public {
        executeIncreasingSwapsDaiForUsdc(OrderSide.Sell);
    }

    function executeIncreasingSwapsDaiForUsdc(OrderSide side) internal {
        bytes32 pair = bytes32(bytes20(DAI_USDC_PAIR));

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * i * 10 ** 6;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(DAI, address(this), amounts[i]);
            IERC20(DAI).approve(address(adapter), amounts[i]);

            trades[i] = adapter.swap(pair, DAI, USDC, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
        }
    }

    function testSwapBuyIncreasingKyberSwapElasticDaiForUsdc() public {
        executeIncreasingSwapsDaiForUsdc(OrderSide.Buy);
    }

    function testSwapSellIncreasingKyberSwapElasticUsdcForDai() public {
        executeIncreasingSwapsUsdcForDai(OrderSide.Sell);
    }

    function executeIncreasingSwapsUsdcForDai(OrderSide side) internal {
        bytes32 pair = bytes32(bytes20(DAI_USDC_PAIR));

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * i * 10 ** 6;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(USDC, address(this), amounts[i]);
            IERC20(USDC).approve(address(adapter), amounts[i]);

            trades[i] = adapter.swap(pair, USDC, DAI, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
        }
    }

    function testSwapBuyIncreasingKyberSwapElasticUsdcForDai() public {
        executeIncreasingSwapsUsdcForDai(OrderSide.Buy);
    }

    function testGetCapabilitiesKyberSwapElastic(
        bytes32 pair,
        address t0,
        address t1
    ) public {
        Capability[] memory res = adapter.getCapabilities(pair, t0, t1);

        assertEq(res.length, 2);
    }

    function testGetLimitsKyberSwapElastic() public {
        bytes32 pair = bytes32(bytes20(DAI_USDC_PAIR));
        uint256[] memory limits = adapter.getLimits(pair, DAI, USDC);

        assertEq(limits.length, 2);
    }

    function testGetTokensKyberSwapElastic() public {
        bytes32 pair = bytes32(bytes20(DAI_USDC_PAIR));
        address[] memory tokens = adapter.getTokens(pair);

        assertEq(tokens.length, 2);
    }
}
