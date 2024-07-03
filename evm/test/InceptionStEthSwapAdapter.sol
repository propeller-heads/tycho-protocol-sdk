// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/inception-steth/InceptionStEthSwapAdapter.sol";
import "src/libraries/FractionMath.sol";

/// @title InceptionStEthSwapAdapterTest
contract InceptionStEthSwapAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    InceptionStEthSwapAdapter adapter;

    address constant STETH = 0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84;
    address constant INSTETH = 0x7FA768E035F956c41d6aeaa3Bd857e7E5141CAd5;

    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 20000000;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new InceptionStEthSwapAdapter(
            IInceptionVault(0x814CC6B8fd2555845541FB843f37418b05977d8d)
        );

        vm.label(address(adapter), "InceptionStEthSwapAdapter");
        vm.label(STETH, "STETH");
        vm.label(INSTETH, "INSTETH");
    }

    function testPriceFuzzInception(uint256 amount0, uint256 amount1) public view {
        bytes32 pair = bytes32(bytes20(address(adapter)));

        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;

        Fraction[] memory prices = adapter.price(pair, STETH, INSTETH, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGe(prices[i].numerator, 0);
            assertGe(prices[i].denominator, 0);
        }
    }

    function testSwapFuzz(uint256 specifiedAmount, bool) public {
        bytes32 pair = bytes32(bytes20(address(adapter)));

        uint256[] memory limits = adapter.getLimits(pair, STETH, INSTETH);
        vm.assume(specifiedAmount > limits[0]);
        vm.assume(specifiedAmount < limits[1]);

        //@dev deal func doesn't seem to work for stETH token
        ILidoEth(STETH).submit{value: specifiedAmount}(address(0));
        IERC20(STETH).transfer(address(adapter), specifiedAmount);

        uint256 steth_balance = IERC20(STETH).balanceOf(address(adapter));
        uint256 insteth_balance = IERC20(INSTETH).balanceOf(address(adapter));

        Trade memory trade =
            adapter.swap(pair, STETH, INSTETH, OrderSide.Sell, specifiedAmount);

        if (trade.calculatedAmount > 0) {
            assertApproxEqAbs(
                specifiedAmount,
                steth_balance - IERC20(STETH).balanceOf(address(adapter)),
                2
            );
            assertApproxEqAbs(
                trade.calculatedAmount,
                IERC20(INSTETH).balanceOf(address(adapter)) - insteth_balance,
                2
            );
            assertLe(trade.gasUsed, 260000);
        }
    }
}

interface ILidoEth {
    function submit(address) external payable;
}
