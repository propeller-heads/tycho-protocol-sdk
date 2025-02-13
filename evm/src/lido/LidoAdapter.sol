// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from
    "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title Lido DAO Adapter
contract LidoAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    IwstETH wstETH;
    IStETH stETH;

    constructor(address _wstETH, address _stETH) {
        wstETH = IwstETH(_wstETH);
        stETH = IStETH(_stETH);
    }

    /// @notice Internal check for input and output tokens
    /// @dev This contract only supports swaps of tokens: ETH(address(0)), stETH
    /// and wstETH, ETH cannot be used as buy token because its withdrawal is
    /// not instant
    modifier checkInputTokens(
        address sellTokenAddress,
        address buyTokenAddress
    ) {
        address wstETHAddress = address(wstETH);
        address stETHAddress = address(stETH);
        bool supported = true;

        if (sellTokenAddress == buyTokenAddress) {
            supported = false;
        } else {
            if (
                sellTokenAddress != wstETHAddress
                    && sellTokenAddress != stETHAddress
                    && sellTokenAddress != address(0)
            ) {
                supported = false;
            } else {
                if (
                    buyTokenAddress != wstETHAddress
                        && buyTokenAddress != stETHAddress
                ) {
                    supported = false;
                }
            }
        }

        if (!supported) {
            revert Unavailable(
                "This contract only supports wstETH<->stETH, ETH(address(0))->wstETH and ETH(address(0))->stETH swaps"
            );
        }
        _;
    }

    /// @dev enable receive to deposit ether for payable swaps
    receive() external payable {}

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32,
        address _sellToken,
        address _buyToken,
        uint256[] memory _specifiedAmounts
    )
        external
        view
        override
        checkInputTokens(_sellToken, _buyToken)
        returns (Fraction[] memory _prices)
    {
        _prices = new Fraction[](_specifiedAmounts.length);

        for (uint256 i = 0; i < _specifiedAmounts.length; i++) {
            _prices[i] =
                _getPriceAt(_specifiedAmounts[i], _sellToken, _buyToken);
        }
    }

    /// @inheritdoc ISwapAdapter
    function swap(
        bytes32,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    )
        external
        checkInputTokens(sellToken, buyToken)
        returns (Trade memory trade)
    {
        if (specifiedAmount == 0) {
            return trade;
        }

        address stETHAddress = address(stETH);
        address wstETHAddress = address(wstETH);

        if (side == OrderSide.Buy) {
            if (sellToken == stETHAddress) {
                // stETH-wstETH
                uint256 amountIn = wstETH.getStETHByWstETH(specifiedAmount);
                IERC20(stETH).safeTransferFrom(
                    msg.sender, address(this), amountIn
                );
                IERC20(stETH).safeIncreaseAllowance(wstETHAddress, amountIn);
                wstETH.wrap(amountIn);
            } else if (sellToken == wstETHAddress) {
                // wstETH-stETH
                uint256 amountIn = wstETH.getWstETHByStETH(specifiedAmount);
                IERC20(wstETH).safeTransferFrom(
                    msg.sender, address(this), amountIn
                );
                wstETH.unwrap(amountIn);
            } else {
                // ETH-wstETH and ETH-stETH
                if (buyToken == stETHAddress) {
                    uint256 amountIn =
                        stETH.getPooledEthByShares(specifiedAmount);
                    (bool sent_,) = wstETHAddress.call{value: amountIn}("");
                    if (!sent_) revert Unavailable("Ether transfer failed");
                    uint256 wstETHAmountReceived =
                        stETH.getSharesByPooledEth(amountIn);
                    wstETH.unwrap(wstETHAmountReceived);
                } else {
                    (bool sent_,) =
                        wstETHAddress.call{value: specifiedAmount}("");
                    if (!sent_) revert Unavailable("Ether transfer failed");
                }
            }
        } else {
            if (sellToken == stETHAddress) {
                IERC20(stETH).safeTransferFrom(
                    msg.sender, address(this), specifiedAmount
                );
                IERC20(stETH).safeIncreaseAllowance(
                    wstETHAddress, specifiedAmount
                );
                wstETH.wrap(specifiedAmount);
            } else if (sellToken == address(0)) {
                if (buyToken == stETHAddress) {
                    (bool sent_,) =
                        wstETHAddress.call{value: specifiedAmount}("");
                    if (!sent_) revert Unavailable("Ether transfer failed");
                    uint256 wstETHAmountReceived =
                        stETH.getSharesByPooledEth(specifiedAmount);
                    wstETH.unwrap(wstETHAmountReceived);
                } else {
                    (bool sent_,) =
                        wstETHAddress.call{value: specifiedAmount}("");
                    if (!sent_) revert Unavailable("Ether transfer failed");
                }
            } else {
                IERC20(wstETH).safeTransferFrom(
                    msg.sender, address(this), specifiedAmount
                );
                wstETH.unwrap(specifiedAmount);
            }
        }
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32, address sellToken, address buyToken)
        external
        view
        override
        checkInputTokens(sellToken, buyToken)
        returns (uint256[] memory limits)
    {
        uint256 currentStakeLimitStETH = stETH.getCurrentStakeLimit(); // same
        // as ETH stake limit
        uint256 currentStakeLimitWstETH =
            wstETH.getWstETHByStETH(currentStakeLimitStETH);
        address stETHAddress = address(stETH);

        limits = new uint256[](2);
        if (sellToken == stETHAddress) {
            // stETH-wstETH
            limits[0] = currentStakeLimitStETH;
            limits[1] = currentStakeLimitWstETH;
        } else if (sellToken == address(wstETH)) {
            // wstETH-stETH
            limits[0] = currentStakeLimitWstETH;
            limits[1] = currentStakeLimitStETH;
        } else {
            // ETH-wstETH and ETH-stETH

            /// @dev Fix for side == Buy, because getTotalPooledEthByShares
            /// would be higher than currentStakeLimit if using the limit as
            /// amount
            uint256 pooledEthByShares_ =
                stETH.getPooledEthByShares(currentStakeLimitStETH);
            if (pooledEthByShares_ > currentStakeLimitStETH) {
                currentStakeLimitStETH = currentStakeLimitStETH
                    - (pooledEthByShares_ - currentStakeLimitStETH);
            }

            limits[0] = currentStakeLimitStETH;
            if (buyToken == stETHAddress) {
                limits[1] = currentStakeLimitStETH;
            } else {
                limits[1] = currentStakeLimitWstETH;
            }
        }
    }

    /// @inheritdoc ISwapAdapter
    function getCapabilities(bytes32, address, address)
        external
        pure
        override
        returns (Capability[] memory capabilities)
    {
        capabilities = new Capability[](5);
        capabilities[0] = Capability.SellOrder;
        capabilities[1] = Capability.BuyOrder;
        capabilities[2] = Capability.PriceFunction;
        capabilities[3] = Capability.ConstantPrice;
        capabilities[4] = Capability.HardLimits;
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(bytes32)
        external
        view
        override
        returns (address[] memory tokens)
    {
        tokens = new address[](3);
        tokens[0] = address(0);
        tokens[1] = address(wstETH);
        tokens[2] = address(stETH);
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
    function _getPriceAt(
        uint256 specifiedAmount,
        address sellTokenAddress,
        address buyTokenAddress
    ) internal view returns (Fraction memory) {
        address stETHAddress = address(stETH);
        uint256 amount0;
        uint256 amount1;

        if (sellTokenAddress == stETHAddress) {
            // stETH-wstETH, ETH is not possible as checked through
            // checkInputTokens
            amount0 = wstETH.getWstETHByStETH(specifiedAmount);
            amount1 = wstETH.getStETHByWstETH(amount0);
        } else if (sellTokenAddress == address(wstETH)) {
            // wstETH-stETH, ETH is not possible as checked through
            // checkInputTokens
            amount0 = wstETH.getStETHByWstETH(specifiedAmount);
            amount1 = wstETH.getWstETHByStETH(amount0);
        } else {
            // ETH (address(0))
            if (buyTokenAddress == stETHAddress) {
                amount0 = stETH.getSharesByPooledEth(specifiedAmount);
                amount1 = stETH.getPooledEthByShares(amount0);
            } else {
                uint256 stETHAmount =
                    stETH.getSharesByPooledEth(specifiedAmount);
                amount0 = wstETH.getWstETHByStETH(stETHAmount);
                amount1 =
                    stETH.getPooledEthByShares(wstETH.getStETHByWstETH(amount0));
            }
        }

        return Fraction(amount0, amount1);
    }
}

/// @dev Wrapped and extended interface for stETH
interface IStETH is IERC20 {
    function getPooledEthByShares(uint256 _sharesAmount)
        external
        view
        returns (uint256);

    function getSharesByPooledEth(uint256 _pooledEthAmount)
        external
        view
        returns (uint256);

    function submit(address _referral) external payable returns (uint256);

    function getCurrentStakeLimit() external view returns (uint256);
}

/// @dev Wrapped interface for wstETH
interface IwstETH is IERC20 {
    function stETH() external view returns (IStETH);

    function wrap(uint256 _stETHAmount) external returns (uint256);

    function unwrap(uint256 _wstETHAmount) external returns (uint256);

    function getWstETHByStETH(uint256 _stETHAmount)
        external
        view
        returns (uint256);

    function getStETHByWstETH(uint256 _wstETHAmount)
        external
        view
        returns (uint256);

    function stEthPerToken() external view returns (uint256);

    function tokensPerStEth() external view returns (uint256);
}
