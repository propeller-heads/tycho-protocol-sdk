// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

interface IBillBroker {
    function swapPerpsForUSD(uint256 perpAmt, uint256 usdAmtMin)
        external
        returns (uint256);
    function swapUSDForPerps(uint256 usdAmt, uint256 perpAmtMin)
        external
        returns (uint256);
    function computePerpToUSDSwapAmt(uint256 perpAmt)
        external
        returns (uint256);
    function computeUSDToPerpSwapAmt(uint256 usdAmt)
        external
        returns (uint256);
    function usdBalance() external view returns (uint256);
    function perpBalance() external view returns (uint256);
}
