// SPDX-License-Identifier: UNLICENCED
pragma solidity ^0.8.0;

import "../interfaces/ISwapExecutor.sol";
import {IFluidDexT1} from "./Interfaces/iDexT1.sol";

contract FluidSwapExecutor is ISwapExecutor {
    /**
     * @dev Executes a Fluid DEX swap.
     * @param givenAmount how much of to swap, depending on exactOut either in or out Amount.
     * @param data the parameters of the swap. This data is roughly the packed
     * encoding of
     *      poolAddress
     *      sellToken
     *      buyToken
     *      side    // true for sell and false for buy
     */
    function swap(
        uint256 givenAmount,
        bytes calldata data
    ) external payable returns (uint256 calculatedAmount) {
        (
            address poolAddress,
            address sellToken,
            address buyToken,
            bool side
        ) = abi.decode(data, (address, address, address, bool));
        if (givenAmount == 0) {
            return 0;
        }

        IFluidDexT1 pool = IFluidDexT1(poolAddress);
        address token0 = pool.constantsView().token0;

        if (sellToken != 0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE) {
            IERC20(sellToken).transferFrom(
                msg.sender,
                address(this),
                givenAmount
            );
            IERC20(sellToken).approve(poolAddress, givenAmount);
        }

        if (side) {
            calculatedAmount = pool.swapIn{value: msg.value}(
                sellToken == token0,
                givenAmount,
                0,
                msg.sender
            );
        } else {
            calculatedAmount = pool.swapOut{value: msg.value}(
                sellToken == token0,
                givenAmount,
                type(uint256).max,
                msg.sender
            );
        }
    }

    receive() external payable {}
}
