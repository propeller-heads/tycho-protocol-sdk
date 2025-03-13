// SPDX-License-Identifier: UNLICENCED
pragma solidity ^0.8.0;

import "../interfaces/ISwapExecutor.sol";
import {IFluidDexT1} from "./Interfaces/iDexT1.sol";
import {IDexCallback} from "./Interfaces/DexCallback.sol";

contract FluidSwapExecutor is ISwapExecutor, IDexCallback {
    using SafeERC20 for IERC20;
    address constant DEAD_ADDRESS = 0x000000000000000000000000000000000000dEaD;
    address public immutable FLUID_LIQUIDITY;
    address public senderTransient;

    constructor(address fluidLiquidity_) {
        FLUID_LIQUIDITY = fluidLiquidity_;
        senderTransient = DEAD_ADDRESS;
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
        (bool swap0to1, address poolAddress, address sellToken) 
            = abi.decode(data, (bool, address, address));

        senderTransient = msg.sender;
        assembly {
            if iszero(givenAmount) {
                revert(0, 0x20)
            }

            let nativeToken := 0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE
            if iszero(eq(sellToken, nativeToken)) {
                let transferFromCallData := mload(0x40)
                mstore(transferFromCallData, 0x23b872dd) // Function selector for transferFrom(address,address,uint256)

                mstore(add(transferFromCallData, 0x04), caller()) // msg.sender
                mstore(add(transferFromCallData, 0x24), address()) // address(this)
                mstore(add(transferFromCallData, 0x44), givenAmount) // givenAmount

                let success := call(
                    gas(),
                    IERC20(sellToken),
                    0,
                    transferFromCallData,
                    0x64,
                    0,
                    0
                )

                if iszero(success) {
                    returndatacopy(0, 0, returndatasize())
                    revert(0, returndatasize())
                }
            }
        }

        IFluidDexT1 pool = IFluidDexT1(poolAddress);

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
        require(msg.sender == FLUID_DEX, "Invalid caller");

        // Transfer tokens from the senderTransient to Fluid liquidity
        assembly {
            let transferFromCallData := mload(0x40)
                mstore(transferFromCallData, 0x23b872dd) // Function selector for transferFrom(address,address,uint256)

                mstore(add(transferFromCallData, 0x04), senderTransient)
                mstore(add(transferFromCallData, 0x24), FLUID_LIQUIDITY)
                mstore(add(transferFromCallData, 0x44), amount_)

                let success := call(
                    gas(),
                    IERC20(token_),
                    0,
                    transferFromCallData,
                    0x64,
                    0,
                    0
                )

                if iszero(success) {
                    returndatacopy(0, 0, returndatasize())
                    revert(0, returndatasize())
                }
        }

        senderTransient = DEAD_ADDRESS; // Reset storage for gas refunds
    }

    receive() external payable {}
}
