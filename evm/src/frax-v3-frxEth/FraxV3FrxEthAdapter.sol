// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {IERC20, ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {ERC20} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import {SafeERC20} from
    "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

// --- Contract Addresses --- //
// frxETH: 0x5e8422345238f34275888049021821e8e08caa1f
// frxETHMinter: 0xbAFA44EFE7901E04E39Dad13167D089C559c1138
// sfrxETH: 0xac3E018457B222d93114458476f3E3416Abbe38F

/// @title FraxV3FrxEthAdapter
/// Adapter for frxETH and sfrxETH tokens of FraxV3
/// @dev This contract only supports: ETH-....
contract FraxV3FrxEthAdapter {

    IFrxEth frxEth;
    IFrxEthMinter frxEthMinter;
    ISfrxEth sfrxEth;

    constructor(address _frxEth) {
        frxEth = IFrxEth(_frxEth);
        address[] mintersArray = frxEth.minters_array(0);
        frxEthMinter = IFrxEthMinter(mintersArray[0]);
        sfrxEth = frxEthMinter.sfrxETHToken();
    }

    /// @dev Comment here explaining modifier
    modifier checkInputTokens(address sellToken, address buyToken) {
        if(false /*add check here*/) {
            revert Unavailable("Message here...");
        }
        else {
            if(false) {
                revert Unavailable("Message here...");
            }
        }
        _;
    }
    
    /// @dev enable receive to fill the contract with ether for payable swaps
    receive() external payable {}

    function myFunction(IERC20 sellToken, IERC20 buyToken) external override view 
    checkInputTokens(address(sellToken), address(buyToken)) {
        /// ....function here....
    }

}

interface IFrxEth {

    function minters_array(uint256 i) external view returns (address[]) {}

}

interface ISfrxEth {

}

interface IFrxEthMinter {

    function sfrxETHToken() external view returns (ISfrxEth) {}

}