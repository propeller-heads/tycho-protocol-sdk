// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";
import "src/stakewise/StakeWiseAdapter.sol";

contract StakeWiseAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    StakeWiseAdapter adapter;
    IERC20 osEth = IERC20(0xf1C9acDc66974dFB6dEcB12aA385b9cD01190E38);

    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 19862102;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new StakeWiseAdapter(0xe6d8d8aC54461b1C5eD15740EEe322043F696C08);

        vm.label(address(osEth), "osEth");
        vm.label(address(adapter), "StakeWiseAdapter");
    }

    receive() external payable {}

    function testPriceFuzzStakeWise(uint256 amount0, uint256 amount1) public {
        bytes32 pair = bytes32(0);
        uint256[] memory limits = adapter.getLimits(
            pair, address(osEth), address(0)
        );
        vm.assume(amount0 < limits[0] && amount0 > 0);
        vm.assume(amount1 < limits[1] && amount1 > 0);

        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;

        Fraction[] memory prices = adapter.price(
            pair, address(osEth), address(0), amounts
        );

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testSwapFuzzStakewiseEth(uint256 specifiedAmount, bool isBuy)
        public
    {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        bytes32 pair = bytes32(0);
        uint256[] memory limits =
            adapter.getLimits(pair, address(0), address(osEth));

        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1] && specifiedAmount > 10);

            /// @dev workaround for eETH "deal", as standard ERC20 does not
            /// work(balance is shares)
            deal(address(osEth), address(adapter), type(uint256).max);
            osEth.approve(address(adapter), type(uint256).max);
        } else {
            vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10);

            deal(address(osEth), address(adapter), type(uint256).max);
            osEth.approve(address(adapter), specifiedAmount);
        }

        uint256 eth_balance_before = address(this).balance;
        uint256 osEth_balance_before = osEth.balanceOf(address(this));

        Trade memory trade = adapter.swap(
            pair, address(osEth), address(0), side, specifiedAmount
        );

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    osEth.balanceOf(address(this)) - osEth_balance_before
                );

                assertEq(
                    trade.calculatedAmount,
                    eth_balance_before - address(this).balance
                );
            } else {
                assertEq(
                    specifiedAmount,
                    eth_balance_before - address(this).balance
                );

                assertEq(
                    trade.calculatedAmount,
                    osEth.balanceOf(address(this)) - osEth_balance_before
                );
            }
        }
    }

    // function testSwapFuzzStakeWiseosEthEeth(uint256 specifiedAmount, bool isBuy)
    //     public
    // {
    //     OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

    //     IERC20 eEth_ = IERC20(address(eEth));
    //     IERC20 osEth_ = IERC20(address(osEth));
    //     uint256 osEth_bal_before = osEth_.balanceOf(address(this));
    //     bytes32 pair = bytes32(0);
    //     uint256[] memory limits =
    //         adapter.getLimits(pair, address(osEth_), address(eEth_));

    //     if (side == OrderSide.Buy) {
    //         vm.assume(specifiedAmount < limits[1] && specifiedAmount > 100);

    //         /// @dev workaround for eETH "deal", as standard ERC20 does not
    //         /// work(balance is shares)
    //         deal(address(adapter), type(uint256).max);
    //         adapter.swap(
    //             pair,
    //             address(address(0)),
    //             address(osEth_),
    //             OrderSide.Buy,
    //             limits[0]
    //         );

    //         osEth_.approve(address(adapter), type(uint256).max);
    //     } else {
    //         vm.assume(specifiedAmount < limits[0] && specifiedAmount > 100);

    //         /// @dev workaround for eETH "deal", as standard ERC20 does not
    //         /// work(balance is shares)
    //         deal(address(adapter), type(uint128).max);
    //         adapter.swap(
    //             pair,
    //             address(address(0)),
    //             address(osEth_),
    //             OrderSide.Buy,
    //             specifiedAmount
    //         );

    //         osEth_.approve(address(adapter), specifiedAmount);
    //     }

    //     uint256 eEth_balance = eEth_.balanceOf(address(this));
    //     uint256 osEth_balance = osEth_.balanceOf(address(this));

    //     /// @dev as of rounding errors in StakeWise, specifiedAmount might lose
    //     /// small digits for small numbers
    //     /// therefore we use osEth_balance - osEth_bal_before as specifiedAmount
    //     uint256 realAmountosEth_ = osEth_balance - osEth_bal_before;

    //     Trade memory trade = adapter.swap(
    //         pair, address(osEth_), address(eEth_), side, realAmountosEth_
    //     );

    //     if (trade.calculatedAmount > 0) {
    //         if (side == OrderSide.Buy) {
    //             assertGe(
    //                 realAmountosEth_,
    //                 eEth_.balanceOf(address(this)) - eEth_balance
    //             );
    //             /// @dev Transfer function contains rounding errors because of
    //             /// rewards in osEth contract, therefore we assume a +/-2
    //             /// tolerance
    //             assertLe(
    //                 realAmountosEth_ - 2,
    //                 eEth_.balanceOf(address(this)) - eEth_balance
    //             );
    //             assertLe(
    //                 trade.calculatedAmount - 2,
    //                 osEth_balance - osEth_.balanceOf(address(this))
    //             );
    //         } else {
    //             assertEq(
    //                 realAmountosEth_,
    //                 osEth_balance - osEth_.balanceOf(address(this))
    //             );
    //             assertLe(
    //                 trade.calculatedAmount - 2,
    //                 eEth_.balanceOf(address(this)) - eEth_balance
    //             );
    //             assertGe(
    //                 trade.calculatedAmount,
    //                 eEth_.balanceOf(address(this)) - eEth_balance
    //             );
    //         }
    //     }
    // }

    // function testSwapFuzzStakeWiseEthEeth(uint256 specifiedAmount, bool isBuy)
    //     public
    // {
    //     OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

    //     address eth_ = address(0);
    //     IERC20 eEth_ = IERC20(address(eEth));
    //     bytes32 pair = bytes32(0);
    //     uint256[] memory limits = adapter.getLimits(pair, eth_, address(eEth_));

    //     if (side == OrderSide.Buy) {
    //         vm.assume(specifiedAmount < limits[1] && specifiedAmount > 10);

    //         deal(address(adapter), eEth_.totalSupply());
    //     } else {
    //         vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10);

    //         deal(address(adapter), specifiedAmount);
    //     }

    //     uint256 eth_balance = address(adapter).balance;
    //     uint256 eEth_balance = eEth_.balanceOf(address(this));

    //     Trade memory trade =
    //         adapter.swap(pair, eth_, address(eEth_), side, specifiedAmount);

    //     if (trade.calculatedAmount > 0) {
    //         if (side == OrderSide.Buy) {
    //             assertGe(
    //                 specifiedAmount,
    //                 eEth_.balanceOf(address(this)) - eEth_balance
    //             );
    //             /// @dev Transfer function contains rounding errors because of
    //             /// rewards in eETH contract, therefore we assume a +/-2
    //             /// tolerance
    //             assertLe(
    //                 specifiedAmount - 2,
    //                 eEth_.balanceOf(address(this)) - eEth_balance
    //             );
    //             assertEq(
    //                 trade.calculatedAmount,
    //                 eth_balance - address(adapter).balance
    //             );
    //         } else {
    //             assertEq(
    //                 specifiedAmount, eth_balance - address(adapter).balance
    //             );
    //             assertEq(
    //                 trade.calculatedAmount,
    //                 eEth_.balanceOf(address(this)) - eEth_balance
    //             );
    //         }
    //     }
    // }

    // function testSwapFuzzStakeWiseEthosEth(uint256 specifiedAmount, bool isBuy)
    //     public
    // {
    //     OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

    //     address eth_ = address(0);
    //     IERC20 osEth_ = IERC20(address(osEth));
    //     bytes32 pair = bytes32(0);
    //     uint256[] memory limits = adapter.getLimits(pair, eth_, address(osEth_));

    //     if (side == OrderSide.Buy) {
    //         vm.assume(specifiedAmount < limits[1] && specifiedAmount > 10);

    //         deal(address(adapter), osEth_.totalSupply());
    //     } else {
    //         vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10);

    //         deal(address(adapter), specifiedAmount);
    //     }

    //     uint256 eth_balance = address(adapter).balance;
    //     uint256 osEth_balance = osEth_.balanceOf(address(this));

    //     Trade memory trade =
    //         adapter.swap(pair, eth_, address(osEth_), side, specifiedAmount);

    //     if (trade.calculatedAmount > 0) {
    //         if (side == OrderSide.Buy) {
    //             assertGe(
    //                 specifiedAmount,
    //                 osEth_.balanceOf(address(this)) - osEth_balance
    //             );
    //             /// @dev Transfer function contains rounding errors because of
    //             /// rewards in eETH contract, therefore we assume a +/-2
    //             /// tolerance
    //             assertLe(
    //                 specifiedAmount - 2,
    //                 osEth_.balanceOf(address(this)) - osEth_balance
    //             );
    //             assertEq(
    //                 trade.calculatedAmount,
    //                 eth_balance - address(adapter).balance
    //             );
    //         } else {
    //             assertEq(
    //                 specifiedAmount, eth_balance - address(adapter).balance
    //             );
    //             assertEq(
    //                 trade.calculatedAmount,
    //                 osEth_.balanceOf(address(this)) - osEth_balance
    //             );
    //         }
    //     }
    // }

    // function testSwapSellIncreasingStakeWise() public {
    //     executeIncreasingSwapsStakeWise(OrderSide.Sell);
    // }

    // function testSwapBuyIncreasingStakeWise() public {
    //     executeIncreasingSwapsStakeWise(OrderSide.Buy);
    // }

    // function executeIncreasingSwapsStakeWise(OrderSide side) internal {
    //     bytes32 pair = bytes32(0);

    //     uint256 amountConstant_ = 10 ** 18;

    //     uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
    //     amounts[0] = amountConstant_;
    //     for (uint256 i = 1; i < TEST_ITERATIONS; i++) {
    //         amounts[i] = amountConstant_ * i;
    //     }

    //     Trade[] memory trades = new Trade[](TEST_ITERATIONS);
    //     uint256 beforeSwap;
    //     for (uint256 i = 1; i < TEST_ITERATIONS; i++) {
    //         beforeSwap = vm.snapshot();

    //         deal(address(osEth), address(this), amounts[i]);
    //         IERC20(address(osEth)).approve(address(adapter), amounts[i]);

    //         trades[i] = adapter.swap(
    //             pair,
    //             address(address(osEth)),
    //             address(address(eEth)),
    //             side,
    //             amounts[i]
    //         );
    //         vm.revertTo(beforeSwap);
    //     }

    //     for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
    //         assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
    //         assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
    //     }
    // }

    // function testGetCapabilitiesStakeWise(bytes32 pair, address t0, address t1)
    //     public
    // {
    //     Capability[] memory res =
    //         adapter.getCapabilities(pair, address(t0), address(t1));

    //     assertEq(res.length, 3);
    // }

    // function testGetTokensStakeWise() public {
    //     bytes32 pair = bytes32(0);
    //     address[] memory tokens = adapter.getTokens(pair);

    //     assertEq(tokens.length, 3);
    // }

    // function testGetLimitsStakeWise() public {
    //     bytes32 pair = bytes32(0);
    //     uint256[] memory limits =
    //         adapter.getLimits(pair, address(eEth), address(osEth));

    //     assertEq(limits.length, 2);
    // }
}
