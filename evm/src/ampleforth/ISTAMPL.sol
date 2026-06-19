// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

interface ISTAMPL {
    function swapUnderlyingForPerps(uint256 underlyingAmt)
        external
        returns (uint256);
    function swapPerpsForUnderlying(uint256 perpAmt)
        external
        returns (uint256);
    function computeUnderlyingToPerpSwapAmt(uint256 underlyingAmt)
        external
        returns (uint256);
    function computePerpToUnderlyingSwapAmt(uint256 perpAmt)
        external
        returns (uint256);
}
