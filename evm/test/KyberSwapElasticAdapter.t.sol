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
    address constant WETH = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
    address constant USDT = 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48;
    address constant USDT_WETH_PAIR = 0xF138462C76568CDFD77C6EB831E973D6963F2006;

    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 18747990;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter =
            new KyberSwapElasticAdapter(0xC7a590291e07B9fe9E64b86c58fD8fC764308C4A);

        vm.label(address(adapter), "KyberSwapElasticAdapter");
        vm.label(WETH, "WETH");
        vm.label(USDT, "USDT");
        vm.label(USDT_WETH_PAIR, "USDT_WETH_PAIR");
    }

    function testSwapFuzzKyberSwapElastic(uint256 specifiedAmount, bool isBuy) public {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(USDT_WETH_PAIR));
        uint256[] memory limits = adapter.getLimits(pair, USDT, WETH);

        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1]);

            deal(USDT, address(this), type(uint256).max);
            IERC20(USDT).approve(address(adapter), type(uint256).max);
        } else {
            vm.assume(specifiedAmount < limits[0]);

            deal(USDT, address(this), specifiedAmount);
            IERC20(USDT).approve(address(adapter), specifiedAmount);
        }

        uint256 USDT_balance = IERC20(USDT).balanceOf(address(this));
        uint256 weth_balance = IERC20(WETH).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, USDT, WETH, side, specifiedAmount);

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    IERC20(WETH).balanceOf(address(this)) - weth_balance
                );
                assertEq(
                    trade.calculatedAmount,
                    USDT_balance - IERC20(USDT).balanceOf(address(this))
                );
            } else {
                assertEq(
                    specifiedAmount,
                    USDT_balance - IERC20(USDT).balanceOf(address(this))
                );
                assertEq(
                    trade.calculatedAmount,
                    IERC20(WETH).balanceOf(address(this)) - weth_balance
                );
            }
        }
    }

    function testSwapSellIncreasingKyberSwapElastic() public {
        executeIncreasingSwaps(OrderSide.Sell);
    }

    function executeIncreasingSwaps(OrderSide side) internal {
        bytes32 pair = bytes32(bytes20(USDT_WETH_PAIR));

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * i * 10 ** 6;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(USDT, address(this), amounts[i]);
            IERC20(USDT).approve(address(adapter), amounts[i]);

            trades[i] = adapter.swap(pair, USDT, WETH, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
        }
    }

    function testSwapBuyIncreasingKyberSwapElastic() public {
        executeIncreasingSwaps(OrderSide.Buy);
    }

    function testGetCapabilitiesKyberSwapElastic(bytes32 pair, address t0, address t1) public {
        Capability[] memory res = adapter.getCapabilities(pair, t0, t1);

        assertEq(res.length, 2);
    }

    function testGetLimitsKyberSwapElastic() public {
        bytes32 pair = bytes32(bytes20(USDT_WETH_PAIR));
        uint256[] memory limits = adapter.getLimits(pair, USDT, WETH);

        assertEq(limits.length, 2);
    }
}
