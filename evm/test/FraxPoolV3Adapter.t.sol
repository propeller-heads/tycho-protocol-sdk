// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";
import "src/frax-v3/FraxPoolV3Adapter.sol";

contract FraxPoolV3AdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    FraxPoolV3Adapter adapter;
    IERC20 constant FRAX = IERC20(0x853d955aCEf822Db058eb8505911ED77F175b99e);
    IERC20 constant USDC = IERC20(0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48);
    address constant poolAddress = 0x2fE065e6FFEf9ac95ab39E5042744d695F560729;

    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 18925175;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new FraxPoolV3Adapter(IFraxPoolV3(poolAddress));

        vm.label(address(FRAX), "FRAX");
        vm.label(address(USDC), "USDC");
    }

    function testPriceFuzzFraxV3(uint256 amount0, uint256 amount1) public {
        bytes32 pair = bytes32(0);
        uint256[] memory limits = adapter.getLimits(pair, FRAX, USDC);
        vm.assume(amount0 > 0.01 ether && amount0 < limits[0]);
        // vm.assume(amount1 > 0.01 ether && amount1 < limits[1]);


        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount0 + 0.01 ether;

        Fraction[] memory prices = adapter.price(pair, FRAX, USDC, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            // assertGt(prices[i].numerator, 0);
            // assertGt(prices[i].denominator, 0);
        }
    }

    function testPriceDecreasingFraxV3() public {
        bytes32 pair = bytes32(0);
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * i * 10**18;
        }

        Fraction[] memory prices = adapter.price(pair, FRAX, USDC, amounts);

        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            assertEq(prices[i].compareFractions(prices[i + 1]), 1);
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    function testSwapFuzzFrax(uint256 specifiedAmount, bool isBuy) public {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        bytes32 pair = bytes32(0);
        uint256[] memory limits = adapter.getLimits(pair, FRAX, USDC);

        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1]);

            deal(address(FRAX), address(this), type(uint256).max);
            FRAX.approve(address(adapter), type(uint256).max);
        } else {
            vm.assume(specifiedAmount < limits[0]);
            ///@dev Need to find the minimum specified amount acceptable
            vm.assume(specifiedAmount > 0.000001 ether);

            deal(address(FRAX), address(this), specifiedAmount);
            FRAX.approve(address(adapter), specifiedAmount);
        }

        uint256 frax_balance_before = FRAX.balanceOf(address(this));
        uint256 USDC_balance_before = USDC.balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, FRAX, USDC, side, specifiedAmount);

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    USDC.balanceOf(address(this)) - USDC_balance_before
                );
                assertEq(
                    trade.calculatedAmount,
                    frax_balance_before - FRAX.balanceOf(address(this))
                );
            } else {
                assertEq(
                    specifiedAmount,
                    frax_balance_before - FRAX.balanceOf(address(this))
                );
                assertEq(
                    trade.calculatedAmount,
                    USDC.balanceOf(address(this)) - USDC_balance_before
                );
            }
        }
    }

    function testSwapSellIncreasingFrax() public {
        executeIncreasingSwapsFrax(OrderSide.Sell);
    }

    function executeIncreasingSwapsFrax(OrderSide side) internal {
        
        bytes32 pair = bytes32(0);

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * i * 10 ** 6;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            if(side == OrderSide.Buy) {
                deal(address(FRAX), address(this), type(uint256).max);
                FRAX.approve(address(adapter), type(uint256).max);
            } else {
                deal(address(FRAX), address(this), amounts[i]);
                FRAX.approve(address(adapter), amounts[i]);
            }


            trades[i] = adapter.swap(pair, FRAX, USDC, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    function testSwapBuyIncreasingFrax() public {
        executeIncreasingSwapsFrax(OrderSide.Buy);
    }

    function testGetCapabilitiesFrax(bytes32 pair, address t0, address t1) public {
        Capability[] memory res =
            adapter.getCapabilities(pair, IERC20(t0), IERC20(t1));

        assertEq(res.length, 3);
    }

    function testGetLimitsFrax() public {
        bytes32 pair = bytes32(0);
        uint256[] memory limits = adapter.getLimits(pair, FRAX, USDC);

        assertEq(limits.length, 2);
    }   

}