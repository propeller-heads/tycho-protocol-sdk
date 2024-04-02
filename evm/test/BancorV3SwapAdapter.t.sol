// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "forge-std/console.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";
import "src/bancor-v3/BancorV3SwapAdapter.sol";

/// @title TemplateSwapAdapterTest
/// @dev This is a template for a swap adapter test.
/// Test all functions that are implemented in your swap adapter, the two test included here are just an example.
/// Feel free to use UniswapV2SwapAdapterTest and BalancerV2SwapAdapterTest as a reference.
contract BancorV3SwapAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    BancorV3SwapAdapter adapter;

    address constant BANCOR_NETWORK_INFO_PROXY_ADDRESS = 0x8E303D296851B320e6a697bAcB979d13c9D6E760;

    IERC20 constant ETH = IERC20(0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE);
    IERC20 constant LINK = IERC20(0x514910771AF9Ca656af840dff83E8264EcF986CA);
    IERC20 constant BNT = IERC20(0x1F573D6Fb3F13d689FF844B4cE37794d79a7FF1C);
    IERC20 constant WBTC = IERC20(0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599);

    bytes32 constant PAIR = bytes32(0);
    uint256 constant TEST_ITERATIONS = 100;

    Token immutable eth = Token(address(ETH));
    Token immutable bnt = Token(address(BNT));
    Token immutable link = Token(address(LINK));
    Token immutable wbtc = Token(address(WBTC));
    
    receive() external payable {}

    function setUp() public {
        uint256 forkBlock = 19489171;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);

        adapter = new BancorV3SwapAdapter(BANCOR_NETWORK_INFO_PROXY_ADDRESS);
    }

    function testPriceFuzzBancorV3LinkBnt(uint256 amount0, uint256 amount1) public {
        uint256[] memory limits = adapter.getLimits(PAIR, LINK, BNT);
        uint256 minAmount = 1;

        vm.assume(amount0 < limits[0]);
        vm.assume(amount0 > minAmount);
        vm.assume(amount1 < limits[0]);
        vm.assume(amount1 > minAmount);
        
        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;

        Fraction[] memory prices = adapter.price(PAIR, LINK, BNT, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testPriceFuzzBancorV3BntLink(uint256 amount0, uint256 amount1) public {
        uint256[] memory limits = adapter.getLimits(PAIR, BNT, LINK);
        uint256 minAmount = 100;

        vm.assume(amount0 < limits[0]);
        vm.assume(amount0 > minAmount);
        vm.assume(amount1 < limits[0]);
        vm.assume(amount1 > minAmount);
        
        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;

        Fraction[] memory prices = adapter.price(PAIR, BNT, LINK, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    /// @dev need to fix for small amounts. Consider to implement a getLimits for minimum amount
    // function testPriceFuzzBancorV3LinkWbtc(uint256 amount0, uint256 amount1) public {
    //     uint256[] memory limits = adapter.getLimits(PAIR, LINK, WBTC);
    //     uint256 minAmount = 10000000000000000;

    //     vm.assume(amount0 < limits[0]);
    //     vm.assume(amount0 > minAmount);
    //     vm.assume(amount1 < limits[0]);
    //     vm.assume(amount1 > minAmount);
        
    //     uint256[] memory amounts = new uint256[](2);
    //     amounts[0] = amount0;
    //     amounts[1] = amount1;

    //     Fraction[] memory prices = adapter.price(PAIR, LINK, WBTC, amounts);

    //     for (uint256 i = 0; i < prices.length; i++) {
    //         assertGt(prices[i].numerator, 0);
    //         assertGt(prices[i].denominator, 0);
    //     }
    // }

    function testPriceFuzzBancorV3WbtcLink(uint256 amount0, uint256 amount1) public {
        uint256[] memory limits = adapter.getLimits(PAIR, WBTC, LINK);
        uint256 minAmount = 1;

        vm.assume(amount0 < limits[0]);
        vm.assume(amount0 > minAmount);
        vm.assume(amount1 < limits[0]);
        vm.assume(amount1 > minAmount);
        
        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;

        Fraction[] memory prices = adapter.price(PAIR, WBTC, LINK, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testPriceDecreasingBancorV3LinkBnt() public {
        bytes32 pair = PAIR;
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * (i + 1) * 10 ** 6;
        }

        Fraction[] memory prices = adapter.price(pair, LINK, BNT, amounts);

        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            assertEq(prices[i].compareFractions(prices[i + 1]), 1);
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    function testPriceDecreasingBancorV3BntLink() public {
        bytes32 pair = PAIR;
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * (i + 1) * 10 ** 6;
        }

        Fraction[] memory prices = adapter.price(pair, BNT, LINK, amounts);

        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            assertEq(prices[i].compareFractions(prices[i + 1]), 1);
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    function testPriceDecreasingBancorV3LinkWbtc() public {
        bytes32 pair = PAIR;
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * (i + 1) * 10 ** 12;
        }

        Fraction[] memory prices = adapter.price(pair, LINK, WBTC, amounts);

        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            assertEq(prices[i].compareFractions(prices[i + 1]), 1);
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    function testPriceDecreasingBancorV3WbtcLink() public {
        bytes32 pair = PAIR;
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * (i + 1) * 10;
        }

        Fraction[] memory prices = adapter.price(pair, WBTC, LINK, amounts);

        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            assertEq(prices[i].compareFractions(prices[i + 1]), 1);
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    function testSwapFuzzBancorV3BntLink(uint256 specifiedAmount, bool isBuy) public {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        uint256[] memory limits = adapter.getLimits(PAIR, BNT, LINK);

        vm.assume(specifiedAmount > 1000);

        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1]);

            deal(address(BNT), address(this), type(uint256).max);
            BNT.approve(address(adapter), type(uint256).max);
        } else {
            vm.assume(specifiedAmount < limits[0]);

            deal(address(BNT), address(this), specifiedAmount);
            BNT.approve(address(adapter), specifiedAmount);
        }

        uint256 bnt_balance_before_swap = BNT.balanceOf(address(this));
        uint256 link_balance_before_swap = LINK.balanceOf(address(this));

        Trade memory trade =
            adapter.swap(PAIR, BNT, LINK, side, specifiedAmount);

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    LINK.balanceOf(address(this)) - link_balance_before_swap
                );
                assertEq(
                    trade.calculatedAmount,
                    bnt_balance_before_swap - BNT.balanceOf(address(this))
                );
            } else {
                assertEq(
                    specifiedAmount,
                    bnt_balance_before_swap - BNT.balanceOf(address(this))
                );
                assertEq(
                    trade.calculatedAmount,
                    LINK.balanceOf(address(this)) - link_balance_before_swap
                );
            }
        }
    }

    function testSwapSellIncreasingBancorV3() public {
        executeIncreasingSwaps(OrderSide.Sell);
    }

    function executeIncreasingSwaps(OrderSide side) internal {

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * (i + 1) * 10 ** 18;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(address(BNT), address(this), type(uint256).max);
            BNT.approve(address(adapter), type(uint256).max);

            trades[i] = adapter.swap(PAIR, BNT, LINK, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    function testSwapBuyIncreasingBancorV3() public {
        executeIncreasingSwaps(OrderSide.Buy);
    }

    function testGetPoolIdsBancorV3() public view {
        bytes32[] memory ids = adapter.getPoolIds(1, 20);
        console.log(ids.length);
        console.logBytes32(ids[1]);
    }

    function testGetLimitsBancorV3() public {
        uint256[] memory limits = adapter.getLimits(bytes32(0), BNT, LINK);
        assertEq(limits.length, 2);

        limits = adapter.getLimits(bytes32(0), ETH, BNT);
        assertEq(limits.length, 2);

        limits = adapter.getLimits(bytes32(0), LINK, ETH);
        assertEq(limits.length, 2);

        limits = adapter.getLimits(bytes32(0), ETH, LINK);
        assertEq(limits.length, 2);
    }

    function testPriceEqualPriceAfterSwapBancorV3() public {
        uint256 amountIn = 10 ether;

        uint256[] memory amounts = new uint256[](1);

        amounts[0] = amountIn;

        Fraction[] memory prices = adapter.price(PAIR, LINK, WBTC, amounts);

        deal(address(LINK), address(this), amountIn);
        LINK.approve(address(adapter), amountIn);

        Fraction memory priceSwap = adapter.swap(PAIR, LINK, WBTC, OrderSide.Sell, amountIn).price;

        console.log("Numerator Price: ", priceSwap.numerator);
        console.log("Numerator price Swap: ", prices[0].numerator);

        console.log("Denominator Price: ", priceSwap.denominator);
        console.log("Denominator price Swap: ", prices[0].denominator);

        assertEq(prices[0].numerator, priceSwap.numerator);
        assertEq(prices[0].denominator, priceSwap.denominator);
        
    }
}
