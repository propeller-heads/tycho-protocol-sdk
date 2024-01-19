// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import "src/rocketpool/RocketPoolAdapter.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";

contract RocketPoolAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    RocketPoolAdapter adapter;
    RocketTokenRETHInterface rocketETH;
    IERC20 constant ETH = IERC20(address(0));
    RocketStorageInterface constant rocketStorage = RocketStorageInterface(0x1d8f8f00cfa6758d7bE78336684788Fb0ee0Fa46);
    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 19021957;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new
            RocketPoolAdapter(rocketStorage);
        rocketETH = RocketTokenRETHInterface(
            rocketStorage.getAddress(keccak256(abi.encodePacked("contract.address", "rocketTokenRETH")))
        );

        vm.label(address(adapter), "RocketPoolAdapter");
        vm.label(address(0), "ETH");
        vm.label(address(rocketETH), "rocketETH");
    }

    /// @dev enable receive as ether will be sent to this address, and it is a contract, to prevent reverts
    receive() external payable {}

    function getMinLimits(address sellTokenAddress) internal view returns (uint256[] memory minLimits) {
        minLimits = new uint256[](2);
        address rocketETHAddress = address(rocketETH);
        RocketDAOProtocolSettingsDepositInterface rocketDao = RocketDAOProtocolSettingsDepositInterface(
            rocketStorage.getAddress(keccak256(abi.encodePacked("contract.address", "rocketDAOProtocolSettingsDeposit")))
        );

        uint256 minETHAmount = rocketDao.getMinimumDeposit();
        if(sellTokenAddress == rocketETHAddress) {
            minLimits[0] = rocketETH.getRethValue(minETHAmount);
            minLimits[1] = minETHAmount;
        }
        else {
            minLimits[0] = minETHAmount;
            minLimits[1] = rocketETH.getRethValue(minETHAmount);
        }
    }

    function testPriceRocketpool(bool isETH) public {
        uint256[] memory minLimits = getMinLimits(isETH ? address(ETH) : address(rocketETH));
        uint256 minLimit = minLimits[0];
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        for(uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = minLimit + (i * 10**12);
        }

        Fraction[] memory prices = isETH ?
            adapter.price(bytes32(0), ETH, rocketETH, amounts) :
            adapter.price(bytes32(0), rocketETH, ETH, amounts); 

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    // function testSwapFuzzAnkr(uint256 specifiedAmount, bool isBuy) public {
    //     OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

    //     bytes32 pair = bytes32(0);
    //     uint256[] memory limits = adapter.getLimits(pair, ankrBNB, BNB);
    //     uint256[] memory minLimits = getMinLimits(address(ankrBNB));

    //     if (side == OrderSide.Buy) {
    //         vm.assume(specifiedAmount < limits[1] && specifiedAmount > minLimits[1]);

    //         deal(address(ankrBNB), address(this), type(uint256).max);
    //         ankrBNB.approve(address(adapter), type(uint256).max);
    //     } else {
    //         vm.assume(specifiedAmount < limits[0] && specifiedAmount > minLimits[0]);

    //         deal(address(ankrBNB), address(this), specifiedAmount);
    //         ankrBNB.approve(address(adapter), specifiedAmount);
    //     }

    //     uint256 ankrBNB_balance = ankrBNB.balanceOf(address(this));
    //     uint256 BNB_balance = address(this).balance;

    //     Trade memory trade =
    //         adapter.swap(pair, ankrBNB, BNB, side, specifiedAmount);

    //     if (trade.calculatedAmount > 0) {
    //         if (side == OrderSide.Buy) {
    //             assertEq(
    //                 specifiedAmount,
    //                 address(this).balance - BNB_balance
    //             );
    //             assertEq(
    //                 trade.calculatedAmount,
    //                 ankrBNB_balance - ankrBNB.balanceOf(address(this))
    //             );
    //         } else {
    //             assertEq(
    //                 specifiedAmount,
    //                 ankrBNB_balance - ankrBNB.balanceOf(address(this))
    //             );
    //             assertEq(
    //                 trade.calculatedAmount,
    //                 address(this).balance - BNB_balance
    //             );
    //         }
    //     }
    // }

    // function testSwapFuzzAnkrWithBNB(bool isBuy) public {
    //     OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

    //     bytes32 pair = bytes32(0);
    //     uint256[] memory minLimits = getMinLimits(address(BNB));
    //     uint256 specifiedAmount = isBuy ? minLimits[1] : minLimits[0];

    //     for(uint256 i = 0; i < TEST_ITERATIONS; i++) {
    //         specifiedAmount = specifiedAmount + (i * 10**6);
    //         if (side == OrderSide.Buy) {
    //             deal(address(this), 10000 ether);
    //             (bool sent, ) = address(adapter).call{value: 10000 ether}("");
    //             /// @dev although send will never fail since contract has receive() function,
    //             /// we add the require anyway to hide the "unused local variable" and "Return value of low-level calls not used" warnings 
    //             require(sent, "Failed to transfer ether");
    //         } else {
    //             deal(address(this), specifiedAmount);
    //             (bool sent, ) = address(adapter).call{value: specifiedAmount}("");
    //             /// @dev although send will never fail since contract has receive() function,
    //             /// we add the require anyway to hide the "unused local variable" and "Return value of low-level calls not used" warnings
    //             require(sent, "Failed to transfer ether");
    //         }

    //         uint256 ankrBNB_balance = ankrBNB.balanceOf(address(this));
    //         uint256 BNB_balance = address(this).balance;

    //         Trade memory trade =
    //             adapter.swap(pair, BNB, ankrBNB, side, specifiedAmount);

    //         if (trade.calculatedAmount > 0) {
    //             if (side == OrderSide.Buy) {
    //                 assertEq(
    //                     specifiedAmount,
    //                     ankrBNB_balance - ankrBNB.balanceOf(address(this))
    //                 );
    //                 assertEq(
    //                     trade.calculatedAmount,
    //                     address(this).balance - BNB_balance
    //                 );
    //             } else {
    //                 assertEq(
    //                     specifiedAmount,
    //                     address(this).balance - BNB_balance
    //                 );
    //                 assertEq(
    //                     trade.calculatedAmount,
    //                     ankrBNB_balance - ankrBNB.balanceOf(address(this))
    //                 );
    //             }
    //         }
    //     }
    // }

    // function testSwapSellIncreasingAnkr() public {
    //     executeIncreasingSwapsAnkr(OrderSide.Sell);
    // }

    // function executeIncreasingSwapsAnkr(OrderSide side) internal {
    //     bytes32 pair = bytes32(0);

    //     uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
    //     uint256[] memory minLimits = getMinLimits(address(ankrBNB));
    //     uint256 specifiedAmount = side == OrderSide.Buy ? minLimits[1] : minLimits[0];

    //     for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
    //         amounts[i] = specifiedAmount + (i * 10 ** 6);
    //     }

    //     Trade[] memory trades = new Trade[](TEST_ITERATIONS);
    //     uint256 beforeSwap;
    //     for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
    //         beforeSwap = vm.snapshot();

    //         deal(address(ankrBNB), address(this), amounts[i]);
    //         ankrBNB.approve(address(adapter), amounts[i]);

    //         trades[i] = adapter.swap(pair, ankrBNB, BNB, side, amounts[i]);
    //         vm.revertTo(beforeSwap);
    //     }

    //     for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
    //         assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
    //         assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
    //     }
    // }

    // function testSwapBuyIncreasingAnkr() public {
    //     executeIncreasingSwapsAnkr(OrderSide.Buy);
    // }

    // function testGetCapabilitiesAnkr(bytes32 pair, address t0, address t1) public {
    //     Capability[] memory res =
    //         adapter.getCapabilities(pair, IERC20(t0), IERC20(t1));

    //     assertEq(res.length, 3);
    // }

    // function testGetLimitsAnkr() public {
    //     bytes32 pair = bytes32(0);
    //     uint256[] memory limits = adapter.getLimits(pair, IERC20(address(ankrBNB)), BNB);

    //     assertEq(limits.length, 2);
    // }
}
