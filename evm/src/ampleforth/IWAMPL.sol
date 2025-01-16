// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

interface IWAMPL {
    function deposit(uint256 underlyingAmt) external returns (uint256);
    function depositFor(address user, uint256 underlyingAmt)
        external
        returns (uint256);
    function depositAll() external returns (uint256);
    function burn(uint256 wrapperAmt) external returns (uint256);
    function burnTo(address user, uint256 wrapperAmt)
        external
        returns (uint256);
    function burnAll() external returns (uint256);
    function underlyingToWrapper(uint256 underlyingAmt)
        external
        view
        returns (uint256);
    function wrapperToUnderlying(uint256 wrapperAmt)
        external
        view
        returns (uint256);
}
