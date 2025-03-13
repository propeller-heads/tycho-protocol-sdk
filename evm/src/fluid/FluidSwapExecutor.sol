// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "../interfaces/ISwapExecutor.sol";
import {IFluidDexT1} from "./Interfaces/iDexT1.sol";
import {IDexCallback} from "./Interfaces/DexCallback.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

contract FluidSwapExecutor is ISwapExecutor, IDexCallback {
    using SafeERC20 for IERC20;
    address public immutable FLUID_LIQUIDITY;
    address transient sender;
    address transient dexAddress; 

    constructor(address fluidLiquidity_) {
        FLUID_LIQUIDITY = fluidLiquidity_;
    }


    /**
     * @dev Executes a Fluid DEX swap.
     * @param givenAmount how much of to swap, depending on exactOut either in or out Amount.
     * @param data the parameters of the swap. This data is roughly the packed
     * encoding of
     *      swap0to1 // true if token0 is being swapped for token1
     *      poolAddress
     *      sellToken
     */
    function swap(
        uint256 givenAmount,
        bytes calldata data
    ) external payable returns (uint256 calculatedAmount) {
        (bool swap0to1, address poolAddress) 
            = abi.decode(data, (bool, address));

        sender = msg.sender;
        dexAddress = poolAddress;

        IFluidDexT1 pool = IFluidDexT1(dexAddress);

        if (swap0to1) {
            calculatedAmount = pool.swapInWithCallback{value: msg.value}(
                swap0to1,
                givenAmount,
                0,
                msg.sender
            );
        } else {
            calculatedAmount = pool.swapOutWithCallback{value: msg.value}(
                swap0to1,
                givenAmount,
                type(uint256).max,
                msg.sender
            );
        }
    }

    /// @notice Callback function called by the DEX
    function dexCallback(address token_, uint256 amount_) external override {
        require(msg.sender == dexAddress, "Invalid caller");
        IERC20(token_).safeTransferFrom(sender, FLUID_LIQUIDITY, amount_);
    }

    receive() external payable {}
}
