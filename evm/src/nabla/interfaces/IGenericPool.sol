//SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

/**
 * @notice Generic ERC20 pool interface, public functions.
 */
interface IGenericPool {
    function asset() external view returns (address _token);

    function poolCap() external view returns (uint256 _maxTokens);

    function assetDecimals() external view returns (uint8 _decimals);
}

/**
 * @notice Access-restricted functions of the IGenericPool.
 */
interface IGenericPoolPermissioned is IGenericPool {
    function setPoolCap(uint256 _maxTokens) external;
}
