//SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import {IGenericPoolPermissioned} from "./IGenericPool.sol";

/**
 * @notice Public functions of ERC20 pool interface.
 */
interface IPool is IGenericPoolPermissioned {
    function deposit(uint256 amount)
        external
        returns (uint256 poolShares, int256 fee);

    function withdraw(uint256 shares, uint256 minimumAmount)
        external
        returns (uint256 finalAmount, int256 fee);
}
