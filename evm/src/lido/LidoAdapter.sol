// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {IERC20, ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";

/// @title Lido DAO Adapter
contract LidoAdapter is ISwapAdapter {

    IwstETH wstETH;
    IStETH stETH;

    constructor(IwstETH _wstETH) {
        wstETH = _wstETH;
        stETH = _wstETH.stETH();
    }

    /// @notice Internal check for input and output tokens
    /// @dev This contract only supports swaps of tokens: ETH(address(0)), stETH and wstETH
    modifier checkInputTokens(IERC20 sellToken, IERC20 buyToken) {
        address sellTokenAddress = address(sellToken);
        address buyTokenAddress = address(buyToken);
        address wstETHAddress = address(wstETH);
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

    /// @dev enable receive to deposit ether for payable swaps
    receive() external payable {}

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32,
        IERC20 _sellToken,
        IERC20 _buyToken,
        uint256[] memory _specifiedAmounts
    ) checkInputTokens(_sellToken, _buyToken) external view override returns (Fraction[] memory _prices) {
        _prices = new Fraction[](_specifiedAmounts.length);
        address sellTokenAddress = address(_sellToken);
        address buyTokenAddress = address(_buyToken);

        for(uint256 i = 0; i < _specifiedAmounts.length; i++) {
            _prices[i] = _getPriceAt(_specifiedAmounts[i], sellTokenAddress, buyTokenAddress);
        }
    }

    /// @inheritdoc ISwapAdapter
    function swap(
        bytes32,
        IERC20 sellToken,
        IERC20 buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) checkInputTokens(sellToken, buyToken) external returns (Trade memory trade) {
        if (specifiedAmount == 0) {
            return trade;
        }

        address stETHAddress = address(stETH);
        address wstETHAddress = address(wstETH);
        if(address(buyToken) == address(0)) {
            revert Unavailable("Cannot swap for ETH since withdrawal is processed externally");
        }

        if(side == OrderSide.Buy) {
            if(address(sellToken) == stETHAddress) {
                uint256 amountIn = wstETH.getStETHByWstETH(specifiedAmount);
                stETH.transferFrom(msg.sender, address(this), amountIn);
                stETH.approve(wstETHAddress, amountIn);
                wstETH.wrap(amountIn);
            }
            else {
                uint256 amountIn = wstETH.getWstETHByStETH(specifiedAmount);
                wstETH.transferFrom(msg.sender, address(this), amountIn);
                wstETH.unwrap(amountIn);
            }
        }
        else {
            if(address(sellToken) == stETHAddress) {
                stETH.transferFrom(msg.sender, address(this), specifiedAmount);
                stETH.approve(wstETHAddress, specifiedAmount);
                wstETH.wrap(specifiedAmount);
            }
            else if(address(sellToken) == address(0)) {
                if(address(buyToken) == stETHAddress) {
                    (bool sent_, ) = wstETHAddress.call{value: specifiedAmount}("");
                    if(!sent_) { revert Unavailable("Ether transfer failed"); }
                    uint256 wstETHAmountReceived = stETH.getSharesByPooledEth(specifiedAmount);
                    wstETH.unwrap(wstETHAmountReceived);
                }
                else {
                    (bool sent_, ) = wstETHAddress.call{value: specifiedAmount}("");
                    if(!sent_) { revert Unavailable("Ether transfer failed"); }
                }
            }
            else {
                wstETH.transferFrom(msg.sender, address(this), specifiedAmount);
                wstETH.unwrap(specifiedAmount);
            }
        }
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32, IERC20 sellToken, IERC20 buyToken)
        checkInputTokens(sellToken, buyToken)
        external
        view
        override
        returns (uint256[] memory limits)
    {
        uint256 currentStakeLimitStETH = stETH.getCurrentStakeLimit(); // same as ETH stake limit
        uint256 currentStakeLimitWstETH = wstETH.getWstETHByStETH(currentStakeLimitStETH);
        address sellTokenAddress = address(sellToken);
        address stETHAddress = address(stETH);

        limits = new uint256[](2);
        if(sellTokenAddress == stETHAddress) { // stETH-wstETH
            limits[0] = currentStakeLimitStETH;
            limits[1] = currentStakeLimitWstETH;
        }
        else if(sellTokenAddress == address(wstETH)) { // wstETH-stETH
            limits[0] = currentStakeLimitWstETH;
            limits[1] = currentStakeLimitStETH;
        }
        else { // ETH-wstETH and ETH-stETH
            limits[0] = currentStakeLimitStETH;
            if(address(buyToken) == stETHAddress) {
                limits[1] = currentStakeLimitStETH;
            }
            else {
                limits[1] = currentStakeLimitWstETH;
            }
        }
    }

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
        override
        returns (IERC20[] memory tokens)
    {
        tokens = new IERC20[](3);
        tokens[0] = IERC20(address(0));
        tokens[1] = IERC20(address(wstETH));
        tokens[2] = IERC20(address(stETH));
    }

    function getPoolIds(uint256, uint256)
        external
        pure
        override
        returns (bytes32[] memory)
    {
        revert NotImplemented("LidoAdapter.getPoolIds");
    }

    /// @notice Get swap price between two tokens with a given specifiedAmount
    /// @param specifiedAmount amount to swap
    /// @param sellTokenAddress address of the token to sell
    /// @param buyTokenAddress address of the token to buy
    /// @return uint256 swap price
    function _getPriceAt(uint256 specifiedAmount, address sellTokenAddress, address buyTokenAddress) internal view returns (Fraction memory) {
        address wstETHAddress = address(wstETH);
        address stETHAddress = address(stETH);
        uint256 amount0;
        uint256 amount1;

        if(sellTokenAddress == stETHAddress) {
            if(buyTokenAddress == wstETHAddress) {
                amount0 = wstETH.getWstETHByStETH(specifiedAmount);
                amount1 = wstETH.getStETHByWstETH(amount0);
            }
            else {
                amount0 = stETH.getPooledEthByShares(specifiedAmount);
                amount1 = stETH.getSharesByPooledEth(amount0);
            }
        }
        else if(sellTokenAddress == wstETHAddress) {
            if(buyTokenAddress == stETHAddress) {
                amount0 = wstETH.getStETHByWstETH(specifiedAmount);
                amount1 = wstETH.getWstETHByStETH(amount0);
            }
            else {
                uint256 stETHAmount = wstETH.getStETHByWstETH(specifiedAmount);
                amount0 = stETH.getPooledEthByShares(stETHAmount);
                amount1 = wstETH.getWstETHByStETH(stETH.getSharesByPooledEth(amount0));
            }
        }
        else { // ETH (address(0))
            if(buyTokenAddress == stETHAddress) {
                amount0 = stETH.getSharesByPooledEth(specifiedAmount);
                amount1 = stETH.getPooledEthByShares(amount0);
            }
            else {
                uint256 stETHAmount = stETH.getSharesByPooledEth(specifiedAmount);
                amount0 = wstETH.getWstETHByStETH(stETHAmount);
                amount1 = stETH.getPooledEthByShares(wstETH.getStETHByWstETH(amount0));
            }
        }

        return Fraction(
            amount0,
            amount1
        );
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
interface IwstETH is IERC20 {

    function stETH() external view returns (IStETH);

    function wrap(uint256 _stETHAmount) external returns (uint256);

    function unwrap(uint256 _wstETHAmount) external returns (uint256);

    function getWstETHByStETH(uint256 _stETHAmount) external view returns (uint256);

    function getStETHByWstETH(uint256 _wstETHAmount) external view returns (uint256);

    function stEthPerToken() external view returns (uint256);

    function tokensPerStEth() external view returns (uint256);

}

