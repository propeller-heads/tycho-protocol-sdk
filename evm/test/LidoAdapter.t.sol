// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import "src/lido/LidoAdapter.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";

contract LidoAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    LidoAdapter adapter;
    IwstETH constant wstETH = IwstETH(0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0);
    IStETH stETH;
    IERC20 constant ETH = IERC20(address(0));
    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 19011957;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new
            LidoAdapter(wstETH);
        stETH = wstETH.stETH();

        vm.label(address(adapter), "LidoAdapter");
        vm.label(address(0), "ETH");
        vm.label(address(wstETH), "wstETH");
        vm.label(address(stETH), "stETH");
    }

    /// @dev enable receive as ether will be sent to this address, and it is a contract, to prevent reverts
    receive() external payable {}

    // function getMinLimits(address sellTokenAddress) internal view returns (uint256[] memory minLimits) {
    //     minLimits = new uint256[](2);
    //     address rocketETHAddress = address(rocketETH);
    //     RocketDAOProtocolSettingsDepositInterface rocketDao = RocketDAOProtocolSettingsDepositInterface(
    //         rocketStorage.getAddress(keccak256(abi.encodePacked("contract.address", "rocketDAOProtocolSettingsDeposit")))
    //     );

    //     uint256 minETHAmount = rocketDao.getMinimumDeposit();
    //     if(sellTokenAddress == rocketETHAddress) {
    //         minLimits[0] = rocketETH.getRethValue(minETHAmount);
    //         minLimits[1] = minETHAmount;
    //     }
    //     else {
    //         minLimits[0] = minETHAmount;
    //         minLimits[1] = rocketETH.getRethValue(minETHAmount);
    //     }
    // }

    /// @dev custom function to mint stETH tokens as normal "deal" will revert due to stETH internal functions
    /// (ref: StETH.sol:375)
    /// @dev because of internal precision losses in Lido contracts, we unwrap the final amount - 1 to prevent overflows,
    /// although the final amounts are the expected since solidity handles it correctly
    /// throughout the contract calls and operations. Indeed the asserts are met in swap functions.
    /// (ref: LidoAdapter.t.sol:196)
    function dealStEthTokens(uint256 amount) internal {
        uint256 wstETHAmount = wstETH.getStETHByWstETH(amount);
        deal(address(wstETH), address(this), wstETHAmount);
        wstETH.unwrap(wstETHAmount);
    }

    function testPriceLidoSteth() public {
        Fraction[] memory prices = new Fraction[](2);

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for(uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = (0.001 ether) + (i * 10**14);
        }

        // stETH-ETH
        prices = adapter.price(
            bytes32(0),
            stETH,
            ETH,
            amounts
        );
        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }

        // stETH-wstETH
        prices = adapter.price(
            bytes32(0),
            stETH,
            wstETH,
            amounts
        );
        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testPriceLidoETH() public {
        Fraction[] memory prices = new Fraction[](2);

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for(uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = (0.001 ether) + (i * 10**14);
        }

        // ETH-stETH
        prices = adapter.price(
            bytes32(0),
            ETH,
            stETH,
            amounts
        );
        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }

        // ETH-wstETH
        prices = adapter.price(
            bytes32(0),
            ETH,
            wstETH,
            amounts
        );
        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testPriceLidoWsteth() public {
        Fraction[] memory prices = new Fraction[](2);

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for(uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = (0.001 ether) + (i * 10**14);
        }

        // wstETH-ETH
        prices = adapter.price(
            bytes32(0),
            wstETH,
            ETH,
            amounts
        );
        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }

        // wstETH-stETH
        prices = adapter.price(
            bytes32(0),
            wstETH,
            stETH,
            amounts
        );
        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testSwapFuzzLidoStEth(uint256 specifiedAmount, bool isBuy) public {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        vm.assume(specifiedAmount > 10);

        bytes32 pair = bytes32(0);

        uint256[] memory limits = adapter.getLimits(pair, stETH, wstETH);
        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1]);

            dealStEthTokens(wstETH.getStETHByWstETH(specifiedAmount));
            stETH.approve(address(adapter), type(uint256).max);
        } else {
            vm.assume(specifiedAmount < limits[0]);

            dealStEthTokens(specifiedAmount);
            stETH.approve(address(adapter), specifiedAmount);
        }
        uint256 stETH_balance = stETH.balanceOf(address(this));
        uint256 wstETH_balance = wstETH.balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, stETH, wstETH, side, specifiedAmount);

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    wstETH.balanceOf(address(this)) - wstETH_balance
                );
                assertEq(
                    trade.calculatedAmount,
                    stETH_balance - stETH.balanceOf(address(this))
                );
            } else {
                assertEq(
                    specifiedAmount,
                    stETH_balance - stETH.balanceOf(address(this))
                );
                assertEq(
                    trade.calculatedAmount,
                    wstETH.balanceOf(address(this)) - wstETH_balance
                );
            }
        }
    }

    function testSwapFuzzLidoWstEth(uint256 specifiedAmount, bool isBuy) public {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        vm.assume(specifiedAmount > 10);

        bytes32 pair = bytes32(0);

        uint256[] memory limits = adapter.getLimits(pair, wstETH, stETH);
        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1]);

            deal(address(wstETH), address(this), wstETH.getWstETHByStETH(specifiedAmount));
            wstETH.approve(address(adapter), type(uint256).max);
        } else {
            vm.assume(specifiedAmount < limits[0]);

            deal(address(wstETH), address(this), specifiedAmount);
            wstETH.approve(address(adapter), specifiedAmount);
        }
        uint256 stETH_balance = stETH.balanceOf(address(this));
        uint256 wstETH_balance = wstETH.balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, wstETH, stETH, side, specifiedAmount);

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    stETH_balance - stETH.balanceOf(address(this))
                );
                assertEq(
                    trade.calculatedAmount,
                    wstETH.balanceOf(address(this)) - wstETH_balance
                );
            } else {
                assertEq(
                    specifiedAmount,
                    wstETH.balanceOf(address(this)) - wstETH_balance
                );
                assertEq(
                    trade.calculatedAmount,
                    stETH_balance - stETH.balanceOf(address(this))
                );
            }
        }
    }

    function testSwapFuzzLidoEth(uint256 specifiedAmount, bool isBuy) public {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        vm.assume(specifiedAmount > 10);

        bytes32 pair = bytes32(0);

        // ETH-stETH
        uint256[] memory limits = adapter.getLimits(pair, ETH, stETH);
        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1]);

            uint256 ethAmount = stETH.getPooledEthByShares(specifiedAmount);
            deal(address(this), ethAmount);
            (bool sent_, ) = address(adapter).call{value: ethAmount}("");
            if(!sent_) { revert(); } // hide warnings
        } else {
            vm.assume(specifiedAmount < limits[0]);

            deal(address(this), specifiedAmount);
            (bool sent_, ) = address(adapter).call{value: specifiedAmount}("");
            if(!sent_) { revert(); } // hide warnings
        }
        uint256 stETH_balance = stETH.balanceOf(address(this));
        uint256 ETH_balance = address(this).balance;
        Trade memory trade =
            adapter.swap(pair, ETH, stETH, side, specifiedAmount);
        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    stETH_balance - stETH.balanceOf(address(this))
                );
                assertEq(
                    trade.calculatedAmount,
                    address(this).balance - ETH_balance
                );
            } else {
                assertEq(
                    specifiedAmount,
                    address(this).balance - ETH_balance
                );
                assertEq(
                    trade.calculatedAmount,
                    stETH_balance - stETH.balanceOf(address(this))
                );
            }
        }

        // ETH-wstETH
        limits = adapter.getLimits(pair, ETH, wstETH);
        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1]);

            uint256 stETHAmount = wstETH.getStETHByWstETH(specifiedAmount);
            uint256 ethAmount = stETH.getPooledEthByShares(stETHAmount);
            deal(address(this), ethAmount);
            (bool sent_, ) = address(adapter).call{value: ethAmount}("");
            if(!sent_) { revert(); } // hide warnings
        } else {
            vm.assume(specifiedAmount < limits[0]);

            deal(address(this), specifiedAmount);
            (bool sent_, ) = address(adapter).call{value: specifiedAmount}("");
            if(!sent_) { revert(); } // hide warnings
        }
        uint256 wstETH_balance = wstETH.balanceOf(address(this));
        ETH_balance = address(this).balance;
        trade =
            adapter.swap(pair, ETH, wstETH, side, specifiedAmount);
        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    wstETH_balance - wstETH.balanceOf(address(this))
                );
                assertEq(
                    trade.calculatedAmount,
                    address(this).balance - ETH_balance
                );
            } else {
                assertEq(
                    specifiedAmount,
                    address(this).balance - ETH_balance
                );
                assertEq(
                    trade.calculatedAmount,
                    wstETH_balance - wstETH.balanceOf(address(this))
                );
            }
        }
    }

    // function testSwapFuzzRocketpoolWithETH(uint256 specifiedAmount, bool isBuy) public {
    //     OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

    //     bytes32 pair = bytes32(0);
    //     uint256[] memory limits = adapter.getLimits(bytes32(0), ETH, rocketETH);
    //     uint256[] memory minLimits = getMinLimits(address(ETH));
 
    //     if (side == OrderSide.Buy) {
    //         vm.assume(specifiedAmount < limits[1] && specifiedAmount > minLimits[1]);

    //         deal(address(this), 10000 ether);
    //         (bool sent, ) = address(adapter).call{value: 10000 ether}("");
    //         /// @dev although send will never fail since contract has receive() function,
    //         /// we add the require anyway to hide the "unused local variable" and "Return value of low-level calls not used" warnings 
    //         require(sent, "Failed to transfer ether");
    //     } else {
    //         vm.assume(specifiedAmount < limits[0] && specifiedAmount > minLimits[0]);

    //         deal(address(this), specifiedAmount);
    //         (bool sent, ) = address(adapter).call{value: specifiedAmount}("");
    //         /// @dev although send will never fail since contract has receive() function,
    //         /// we add the require anyway to hide the "unused local variable" and "Return value of low-level calls not used" warnings
    //         require(sent, "Failed to transfer ether");
    //     }

    //     uint256 rocketETH_balance = rocketETH.balanceOf(address(this));
    //     uint256 ETH_balance = address(this).balance;

    //     Trade memory trade =
    //         adapter.swap(pair, ETH, rocketETH, side, specifiedAmount);

    //     if (trade.calculatedAmount > 0) {
    //         if (side == OrderSide.Buy) {
    //             assertEq(
    //                 specifiedAmount,
    //                 rocketETH_balance - rocketETH.balanceOf(address(this))
    //             );
    //             assertEq(
    //                 trade.calculatedAmount,
    //                 address(this).balance - ETH_balance
    //             );
    //         } else {
    //             assertEq(
    //                 specifiedAmount,
    //                 address(this).balance - ETH_balance
    //             );
    //             assertEq(
    //                 trade.calculatedAmount,
    //                 rocketETH_balance - rocketETH.balanceOf(address(this))
    //             );
    //         }
    //     }
    // }

    // function testSwapSellIncreasingRocketpool() public {
    //     executeIncreasingSwapsRocketpool(OrderSide.Sell);
    // }

    // function executeIncreasingSwapsRocketpool(OrderSide side) internal {
    //     bytes32 pair = bytes32(0);

    //     uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
    //     uint256[] memory minLimits = getMinLimits(address(rocketETH));
    //     uint256 specifiedAmount = side == OrderSide.Buy ? minLimits[1] : minLimits[0];

    //     for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
    //         amounts[i] = specifiedAmount + (i * 10 ** 6);
    //     }

    //     Trade[] memory trades = new Trade[](TEST_ITERATIONS);
    //     uint256 beforeSwap;
    //     for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
    //         beforeSwap = vm.snapshot();

    //         deal(address(rocketETH), address(this), amounts[i]);
    //         rocketETH.approve(address(adapter), amounts[i]);

    //         trades[i] = adapter.swap(pair, rocketETH, ETH, side, amounts[i]);
    //         vm.revertTo(beforeSwap);
    //     }

    //     for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
    //         assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
    //         assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
    //     }
    // }

    // function testSwapBuyIncreasingRocketpool() public {
    //     executeIncreasingSwapsRocketpool(OrderSide.Buy);
    // }

    // function testGetCapabilitiesRocketpool(bytes32 pair, address t0, address t1) public {
    //     Capability[] memory res =
    //         adapter.getCapabilities(pair, IERC20(t0), IERC20(t1));

    //     assertEq(res.length, 3);
    // }

    // function testGetLimitsRocketpool() public {
    //     bytes32 pair = bytes32(0);
    //     uint256[] memory limits = adapter.getLimits(pair, IERC20(address(rocketETH)), ETH);

    //     assertEq(limits.length, 2);
    // }
}
