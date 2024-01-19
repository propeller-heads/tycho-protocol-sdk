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
    IAnkrBNBStakingPool constant pool = IAnkrBNBStakingPool(0x9e347Af362059bf2E55839002c699F7A5BaFE86E);
    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 34449980;
        vm.createSelectFork(vm.rpcUrl("bsc"), forkBlock);
        adapter = new
            AnkrBNBStakingPoolAdapter(pool);

        vm.label(address(adapter), "AnkrBNBStakingPoolAdapter");
        vm.label(address(0), "BNB");
        vm.label(address(ankrBNB), "ankrBNB");
    }

    /// @dev enable receive as ether will be sent to this address, and it is a contract, to prevent reverts
    receive() external payable {}

    function getMinLimits(address sellTokenAddress) internal view returns (uint256[] memory minLimits) {
        minLimits = new uint256[](2);
        address ankrBNBAddress = address(ankrBNB);

        uint256 minBNBAmount = pool.getMinUnstake();
        if(sellTokenAddress == ankrBNBAddress) {
            minLimits[0] = ankrBNB.bondsToShares(minBNBAmount);
            minLimits[1] = minBNBAmount;
        }
        else {
            minLimits[0] = minBNBAmount;
            minLimits[1] = ankrBNB.bondsToShares(minBNBAmount);
        }
    }

    function testPriceFuzzAnkr() public {
        uint256[] memory minLimits = getMinLimits(address(BNB));
        uint256 minLimit = minLimits[0];
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        /**
         * @dev as Ankr implements min limits and tests are likely exceeding the 65536 limit of iterations
         * if the amount is in the input(as must check: minLimit < amount < maxLimit),
         * we use 100 iterations to make sure the price is working as expected without triggering foundry errors
         */
        for(uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = minLimit + (i * 10**12);
        }

        Fraction[] memory prices = adapter.price(bytes32(0), BNB, ankrBNB, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testSwapFuzzAnkr(uint256 specifiedAmount, bool isBuy) public {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        bytes32 pair = bytes32(0);
        uint256[] memory limits = adapter.getLimits(pair, ankrBNB, BNB);
        uint256[] memory minLimits = getMinLimits(address(ankrBNB));

        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1] && specifiedAmount > minLimits[1]);

            deal(address(ankrBNB), address(this), type(uint256).max);
            ankrBNB.approve(address(adapter), type(uint256).max);
        } else {
            vm.assume(specifiedAmount < limits[0] && specifiedAmount > minLimits[0]);

            deal(address(ankrBNB), address(this), specifiedAmount);
            ankrBNB.approve(address(adapter), specifiedAmount);
        }

        uint256 ankrBNB_balance = ankrBNB.balanceOf(address(this));
        uint256 BNB_balance = address(this).balance;

        Trade memory trade =
            adapter.swap(pair, ankrBNB, BNB, side, specifiedAmount);

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    address(this).balance - BNB_balance
                );
                assertEq(
                    trade.calculatedAmount,
                    ankrBNB_balance - ankrBNB.balanceOf(address(this))
                );
            } else {
                assertEq(
                    specifiedAmount,
                    ankrBNB_balance - ankrBNB.balanceOf(address(this))
                );
                assertEq(
                    trade.calculatedAmount,
                    address(this).balance - BNB_balance
                );
            }
        }
    }

    function testSwapFuzzAnkrWithBNB() public {
        bool isBuy = false;
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        bytes32 pair = bytes32(0);
        uint256[] memory limits = adapter.getLimits(pair, BNB, ankrBNB);
        uint256[] memory minLimits = getMinLimits(address(BNB));
        uint256 specifiedAmount = isBuy ? minLimits[1] : minLimits[0];

        for(uint256 i = 0; i < TEST_ITERATIONS; i++) {
            if (side == OrderSide.Buy) {
                deal(address(this), 10000 ether);
                address(adapter).call{value: 1000 ether}("");
            } else {
                deal(address(this), specifiedAmount);
                address(adapter).call{value: specifiedAmount}("");
            }

            uint256 ankrBNB_balance = ankrBNB.balanceOf(address(this));
            uint256 BNB_balance = address(this).balance;

            Trade memory trade =
                adapter.swap(pair, BNB, ankrBNB, side, specifiedAmount);

            if (trade.calculatedAmount > 0) {
                if (side == OrderSide.Buy) {
                    assertEq(
                        specifiedAmount,
                        ankrBNB_balance - ankrBNB.balanceOf(address(this))
                    );
                    assertEq(
                        trade.calculatedAmount,
                        address(this).balance - BNB_balance
                    );
                } else {
                    assertEq(
                        specifiedAmount,
                        address(this).balance - BNB_balance
                    );
                    assertEq(
                        trade.calculatedAmount,
                        ankrBNB_balance - ankrBNB.balanceOf(address(this))
                    );
                }
            }
        }
    }

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

    function testGetCapabilitiesAnkr(bytes32 pair, address t0, address t1) public {
        Capability[] memory res =
            adapter.getCapabilities(pair, IERC20(t0), IERC20(t1));

        assertEq(res.length, 3);
    }

    function testGetLimitsAnkr() public {
        bytes32 pair = bytes32(0);
        uint256[] memory limits = adapter.getLimits(pair, IERC20(address(ankrBNB)), BNB);

        assertEq(limits.length, 2);
    }
}
