// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import "src/ankr-bnb/AnkrBNBStakingPoolAdapter.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";

contract AnkrBNBStakingPoolAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    AnkrBNBStakingPoolAdapter adapter;
    ICertificateToken constant ankrBNB = ICertificateToken(0x52F24a5e03aee338Da5fd9Df68D2b6FAe1178827);
    IERC20 constant BNB = IERC20(address(0));
    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 34349980;
        vm.createSelectFork(vm.rpcUrl("bsc"), forkBlock);
        adapter = new
            AnkrBNBStakingPoolAdapter(IAnkrBNBStakingPool(0x9e347Af362059bf2E55839002c699F7A5BaFE86E));

        vm.label(address(adapter), "AnkrBNBStakingPoolAdapter");
        vm.label(address(0), "BNB");
        vm.label(address(ankrBNB), "ankrBNB");
    }

    function testMe() public {
        uint256[] memory tPrice = new uint256[](1);
        tPrice[0] = 10**18;
        Fraction[] memory prices = adapter.price(bytes32(0), IERC20(address(0)), IERC20(address(ankrBNB)), tPrice);
        uint256 ciao = prices[0].numerator;
        uint256[] memory limits = adapter.getLimits(bytes32(0), BNB, ankrBNB);
        console.log(limits[1]);
    }

    // function testPriceFuzz(uint256 amount0, uint256 amount1) public {
    //     bytes32 pair = bytes32(bytes20(USDC_WETH_PAIR));
    //     uint256[] memory limits = adapter.getLimits(pair, BNB, ankrBNB);
    //     vm.assume(amount0 < limits[0]);
    //     vm.assume(amount0 >= 0.0001 ether);
    //     vm.assume(amount1 < limits[0]);
    //     vm.assume(amount1 >= 0.0001 ether);

    //     uint256[] memory amounts = new uint256[](2);
    //     amounts[0] = amount0;
    //     amounts[1] = amount1;

    //     Fraction[] memory prices = adapter.price(pair, BNB, ankrBNB, amounts);

    //     for (uint256 i = 0; i < prices.length; i++) {
    //         assertGt(prices[i].numerator, 0);
    //         assertGt(prices[i].denominator, 0);
    //     }
    // }

    // function testPriceDecreasing() public {
    //     bytes32 pair = bytes32(bytes20(USDC_WETH_PAIR));
    //     uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

    //     for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
    //         amounts[i] = 1000 * i * 10 ** 6;
    //     }

    //     Fraction[] memory prices = adapter.price(pair, WETH, USDC, amounts);

    //     for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
    //         assertEq(prices[i].compareFractions(prices[i + 1]), 1);
    //         assertGt(prices[i].denominator, 0);
    //         assertGt(prices[i + 1].denominator, 0);
    //     }
    // }

    // function testSwapFuzz(uint256 specifiedAmount, bool isBuy) public {
    //     OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

    //     bytes32 pair = bytes32(bytes20(USDC_WETH_PAIR));
    //     uint256[] memory limits = adapter.getLimits(pair, USDC, WETH);

    //     if (side == OrderSide.Buy) {
    //         vm.assume(specifiedAmount < limits[1]);

    //         // TODO calculate the amountIn by using price function as in
    //         // BalancerV2 testPriceDecreasing
    //         deal(address(USDC), address(this), type(uint256).max);
    //         USDC.approve(address(adapter), type(uint256).max);
    //     } else {
    //         vm.assume(specifiedAmount < limits[0]);

    //         deal(address(USDC), address(this), specifiedAmount);
    //         USDC.approve(address(adapter), specifiedAmount);
    //     }

    //     uint256 usdc_balance = USDC.balanceOf(address(this));
    //     uint256 weth_balance = WETH.balanceOf(address(this));

    //     Trade memory trade =
    //         adapter.swap(pair, USDC, WETH, side, specifiedAmount);

    //     if (trade.calculatedAmount > 0) {
    //         if (side == OrderSide.Buy) {
    //             assertEq(
    //                 specifiedAmount,
    //                 WETH.balanceOf(address(this)) - weth_balance
    //             );
    //             assertEq(
    //                 trade.calculatedAmount,
    //                 usdc_balance - USDC.balanceOf(address(this))
    //             );
    //         } else {
    //             assertEq(
    //                 specifiedAmount,
    //                 usdc_balance - USDC.balanceOf(address(this))
    //             );
    //             assertEq(
    //                 trade.calculatedAmount,
    //                 WETH.balanceOf(address(this)) - weth_balance
    //             );
    //         }
    //     }
    // }

    // function testSwapSellIncreasing() public {
    //     executeIncreasingSwaps(OrderSide.Sell);
    // }

    // function executeIncreasingSwaps(OrderSide side) internal {
    //     bytes32 pair = bytes32(bytes20(USDC_WETH_PAIR));

    //     uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
    //     for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
    //         amounts[i] = 1000 * i * 10 ** 6;
    //     }

    //     Trade[] memory trades = new Trade[](TEST_ITERATIONS);
    //     uint256 beforeSwap;
    //     for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
    //         beforeSwap = vm.snapshot();

    //         deal(address(USDC), address(this), amounts[i]);
    //         USDC.approve(address(adapter), amounts[i]);

    //         trades[i] = adapter.swap(pair, USDC, WETH, side, amounts[i]);
    //         vm.revertTo(beforeSwap);
    //     }

    //     for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
    //         assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
    //         assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
    //         assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
    //     }
    // }

    // function testSwapBuyIncreasing() public {
    //     executeIncreasingSwaps(OrderSide.Buy);
    // }

    // function testGetCapabilities(bytes32 pair, address t0, address t1) public {
    //     Capability[] memory res =
    //         adapter.getCapabilities(pair, IERC20(t0), IERC20(t1));

    //     assertEq(res.length, 3);
    // }

    // function testGetLimits() public {
    //     bytes32 pair = bytes32(bytes20(USDC_WETH_PAIR));
    //     uint256[] memory limits = adapter.getLimits(pair, USDC, WETH);

    //     assertEq(limits.length, 2);
    // }
}
