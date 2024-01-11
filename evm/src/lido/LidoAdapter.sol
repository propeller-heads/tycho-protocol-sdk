// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {IERC20, ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";

/// @title Lido DAO Adapter
contract LidoAdapter is ISwapAdapter {

    IwstETH wstEth;
    IStETH stETH;

    constructor(IwstETH _wstETH) {
        wstEth = _wstETH;
        stETH = _wstETH.stETH();
    }

    /// @notice Internal check for input and output tokens
    /// @dev This contract only supports swaps of tokens: ETH(address(0)), stETH and wstETH
    modifier checkInputTokens(IERC20 sellToken, IERC20 buyToken) {
        address sellTokenAddress = address(sellToken);
        address buyTokenAddress = address(buyToken);
        address wstETHAddress = address(wstEth);
        address stETHAddress = address(stETH);
        bool supported = true;

        if(sellTokenAddress == wstETHAddress) {
            if(buyTokenAddress != stETHAddress && buyTokenAddress != address(0)) {
                supported = false;
            }
        }
        else if(sellTokenAddress == stETHAddress) {
            if(buyTokenAddress != wstETHAddress && buyTokenAddress != address(0)) {
                supported = false;
            }
        }
        else if(sellTokenAddress == address(0)) {
            if(buyTokenAddress != wstETHAddress && buyTokenAddress != stETHAddress) {
                supported = false;
            }
        }
        
        if(!supported) {
            revert Unavailable("This contract only supports wstETH, ETH(address(0)) and stETH tokens");
        }
        _;
    }

    function price(
        bytes32 _poolId,
        IERC20 _sellToken,
        IERC20 _buyToken,
        uint256[] memory _specifiedAmounts
    ) external view override returns (Fraction[] memory _prices) {
        revert NotImplemented("LidoAdapter.price");
    }

    function swap(
        bytes32 poolId,
        IERC20 sellToken,
        IERC20 buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external returns (Trade memory trade) {
        revert NotImplemented("LidoAdapter.swap");
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32, IERC20 sellToken, IERC20 buyToken)
        checkInputTokens(sellToken, buyToken)
        external
        view
        override
        returns (uint256[] memory limits)
    {
        limits = new uint256[](2);
        if(address(sellToken) == address(stETH)) {
            limits[0] = stETH.getCurrentStakeLimit();
            limits[1] = type(uint256).max;
        }
        else if(address(buyToken) == address(stETH)) {
            limits[0] = type(uint256).max;
            limits[1] = stETH.getCurrentStakeLimit();
        }
        else {
            limits[0] = type(uint256).max;
            limits[1] = type(uint256).max;
        }
    }

    function getCapabilities(bytes32 poolId, IERC20 sellToken, IERC20 buyToken)
        external
        returns (Capability[] memory capabilities)
    {
        revert NotImplemented("LidoAdapter.getCapabilities");
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(bytes32)
        external
        view
        override
        returns (IERC20[] memory tokens)
    {
        tokens = new IERC20[](3);
        tokens[0] = IERC20(address(0));
        tokens[1] = IERC20(address(wstEth));
        tokens[2] = IERC20(address(stETH));
    }

    function getPoolIds(uint256 offset, uint256 limit)
        external
        returns (bytes32[] memory ids)
    {
        revert NotImplemented("LidoAdapter.getPoolIds");
    }
}

/// @dev Wrapped and extended interface for stETH
interface IStETH is IERC20 {
    function getPooledEthByShares(uint256 _sharesAmount) external view returns (uint256);

    function getSharesByPooledEth(uint256 _pooledEthAmount) external view returns (uint256);

    function submit(address _referral) external payable returns (uint256);

    function getCurrentStakeLimit() external view returns (uint256);
}

/// @dev Wrapped interface for wstETH
interface IwstETH {

    function stETH() external view returns (IStETH);

    function wrap(uint256 _stETHAmount) external returns (uint256);

    function unwrap(uint256 _wstETHAmount) external returns (uint256);

    function getWstETHByStETH(uint256 _stETHAmount) external view returns (uint256);

    function getStETHByWstETH(uint256 _wstETHAmount) external view returns (uint256);

    function stEthPerToken() external view returns (uint256);

    function tokensPerStEth() external view returns (uint256);

}

