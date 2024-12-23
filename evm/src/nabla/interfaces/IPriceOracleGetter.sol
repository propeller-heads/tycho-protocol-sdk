// Based on AAVE protocol
// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

/// @title IPriceOracleGetter interface
interface IPriceOracleGetter {
    /// @dev returns the asset price in USD
    function getAssetPrice(address _asset) external view returns (uint256);
}
