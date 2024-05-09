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

    function setUp() public {
        uint256 forkBlock = 18835309;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new sDaiSwapAdapter(SDAI_ADDRESS);
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

    function testGetAssetAddress() public {
        address dai = adapter.getAssetAddress();
        console.log("Dai address", dai);
    }

}
