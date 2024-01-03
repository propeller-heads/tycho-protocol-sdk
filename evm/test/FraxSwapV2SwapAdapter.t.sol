// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";
import "src/fraxswap-v2/FraxSwapV2SwapAdapter.sol";

///// Ethereum Network
//// factory Address: 0x43eC799eAdd63848443E2347C49f5f52e8Fe0F6f
/// FRAX-WETH Pair Address: 0x31351Bf3fba544863FBff44DDC27bA880916A199
// t0 - FRAX Token Address: 0x853d955aCEf822Db058eb8505911ED77F175b99e
// t1 - WETH Token Address: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2

contract FraxSwapV2SwapAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    FraxSwapV2SwapAdapter adapter;
    IERC20 constant FRAX = IERC20(0x853d955aCEf822Db058eb8505911ED77F175b99e);
    IERC20 constant WETH = IERC20(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2);
    address constant FRAX_WETH_PAIR = 0x31351Bf3fba544863FBff44DDC27bA880916A199;
    address constant factoryAddress = 0x43eC799eAdd63848443E2347C49f5f52e8Fe0F6f;

    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 18925175;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new FraxSwapV2SwapAdapter(factoryAddress);

        vm.label(address(FRAX), "FRAX");
        vm.label(address(WETH), "WETH");
        vm.label(address(FRAX_WETH_PAIR), "FRAX_WETH_PAIR");
    }

    function testPriceFuzzFrax(uint256 amount0, uint256 amount1) public {
        bytes32 pair = bytes32(bytes20(FRAX_WETH_PAIR));
        uint256[] memory limits = adapter.getLimits(pair, FRAX, WETH);
        vm.assume(amount0 < limits[0]);
        vm.assume(amount1 < limits[1]);

        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;

        Fraction[] memory prices = adapter.price(pair, FRAX, WETH, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testPriceDecreasingFrax() public {
        bytes32 pair = bytes32(bytes20(FRAX_WETH_PAIR));
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * i * 10**18;
        }

        Fraction[] memory prices = adapter.price(pair, FRAX, WETH, amounts);

        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            assertEq(prices[i].compareFractions(prices[i + 1]), 1);
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    function testSwapFuzzFrax(uint256 specifiedAmount, bool isBuy) public {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(FRAX_WETH_PAIR));
        uint256[] memory limits = adapter.getLimits(pair, FRAX, WETH);

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
        uint256 weth_balance_before = WETH.balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, FRAX, WETH, side, specifiedAmount);

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    WETH.balanceOf(address(this)) - weth_balance_before
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
                    WETH.balanceOf(address(this)) - weth_balance_before
                );
            }
        }
    }

    function testSwapSellIncreasingFrax() public {
        executeIncreasingSwapsFrax(OrderSide.Sell);
    }

    function executeIncreasingSwapsFrax(OrderSide side) internal {
        
        bytes32 pair = bytes32(bytes20(FRAX_WETH_PAIR));

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


            trades[i] = adapter.swap(pair, FRAX, WETH, side, amounts[i]);
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
        bytes32 pair = bytes32(bytes20(FRAX_WETH_PAIR));
        uint256[] memory limits = adapter.getLimits(pair, FRAX, WETH);

        assertEq(limits.length, 2);
    }   

}