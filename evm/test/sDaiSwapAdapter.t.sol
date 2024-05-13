// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";
import "src/sDai/sDaiSwapAdapter.sol";
import "forge-std/console.sol";
import "forge-std/console2.sol";

/// @title sDaiSwapAdapterTest

contract sDaiSwapAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    sDaiSwapAdapter adapter;
    ISavingsDai savingsDai;

    address constant DAI_ADDRESS = 0x6B175474E89094C44Da98b954EedeAC495271d0F;
    address constant SDAI_ADDRESS = 0x83F20F44975D03b1b09e64809B757c47f942BEeA;
    address constant SAVINGS_DAI_PARAMETERS_ADDRESS = 0x197E90f9FAD81970bA7976f33CbD77088E5D7cf7;

    IERC20 constant DAI = IERC20(DAI_ADDRESS);
    IERC20 constant SDAI = IERC20(SDAI_ADDRESS);
    
    bytes32 constant PAIR = bytes32(0);

    uint256 constant TEST_ITERATIONS = 20;

    function setUp() public {
        uint256 forkBlock = 18835309;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new sDaiSwapAdapter(SDAI_ADDRESS);
    }

    //     function testPriceFuzz(uint256 amount0, uint256 amount1) public {
    //     bytes32 pair = bytes32(bytes20(USDC_WETH_PAIR));
    //     uint256[] memory limits = adapter.getLimits(pair, USDC, WETH);
    //     vm.assume(amount0 < limits[0]);
    //     vm.assume(amount1 < limits[0]);

    //     uint256[] memory amounts = new uint256[](2);
    //     amounts[0] = amount0;
    //     amounts[1] = amount1;

    //     Fraction[] memory prices = adapter.price(pair, WETH, USDC, amounts);

    //     for (uint256 i = 0; i < prices.length; i++) {
    //         assertGt(prices[i].numerator, 0);
    //         assertGt(prices[i].denominator, 0);
    //     }
    // }

    function testPriceAfterSwapEqPriceBeforeSwapSellDaiForSDai() public {
        testPriceAfterSwapEqPriceBeforeSwap(DAI_ADDRESS, SDAI_ADDRESS, OrderSide.Sell);
    }

    function testPriceAfterSwapEqPriceBeforeSwapSellSDaiForDai() public {
        testPriceAfterSwapEqPriceBeforeSwap(SDAI_ADDRESS, DAI_ADDRESS, OrderSide.Sell);
    }

    function testPriceAfterSwapEqPriceBeforeSwapBuySDaiWithDai() public {
        testPriceAfterSwapEqPriceBeforeSwap(DAI_ADDRESS, SDAI_ADDRESS, OrderSide.Buy);
    }

    function testPriceAfterSwapEqPriceBeforeSwapBuyDaiWithSDai() public {
        testPriceAfterSwapEqPriceBeforeSwap(SDAI_ADDRESS, DAI_ADDRESS, OrderSide.Buy);
    }

    function testPriceAfterSwapEqPriceBeforeSwap(address sellToken, address buyToken, OrderSide side ) internal {

        uint256 testIterations = 20;

        for(uint256 i = 1; i < testIterations; i++) {

            uint256 amount = 12345 * 10**i;

            Fraction memory priceBeforeSwap = adapter.getPriceSwapAt(sellToken, amount);

            deal(sellToken, address(this), type(uint256).max);
            IERC20(sellToken).approve(address(adapter), type(uint256).max);

            Trade memory trade = adapter.swap(PAIR, sellToken, buyToken, side, amount);

            Fraction memory priceAfterSwap = adapter.getPriceSwapAt(sellToken, amount);

            assertEq(FractionMath.compareFractions(priceBeforeSwap, priceAfterSwap), 0);
        }
    }

    function testSwapFuzzDaiForSDai(
        uint256 specifiedAmount,
        bool isBuy
    ) public {
        vm.assume(specifiedAmount > 1);

        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        uint256[] memory limits = adapter.getLimits(PAIR, DAI_ADDRESS, SDAI_ADDRESS);

        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1]);

            deal(DAI_ADDRESS, address(this), type(uint256).max);
            DAI.approve(address(adapter), type(uint256).max);
        } else {
            vm.assume(specifiedAmount < limits[0]);

            deal(DAI_ADDRESS, address(this), specifiedAmount);
            DAI.approve(address(adapter), specifiedAmount);
        }

        uint256 dai_balance_before = DAI.balanceOf(address(this));
        uint256 sDai_balance_before = SDAI.balanceOf(address(this));

        Trade memory trade =
            adapter.swap(PAIR, DAI_ADDRESS, SDAI_ADDRESS, side, specifiedAmount);

        uint256 dai_balance_after = DAI.balanceOf(address(this));
        uint256 sDai_balance_after = SDAI.balanceOf(address(this));

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    sDai_balance_after - sDai_balance_before
                );
                assertEq(
                    trade.calculatedAmount,
                    dai_balance_before - dai_balance_after
                );
            } else {
                assertEq(
                    specifiedAmount,
                    dai_balance_before - dai_balance_after
                );
                assertEq(
                    trade.calculatedAmount,
                    sDai_balance_after - sDai_balance_before
                );
            }
        }
    }

    function testSwapFuzzSDaiForDai(
        uint256 specifiedAmount,
        bool isBuy
    ) public {
        vm.assume(specifiedAmount > 1);

        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        uint256[] memory limits = adapter.getLimits(PAIR, SDAI_ADDRESS, DAI_ADDRESS);

        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1]);

            deal(SDAI_ADDRESS, address(this), type(uint256).max);
            SDAI.approve(address(adapter), type(uint256).max);
        } else {
            vm.assume(specifiedAmount < limits[0]);

            deal(SDAI_ADDRESS, address(this), specifiedAmount);
            SDAI.approve(address(adapter), specifiedAmount);
        }

        uint256 sDai_balance_before = SDAI.balanceOf(address(this));
        uint256 dai_balance_before = DAI.balanceOf(address(this));

        Trade memory trade =
            adapter.swap(PAIR, SDAI_ADDRESS, DAI_ADDRESS, side, specifiedAmount);

        uint256 sDai_balance_after = SDAI.balanceOf(address(this));
        uint256 dai_balance_after = DAI.balanceOf(address(this));

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    dai_balance_after - dai_balance_before
                );
                assertEq(
                    trade.calculatedAmount,
                    sDai_balance_before - sDai_balance_after
                );
            } else {
                assertEq(
                    specifiedAmount,
                    sDai_balance_before - sDai_balance_after
                );
                assertEq(
                    trade.calculatedAmount,
                    dai_balance_after - dai_balance_before
                );
            }
        }
    }

    /// @dev check why this test is failing
    function testSwapSellIncreasingDai() public {
        executeIncreasingSwapsDaiForSDai(OrderSide.Sell);
    }

    function testSwapBuyIncreasingSDai() public {
        executeIncreasingSwapsDaiForSDai(OrderSide.Buy);
    }

    /// @notice price is constant for any amount of DAI token we selling
    function executeIncreasingSwapsDaiForSDai(OrderSide side) public {
        uint256 amountConstant_ = 10 ** 18;

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        for (uint256 i = 1; i < TEST_ITERATIONS; i++) {
            amounts[i] = amountConstant_ * i;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap; 
        for (uint256 i = 1; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(DAI_ADDRESS, address(this), type(uint256).max);
            DAI.approve(address(adapter), type(uint256).max);

            trades[i] = adapter.swap(PAIR, DAI_ADDRESS, SDAI_ADDRESS, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
            assertEq(FractionMath.compareFractions(trades[i].price, trades[i + 1].price), 0);
        }
    }

    function testGetTokensSDai() public {
        address[] memory tokens = adapter.getTokens(PAIR);

        assertEq(tokens[0], DAI_ADDRESS);
        assertEq(tokens[1], SDAI_ADDRESS);
        assertEq(tokens.length, 2);
    }

    function testGetLimitsSDai() public {
        uint256[] memory limits = adapter.getLimits(PAIR, DAI_ADDRESS, SDAI_ADDRESS);
        console.log("Limit SellDai Dai: ", limits[0]);
        console.log("Limit SellDai sDai: ", limits[1]);
        assertEq(limits.length, 2);
    }


}
