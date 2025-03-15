// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./SwapExecutor.t.sol";
import "../src/fluid/FluidSwapExecutor.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

contract TestFluidSwapExecutor is SwapExecutorTest {
    using SafeERC20 for IERC20;

    FluidSwapExecutor fluid;
    IERC20 USDC = IERC20(USDC_ADDR);
    IERC20 USDT = IERC20(USDT_ADDR);
    address constant FLUID_LIQUIDITY = 0x52Aa899454998Be5b000Ad077a46Bbe360F4e497;
    address constant DEX_POOL = 0x667701e51B4D1Ca244F17C78F7aB8744B4C99F9B; // USDC-USDT pool

    function setUp() public {
        // Fork mainnet at a specific block
        uint256 forkBlock = 22052269;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);

        // Deploy the FluidSwapExecutor contract
        fluid = new FluidSwapExecutor(FLUID_LIQUIDITY);
    }

    function testFluidSwapExactIn() public {
        uint256 sellAmount = 1000 * 10 ** 6; // 1000 USDT
        uint256 expectedAmount = 998 * 10 ** 6; // Expected 998 USDC
        bool swap0to1 = true;
        bytes memory protocolData = abi.encode(swap0to1, DEX_POOL);

        // Fund the FluidSwapExecutor contract
        deal(USDT_ADDR, bob, sellAmount);

        vm.prank(bob);
        IERC20(USDT_ADDR).approve(address(fluid), sellAmount);

        vm.prank(bob);
        uint256 responseAmount = fluid.swap(sellAmount, protocolData);

        // Assertions
        assertEq(responseAmount, expectedAmount);
        assertEq(USDC.balanceOf(bob), expectedAmount);
        assertEq(USDT.balanceOf(bob), 0);
    }

    function testFluidSwapExactOut() public {
        uint256 buyAmount = 1000 * 10 ** 6; // 1000 USDC
        uint256 expectedSellAmount = 1005 * 10 ** 6; // Expected to sell 1002 USDT
        bool swap0to1 = false;
        bytes memory protocolData = abi.encode(swap0to1, DEX_POOL);

        // Fund the FluidSwapExecutor contract
        deal(USDC_ADDR, bob, expectedSellAmount);

        vm.prank(bob);
        IERC20(USDC_ADDR).approve(address(fluid), expectedSellAmount);

        vm.prank(bob);
        uint256 responseAmount = fluid.swap(buyAmount, protocolData);

        // Assertions
        assertEq(responseAmount, expectedSellAmount);
        assertEq(USDT.balanceOf(bob), buyAmount);
        assertEq(USDC.balanceOf(bob), 0);
    }
}
