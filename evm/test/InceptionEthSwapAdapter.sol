// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/inception-eth/InceptionEthSwapAdapter.sol";
import "src/libraries/FractionMath.sol";

/// @title InceptionEthSwapAdapterTest
contract InceptionEthSwapAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    InceptionEthSwapAdapter adapter;

    address constant INETH = 0xf073bAC22DAb7FaF4a3Dd6c6189a70D54110525C;

    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 20000000;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new InceptionEthSwapAdapter(
            IInceptionPool(0x46199cAa0e453971cedf97f926368d9E5415831a),
            IInceptionToken(0xf073bAC22DAb7FaF4a3Dd6c6189a70D54110525C)
        );

        vm.label(address(adapter), "InceptionEthSwapAdapter");
        vm.label(INETH, "INETH");
    }

    function testPriceFuzzInception(uint256 amount0, uint256 amount1) public view {
        bytes32 pair = bytes32(bytes20(address(adapter)));

        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;

        Fraction[] memory prices = adapter.price(pair, address(0), INETH, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGe(prices[i].numerator, 0);
            assertGe(prices[i].denominator, 0);
        }
    }

    function testSwapFuzz(uint256 specifiedAmount, bool) public {
        bytes32 pair = bytes32(bytes20(address(adapter)));

        uint256[] memory limits = adapter.getLimits(pair, address(0), INETH);
        vm.assume(specifiedAmount > limits[0]);
        vm.assume(specifiedAmount < limits[1]);
        vm.assume(specifiedAmount < adapter.availableToStake());

        (bool success, ) = address(adapter).call{value: specifiedAmount}("");
        require(success);
        uint256 eth_balance = address(adapter).balance;
        uint256 ineth_balance = IERC20(INETH).balanceOf(address(adapter));

        Trade memory trade =
            adapter.swap(pair, address(0), INETH, OrderSide.Sell, specifiedAmount);

        if (trade.calculatedAmount > 0) {
            assertApproxEqAbs(
                specifiedAmount,
                eth_balance - address(adapter).balance,
                2
            );
            assertApproxEqAbs(
                trade.calculatedAmount,
                IERC20(INETH).balanceOf(address(adapter)) - ineth_balance,
                2
            );
            assertLe(trade.gasUsed, 130000);
        }
    }
}

interface ILidoEth {
    function submit(address) external payable;
  }
