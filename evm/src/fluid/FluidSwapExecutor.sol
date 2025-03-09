// SPDX-License-Identifier: UNLICENCED
pragma solidity ^0.8.0;

import "../interfaces/ISwapExecutor.sol";
import {FluidDexReservesResolver} from "./Interfaces/FluidInterfaces.sol";
import {IFluidDexT1} from "./Interfaces/iDexT1.sol";


contract FluidSwapExecutor is ISwapExecutor {

    /// @notice Reserves resolver contract
    FluidDexReservesResolver public immutable resolver;

    /// @notice Constructor to set the reserves resolver
    /// @param resolver_ Address of the reserves resolver contract
    constructor(address resolver_) {
        resolver = FluidDexReservesResolver(resolver_);
    }

    /**
     * @dev Executes a Balancer swap.
     * @param givenAmount how much of to swap, depending on exactOut either in-
     * or outAmount.
     * @param data the parameters of the swap. This data is roughly the packed
     * encoding of
     *      poolId
     *      sellToken
     *      buyToken
     *      side    // true for sell and false for buy
     */
    function swap(uint256 givenAmount, bytes calldata data)
        external
        payable
        returns (uint256 calculatedAmount)
    {

    (bytes32 poolId, address sellToken, address buyToken, bool side) = abi.decode(data, (bytes32, address, address, bool));
        if (givenAmount == 0) {
            return 0;
        }
        address poolAddress = resolver.getPoolAddress(uint256(poolId));
        IFluidDexT1 pool = IFluidDexT1(poolAddress);

        (address token0,) = resolver.getPoolTokens(poolAddress);

        if(sellToken != 0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE){
            IERC20(sellToken).transferFrom(msg.sender, address(this), givenAmount);
            IERC20(sellToken).approve(poolAddress, givenAmount);
        }

        if (side) {
            calculatedAmount = pool.swapIn{value: msg.value}(
                sellToken == token0, givenAmount, 0, msg.sender
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

    function receive() external payable {}
}
