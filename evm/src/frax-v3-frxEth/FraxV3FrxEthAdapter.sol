// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {IERC20, ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {ERC20} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import {SafeERC20} from
    "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import "src/libraries/FractionMath.sol";

/// @title FraxV3FrxEthAdapter
/// Adapter for frxETH and sfrxETH tokens of FraxV3
/// @dev This contract only supports: ETH -> sfrxETH and frxETH <-> sfrxETH
contract FraxV3FrxEthAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;
    using FractionMath for Fraction;

    IFrxEth frxEth;
    IFrxEthMinter frxEthMinter;
    ISfrxEth sfrxEth;

    constructor(address _frxEth, address _frxEthMinter, address _sfrxEth) {
        frxEth = IFrxEth(_frxEth);
        frxEthMinter = IFrxEthMinter(_frxEthMinter);
        sfrxEth = ISfrxEth(_sfrxEth);
    }

    /// @dev Check if tokens in input are supported
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

    function price(
        bytes32 _poolId,
        IERC20 _sellToken,
        IERC20 _buyToken,
        uint256[] memory _specifiedAmounts
    ) external view override returns (Fraction[] memory _prices) {
        revert NotImplemented("FraxV3FrxEthAdapter.price");
    }

    function swap(
        bytes32 poolId,
        IERC20 sellToken,
        IERC20 buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external returns (Trade memory trade) {
        revert NotImplemented("FraxV3FrxEthAdapter.swap");
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32 poolId, IERC20 sellToken, IERC20 buyToken)
        external
        view
        override
        onlySupportedTokens(address(sellToken), address(buyToken))
        returns (uint256[] memory limits)
    {
        limits = new uint256[](2);
        address sellTokenAddress = address(sellToken);
        address buyTokenAddress = address(buyToken);
        if(sellTokenAddress == address(0) && buyTokenAddress == address(sfrxEth)) {

            limits[0] = type(uint256).max;
            limits[1] = sfrxEth.previewDeposit(limits[0]);

        } else {

            if (sellTokenAddress == address(frxEth) && buyTokenAddress == address(sfrxEth)) {

                limits[0] = frxEth.totalSupply() - sfrxEth.balanceOf(sellTokenAddress);
                limits[1] = sfrxEth.previewDeposit(limits[0]);

            } else {
                
                limits[0] = sfrxEth.totalSupply();
                limits[1] = sfrxEth.previewRedeem(limits[0]);
            }
        }
    }

    /// @inheritdoc ISwapAdapter
    function getCapabilities(bytes32, IERC20, IERC20)
        external
        pure
        override
        returns (Capability[] memory capabilities)
    {
        capabilities = new Capability[](3);
        capabilities[0] = Capability.SellOrder;
        capabilities[1] = Capability.BuyOrder;
        capabilities[2] = Capability.PriceFunction;
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(bytes32)
        external
        view
        returns (IERC20[] memory tokens)
    {
        tokens = new IERC20[](3);

        tokens[0] = IERC20(address(0));
        tokens[1] = IERC20(frxEthMinter.frxETHToken());
        tokens[2] = IERC20(frxEthMinter.sfrxETHToken());
    }

    /// @inheritdoc ISwapAdapter
    /// @dev although FraxV3 frxETH has no pool ids, we return the sFrxETH and frxETHMinter addresses as pools
    function getPoolIds(uint256, uint256)
        external
        returns (bytes32[] memory ids)
    {
        ids = new bytes32[](2);
        ids[0] = bytes20(address(sfrxEth));
        ids[1] = bytes20(address(frxEthMinter));
    }
}

interface IFrxEth {
    // function minters_array(uint256) external view returns (address[] memory);

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
    //function sfrxETHTokenContract() external view returns (ISfrxEth);

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
