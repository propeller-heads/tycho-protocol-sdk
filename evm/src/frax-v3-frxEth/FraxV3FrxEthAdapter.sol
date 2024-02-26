// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {IERC20, ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {ERC20} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import {SafeERC20} from
    "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title FraxV3FrxEthAdapter
/// Adapter for frxETH and sfrxETH tokens of FraxV3
/// @dev This contract only supports: ETH -> sfrxETH and frxETH <-> sfrxETH
contract FraxV3FrxEthAdapter {
    IFrxEth frxEth;
    IFrxEthMinter frxEthMinter;
    ISfrxEth sfrxEth;

    constructor(address _frxEth) {
        frxEth = IFrxEth(_frxEth);
        address[] mintersArray = frxEth.minters_array(0);
        frxEthMinter = IFrxEthMinter(mintersArray[0]);
        sfrxEth = frxEthMinter.sfrxETHTokenContract();
    }

    /// @dev Check if tokens in input are supported
    /// @inheritdoc ISwapAdapter
    modifier onlySupportedTokens(address sellToken, address buyToken) {
        address sellTokenAddress = sellToken;
        address buyTokenAddress = buyToken;

        if (sellTokenAddress == address(0)) {
            if (buyTokenAddress != address(sfrxEth)) {
                revert Unavailable(
                    "Only supported swaps are: ETH -> sfrxETH and frxETH <-> sfrxETH"
                );
            }
        } else {
            if (buyTokenAddress == address(0)) {
                revert Unavailable(
                    "Only supported swaps are: ETH -> sfrxETH and frxETH <-> sfrxETH"
                );
            }
            if (
                sellTokenAddress != address(sfrxEth) && buyTokenAddress != address(frxEth) || 
                sellTokenAddress != address(frxEth) && buyTokenAddress != address(sfrxEth)
            ) {
                revert Unavailable(
                    "Only supported swaps are: ETH -> sfrxETH and frxETH <-> sfrxETH"
                );
            }
        }
        _;
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(bytes32)
        external
        view
        returns (IERC20[] memory tokens)
    {
        tokens = new tokens[](3);

        tokens[0] = IERC20(address(0));
        tokens[1] = IERC20(frxEthMinter.frxETHToken());
        tokens[2] = IERC20(frxEthMinter.sfrxETHToken());
    }
}

interface IFrxEth {
    function minters_array(uint256 i) external view returns (address[]);

    function balanceOf(address) external view returns (uint256);

    function totalSupply() external view returns (uint256);
}

interface ISfrxEth {
    /// @dev even though the balance address of frxETH token is around 223,701
    /// tokens, it returns 0 when the
    /// address of frxEth is passed as an argument
    function balanceOf(address) external view returns (uint256);

    /// @dev to be clarified if the accepted asset is ETH or frxETH
    function previewDeposit(uint256 assets) external view returns (uint256);

    /// @dev It should accept sfrxETH, to be clarified if it returns ETH or
    /// frxET
    function previewMint(uint256 shares) external view returns (uint256);

    /// @dev It should accept sfrxETH, to be clarified if it returns ETH or
    /// frxET
    function previewRedeem(uint256 shares) external view returns (uint256);

    /// @dev It should accept sfrxETH, to be clarified if it returns ETH or
    /// frxET
    function previewWithdraw(uint256 assets) external view returns (uint256);

    /// @dev returns the totalSupply of frxETH
    function totalSupply() external view returns (uint256);

    /// @notice Compute the amount of tokens available to share holders
    function totalAssets() external view returns (uint256);

    /// @notice missing a public function for storedTotaAssets

    function deposit(uint256 assets, address receiver)
        external
        returns (uint256 shares);

    function mint(uint256 shares, address receiver)
        external
        returns (uint256 assets);

    function storedTotalAssets() external view returns (uint256);

    function withdraw(uint256 assets, address receiver, address owner)
        external
        returns (uint256 shares);

    function redeem(uint256 shares, address receiver, address owner)
        external
        returns (uint256 assets);
}

interface IFrxEthMinter {
    function sfrxETHTokenContract() external view returns (ISfrxEth);

    function sfrxETHToken() external view returns (address);

    function frxETHToken() external view returns (address);

    function currentWithheldETH() external view returns (uint256);

    function DEPOSIT_SIZE() external view returns (uint256);

    /// @notice Mint frxETH to the sender depending on the ETH value sent
    function submit() external payable;

    /// @notice Mint frxETH and deposit it to receive sfrxETH in one transaction
    function submitAndDeposit(address recipient)
        external
        payable
        returns (uint256 shares);
}
