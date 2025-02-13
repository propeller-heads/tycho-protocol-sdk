// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import "src/renzo/RenzoAdapter.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "forge-std/console.sol";
import {FractionMath} from "src/libraries/FractionMath.sol";
import "./AdapterTest.sol";

contract RenzoAdapterTest is Test, ISwapAdapterTypes, AdapterTest {
    using FractionMath for Fraction;

    RenzoAdapter adapter;
    IERC20 ezETH;
    IRestakeManager constant restakeManager =
        IRestakeManager(0x74a09653A083691711cF8215a6ab074BB4e99ef5);
    IERC20 wBETH = IERC20(0xa2E3356610840701BDf5611a53974510Ae27E2e1);
    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 19797651;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        IERC20 ezETH_ = IERC20(address(restakeManager.ezETH()));
        IRenzoOracle renzoOracle_ = restakeManager.renzoOracle();

        adapter = new RenzoAdapter(
            address(restakeManager), address(renzoOracle_), address(ezETH_)
        );
        ezETH = ezETH_;

        vm.label(address(adapter), "RenzoAdapter");
        vm.label(address(renzoOracle_), "RenzoOracle");
        vm.label(address(ezETH), "ezETH");
        vm.label(address(wBETH), "wBETH");
    }

    function testPriceFuzzRenzo(uint256 amount0, uint256 amount1) public {
        bytes32 pair = bytes32(0);
        uint256[] memory limits =
            adapter.getLimits(pair, address(wBETH), address(ezETH));
        vm.assume(amount0 < limits[0] && amount0 > 10 ** 8);
        vm.assume(amount1 < limits[0] && amount1 > 10 ** 8);

        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;

        Fraction[] memory prices =
            adapter.price(pair, address(wBETH), address(ezETH), amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testSwapFuzzRenzo(uint256 specifiedAmount, bool isBuy) public {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        bytes32 pair = bytes32(0);
        uint256[] memory limits =
            adapter.getLimits(pair, address(wBETH), address(ezETH));
        Fraction[] memory price = new Fraction[](1);

        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1] && specifiedAmount > 10 ** 12);

            deal(address(wBETH), address(this), type(uint256).max);
            wBETH.approve(address(adapter), type(uint256).max);
        } else {
            vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 12);

            deal(address(wBETH), address(this), specifiedAmount);
            wBETH.approve(address(adapter), specifiedAmount);

            uint256[] memory specifiedAmounts = new uint256[](1);
            specifiedAmounts[0] = specifiedAmount;
            price = adapter.price(
                pair, address(wBETH), address(ezETH), specifiedAmounts
            );
        }

        uint256 wBETH_balance = wBETH.balanceOf(address(this));
        uint256 ezETH_balance = ezETH.balanceOf(address(this));

        Trade memory trade = adapter.swap(
            pair, address(wBETH), address(ezETH), side, specifiedAmount
        );

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertGe(
                    ezETH.balanceOf(address(this)) - ezETH_balance,
                    specifiedAmount - 10 ** 9
                );
                assertEq(
                    trade.calculatedAmount,
                    wBETH_balance - wBETH.balanceOf(address(this))
                );
            } else {
                assertEq(price[0].numerator, trade.price.numerator);
                assertEq(price[0].denominator, trade.price.denominator);
                assertEq(
                    specifiedAmount,
                    wBETH_balance - wBETH.balanceOf(address(this))
                );
                assertEq(
                    trade.calculatedAmount,
                    ezETH.balanceOf(address(this)) - ezETH_balance
                );
            }
        }
    }

    function testSwapFuzzRenzoWithETH(uint256 specifiedAmount, bool isBuy)
        public
    {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        bytes32 pair = bytes32(0);
        uint256[] memory limits =
            adapter.getLimits(pair, address(0), address(ezETH));
        Fraction[] memory price = new Fraction[](1);

        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1] && specifiedAmount > 10 ** 12);

            deal(address(adapter), type(uint256).max);
        } else {
            vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 12);

            deal(address(adapter), specifiedAmount);
            uint256[] memory specifiedAmounts = new uint256[](1);
            specifiedAmounts[0] = specifiedAmount;
            price = adapter.price(
                pair, address(0), address(ezETH), specifiedAmounts
            );
        }

        uint256 ETH_balance = address(adapter).balance;
        uint256 ezETH_balance = ezETH.balanceOf(address(this));

        Trade memory trade = adapter.swap(
            pair, address(0), address(ezETH), side, specifiedAmount
        );

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertGe(
                    ezETH.balanceOf(address(this)) - ezETH_balance,
                    specifiedAmount - 10 ** 9
                );
                assertEq(
                    trade.calculatedAmount,
                    ETH_balance - address(adapter).balance
                );
            } else {
                assertEq(price[0].numerator, trade.price.numerator);
                assertEq(price[0].denominator, trade.price.denominator);
                assertEq(
                    specifiedAmount, ETH_balance - address(adapter).balance
                );
                assertEq(
                    trade.calculatedAmount,
                    ezETH.balanceOf(address(this)) - ezETH_balance
                );
            }
        }
    }

    function testSwapSellIncreasingRenzo() public {
        executeIncreasingSwapsRenzo(OrderSide.Sell);
    }

    function testSwapBuyIncreasingRenzo() public {
        executeIncreasingSwapsRenzo(OrderSide.Buy);
    }

    function executeIncreasingSwapsRenzo(OrderSide side) internal {
        bytes32 pair = bytes32(0);

        uint256 amountConstant_ =
            side == OrderSide.Sell ? 1000 * 10 ** 7 : 10 ** 17;

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        amounts[0] = amountConstant_;
        for (uint256 i = 1; i < TEST_ITERATIONS; i++) {
            amounts[i] = amountConstant_ * i;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 1; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(address(wBETH), address(this), amounts[i]);
            wBETH.approve(address(adapter), amounts[i]);

            trades[i] = adapter.swap(
                pair, address(wBETH), address(ezETH), side, amounts[i]
            );
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
        }
    }

    function testGetCapabilitiesRenzo(bytes32 pair, address t0, address t1)
        public
    {
        Capability[] memory res = adapter.getCapabilities(pair, t0, t1);

        assertEq(res.length, 3);
    }

    function testGetTokensRenzo() public {
        bytes32 pair = bytes32(0);
        address[] memory tokens = adapter.getTokens(pair);

        assertGe(tokens.length, 2);
    }

    function testGetLimitsRenzo() public {
        bytes32 pair = bytes32(0);
        uint256[] memory limits =
            adapter.getLimits(pair, address(wBETH), address(ezETH));

        assertEq(limits.length, 2);
    }

    /**
     * @dev test fails with poolIds[0] =  bytes32(bytes20(address(ETH))) because
     * AdapterTest.sol
     * @dev doesn't handle selling eth
     * @dev And because AdapterTest.sol:76 does not consider handle the ezETH
     * TVL (price can actually be Higher)
     */
    // function testPoolBehaviourRenzo() public {
    //     bytes32[] memory poolIds = adapter.getPoolIds(0, 1000);
    //     runPoolBehaviourTest(adapter, poolIds);
    // }
}
