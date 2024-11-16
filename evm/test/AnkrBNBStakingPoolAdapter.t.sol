// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import "src/ankr-bnb/AnkrBNBStakingPoolAdapter.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";
import "./AdapterTest.sol";

contract AnkrBNBStakingPoolAdapterTest is Test, ISwapAdapterTypes, AdapterTest {
    using FractionMath for Fraction;

    AnkrBNBStakingPoolAdapter adapter;
    ICertificateToken ankrBNB;
    address constant BNB = address(0);
    IAnkrBNBStakingPool constant pool =
        IAnkrBNBStakingPool(0x9e347Af362059bf2E55839002c699F7A5BaFE86E);
    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 34449980;
        vm.createSelectFork(vm.rpcUrl("bsc"), forkBlock);
        adapter = new AnkrBNBStakingPoolAdapter(address(pool));
        (, address ankrBNBAddress_) = pool.getTokens();
        ankrBNB = ICertificateToken(ankrBNBAddress_);

        vm.label(address(adapter), "AnkrBNBStakingPoolAdapter");
        vm.label(address(0), "BNB");
        vm.label(address(ankrBNB), "ankrBNB");
    }

    /// @dev enable receive as ether will be sent to this address, and it is a
    /// contract, to prevent reverts
    receive() external payable {}

    function getMinLimits(
        address sellTokenAddress
    ) internal view returns (uint256[] memory minLimits) {
        minLimits = new uint256[](2);
        address ankrBNBAddress = address(ankrBNB);

        uint256 minAnkrBNBAmount = pool.getMinUnstake();
        if (sellTokenAddress == ankrBNBAddress) {
            minLimits[0] = minAnkrBNBAmount;
            minLimits[1] = ankrBNB.sharesToBonds(minAnkrBNBAmount);
        } else {
            minLimits[0] = ankrBNB.sharesToBonds(minAnkrBNBAmount);
            minLimits[1] = minAnkrBNBAmount;
        }
    }

    function testPriceFuzzAnkr() public {
        uint256[] memory minLimits = getMinLimits(address(BNB));
        uint256 minLimit = minLimits[0];
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        /**
         * @dev as Ankr implements min limits and tests are likely exceeding the
         * 65536 limit of iterations
         * if the amount is in the input(as must check: minLimit < amount <
         * maxLimit),
         * we use 100 iterations to make sure the price is working as expected
         * without triggering foundry errors
         */
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = minLimit + (i * 10 ** 12);
        }

        Fraction[] memory prices = adapter.price(
            bytes32(0),
            BNB,
            address(ankrBNB),
            amounts
        );

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testSwapFuzzAnkrWithAnkrBNB(bool isBuy) public {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        bytes32 pair = bytes32(0);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            uint256[] memory limits = adapter.getLimits(
                pair,
                address(ankrBNB),
                BNB
            );
            uint256[] memory minLimits = getMinLimits(address(ankrBNB));

            uint256 specifiedAmount = side == OrderSide.Buy
                ? minLimits[1]
                : minLimits[0];
            specifiedAmount = specifiedAmount + i;

            if (side == OrderSide.Buy) {
                if (specifiedAmount > limits[1]) {
                    specifiedAmount = limits[1];
                }

                /** 
                 * @dev 
                 * Min. limit is very close to max. limit in this block, therefore at some point if we don't stop the test
                 * It will fail because, e.g.:
                 * Max. limit(BNB) 45231099045423138
                 * Min. limit(BNB) 107007575936415118
                 */
                if(specifiedAmount <= minLimits[1]) {
                    break;
                }

                deal(address(ankrBNB), address(this), type(uint256).max);
                ankrBNB.approve(address(adapter), type(uint256).max);
            } else {
                if (specifiedAmount > limits[0]) {
                    specifiedAmount = limits[0];
                }

                /** 
                 * @dev 
                 * Min. limit is very close to max. limit in this block, therefore at some point if we don't stop the test
                 * It will fail because, e.g.:
                 * Max. limit(BNB) 45231099045423138
                 * Min. limit(BNB) 107007575936415118
                 */
                if(specifiedAmount <= minLimits[0]) {
                    break;
                }

                deal(address(ankrBNB), address(this), specifiedAmount);
                ankrBNB.approve(address(adapter), specifiedAmount);
            }

            uint256 ankrBNB_balance = ankrBNB.balanceOf(address(this));
            uint256 BNB_balance = address(this).balance;

            Trade memory trade = adapter.swap(
                pair,
                address(ankrBNB),
                BNB,
                side,
                specifiedAmount
            );

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
    }

    function testSwapFuzzAnkrWithBNB(bool isBuy) public {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        bytes32 pair = bytes32(0);
        uint256[] memory minLimits = getMinLimits(address(BNB));
        uint256 specifiedAmount = isBuy ? minLimits[1] : minLimits[0];

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            specifiedAmount = specifiedAmount + (i * 10 ** 6);
            if (side == OrderSide.Buy) {
                deal(address(this), 10000 ether);
                (bool sent, ) = address(adapter).call{value: 10000 ether}("");
                /// @dev although send will never fail since contract has
                /// receive() function,
                /// we add the require anyway to hide the "unused local
                /// variable" and "Return value of low-level calls not used"
                /// warnings
                require(sent, "Failed to transfer ether");
            } else {
                deal(address(this), specifiedAmount);
                (bool sent, ) = address(adapter).call{value: specifiedAmount}(
                    ""
                );
                /// @dev although send will never fail since contract has
                /// receive() function,
                /// we add the require anyway to hide the "unused local
                /// variable" and "Return value of low-level calls not used"
                /// warnings
                require(sent, "Failed to transfer ether");
            }

            uint256 ankrBNB_balance = ankrBNB.balanceOf(address(this));
            uint256 BNB_balance = address(this).balance;

            Trade memory trade = adapter.swap(
                pair,
                BNB,
                address(ankrBNB),
                side,
                specifiedAmount
            );

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

    function testSwapSellIncreasingAnkr() public {
        executeIncreasingSwapsAnkr(OrderSide.Sell);
    }

    function executeIncreasingSwapsAnkr(OrderSide side) internal {
        bytes32 pair = bytes32(0);

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        uint256[] memory minLimits = getMinLimits(address(ankrBNB));
        uint256 specifiedAmount = side == OrderSide.Buy
            ? minLimits[1]
            : minLimits[0];

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = specifiedAmount + (i * 10 ** 6);
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(address(ankrBNB), address(this), amounts[i]);
            ankrBNB.approve(address(adapter), amounts[i]);

            trades[i] = adapter.swap(
                pair,
                address(ankrBNB),
                BNB,
                side,
                amounts[i]
            );
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(
                trades[i].calculatedAmount,
                trades[i + 1].calculatedAmount
            );
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
        }
    }

    function testSwapBuyIncreasingAnkr() public {
        executeIncreasingSwapsAnkr(OrderSide.Buy);
    }

    function testGetCapabilitiesAnkr(
        bytes32 pair,
        address t0,
        address t1
    ) public {
        Capability[] memory res = adapter.getCapabilities(pair, t0, t1);

        assertGe(res.length, 4);
    }

    function testGetLimitsAnkr() public {
        bytes32 pair = bytes32(0);
        uint256[] memory limits = adapter.getLimits(
            pair,
            address(ankrBNB),
            BNB
        );

        assertEq(limits.length, 2);
    }

    // This test is currently broken due to a bug in runPoolBehaviour
    // with constant price pools.
    //
    //    function testPoolBehaviourFraxV3Sfrax() public {
    //        bytes32[] memory poolIds = new bytes32[](1);
    //        poolIds[0] = bytes32(0);
    //        runPoolBehaviourTest(adapter, poolIds);
    //    }
}
