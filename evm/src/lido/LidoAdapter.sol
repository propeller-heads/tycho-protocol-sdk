// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from
    "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import "forge-std/console.sol";

/// @title Lido DAO Adapter
contract LidoAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    IwstETH public immutable wstEth;
    IStETH public immutable stEth;
    uint256 public constant BUFFER = 1000000;

    constructor(address _wstEth, address _stEth) {
        wstEth = IwstETH(_wstEth);
        stEth = IStETH(_stEth);
    }

    /// @notice Internal check for input and output tokens
    /// @dev This contract only supports swaps of tokens: Eth(address(0)), stEth
    /// and wstEth, Eth cannot be used as buy token because its withdrawal is
    /// not instant
    modifier checkInputTokens(address sellToken, address buyToken) {
        bool supported = true;

        if (sellToken == buyToken) {
            supported = false;
        } else {
            if (
                sellToken != address(wstEth) && sellToken != address(stEth)
                    && sellToken != address(0)
            ) {
                supported = false;
            } else {
                if (buyToken != address(wstEth) && buyToken != address(stEth)) {
                    supported = false;
                }
            }
        }

        if (!supported) {
            revert Unavailable(
                "This contract only supports wstEth<->stEth, Eth(address(0))->wstEth and Eth(address(0))->stEth swaps"
            );
        }
        _;
    }

    /// @dev enable receive to deposit ether for payable swaps
    receive() external payable {}

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32,
        address sellToken,
        address buyToken,
        uint256[] memory specifiedAmounts
    )
        external
        view
        override
        checkInputTokens(sellToken, buyToken)
        returns (Fraction[] memory prices)
    {
        prices = new Fraction[](specifiedAmounts.length);

        for (uint256 i = 0; i < specifiedAmounts.length; i++) {
            prices[i] = _getPriceAt(specifiedAmounts[i], sellToken, buyToken);
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

        uint256 gasBefore = gasleft();

        if (side == OrderSide.Buy) {
            if (sellToken == address(stEth)) {
                uint256 neededStEth =
                    stEth.getPooledEthByShares(specifiedAmount);
                neededStEth = neededStEth + (neededStEth / BUFFER);

                IERC20(stEth).safeTransferFrom(
                    msg.sender, address(this), neededStEth
                );

                IERC20(stEth).safeIncreaseAllowance(
                    address(wstEth), neededStEth
                );

                uint256 receivedWstEth = wstEth.wrap(neededStEth);

                if (receivedWstEth < specifiedAmount) {
                    revert Unavailable("Insufficient wstEth received");
                }

                IERC20(wstEth).safeTransfer(msg.sender, specifiedAmount);

                trade.calculatedAmount = neededStEth;
            } else if (sellToken == address(wstEth)) {
                uint256 neededWstEth =
                    stEth.getSharesByPooledEth(specifiedAmount);

                neededWstEth = neededWstEth + (neededWstEth / BUFFER);

                IERC20(wstEth).safeTransferFrom(
                    msg.sender, address(this), neededWstEth
                );

                uint256 receivedStEth = wstEth.unwrap(neededWstEth);

                if (receivedStEth < specifiedAmount) {
                    revert Unavailable("Insufficient stEth received");
                }

                IERC20(stEth).safeTransfer(msg.sender, specifiedAmount);

                trade.calculatedAmount = neededWstEth;
            } else if (sellToken == address(0) && buyToken == address(stEth)) {
                uint256 neededEth = specifiedAmount + (specifiedAmount / BUFFER);
                uint256 receivedStEth =
                    stEth.submit{value: neededEth}(address(0));

                uint256 stEthTotalSupply = stEth.totalSupply();
                uint256 stEthTotalShares = stEth.getTotalShares();
                uint256 calculatedReceivedStEth =
                    (receivedStEth * stEthTotalSupply) / stEthTotalShares;

                if (calculatedReceivedStEth < specifiedAmount) {
                    revert Unavailable("Insufficient stEth received");
                }

                IERC20(stEth).safeTransfer(msg.sender, specifiedAmount);

                trade.calculatedAmount = neededEth;
            } else if (sellToken == address(0) && buyToken == address(wstEth)) {
                uint256 neededEth = stEth.getPooledEthByShares(specifiedAmount);
                neededEth = neededEth + (neededEth / BUFFER);
                uint256 receivedShares =
                    stEth.submit{value: neededEth}(address(0));
                uint256 stEthTotalSupply = stEth.totalSupply();
                uint256 stEthTotalShares = stEth.getTotalShares();

                uint256 calculatedReceivedStEth =
                    (receivedShares * stEthTotalSupply) / stEthTotalShares;

                IERC20(stEth).safeIncreaseAllowance(
                    address(wstEth), calculatedReceivedStEth
                );

                uint256 receivedWstEth = wstEth.wrap(calculatedReceivedStEth);

                if (receivedWstEth < specifiedAmount) {
                    revert Unavailable("Insufficient wstEth received");
                }

                IERC20(wstEth).safeTransfer(msg.sender, specifiedAmount);

                trade.calculatedAmount = neededEth;
            }

            trade.price = Fraction(specifiedAmount, trade.calculatedAmount);
        } else {
            if (sellToken == address(stEth)) {
                IERC20(stEth).safeTransferFrom(
                    msg.sender, address(this), specifiedAmount
                );

                IERC20(stEth).safeIncreaseAllowance(
                    address(wstEth), specifiedAmount
                );

                uint256 expectedWstEth =
                    stEth.getSharesByPooledEth(specifiedAmount);

                uint256 receivedWstEth = wstEth.wrap(specifiedAmount);

                if (receivedWstEth < expectedWstEth) {
                    revert Unavailable("Insufficient wstEth received");
                }

                IERC20(wstEth).safeTransfer(msg.sender, receivedWstEth);

                trade.calculatedAmount = receivedWstEth;
            } else if (sellToken == address(wstEth)) {
                IERC20(wstEth).safeTransferFrom(
                    msg.sender, address(this), specifiedAmount
                );

                IERC20(wstEth).safeIncreaseAllowance(
                    address(wstEth), specifiedAmount
                );

                uint256 expectedStEth =
                    stEth.getPooledEthByShares(specifiedAmount);

                uint256 receivedStEth = wstEth.unwrap(specifiedAmount);

                if (receivedStEth < expectedStEth) {
                    revert Unavailable("Insufficient stEth received");
                }

                IERC20(stEth).safeTransfer(msg.sender, receivedStEth);

                trade.calculatedAmount = receivedStEth;
            } else if (sellToken == address(0) && buyToken == address(wstEth)) {
                uint256 receivedShares =
                    stEth.submit{value: specifiedAmount}(address(0));

                uint256 stEthTotalSupply = stEth.totalSupply();
                uint256 stEthTotalShares = stEth.getTotalShares();
                uint256 stEthAmount =
                    (receivedShares * stEthTotalSupply) / stEthTotalShares;

                uint256 expectedWstEth = stEth.getSharesByPooledEth(stEthAmount);

                IERC20(stEth).safeIncreaseAllowance(
                    address(wstEth), stEthAmount
                );

                uint256 receivedWstEth = wstEth.wrap(stEthAmount);

                if (receivedWstEth < expectedWstEth) {
                    revert Unavailable("Insufficient wstEth received");
                }

                IERC20(wstEth).safeTransfer(msg.sender, receivedWstEth);

                trade.calculatedAmount = receivedWstEth;
            } else if (sellToken == address(0) && buyToken == address(stEth)) {
                (bool sent,) = address(stEth).call{value: specifiedAmount}("");
                if (!sent) {
                    revert Unavailable(
                        "Ether transfer to stEth contract failed"
                    );
                }

                IERC20(stEth).safeTransfer(msg.sender, specifiedAmount);

                uint256 stEthTotalSupply = stEth.totalSupply();
                uint256 stEthTotalShares = stEth.getTotalShares();
                uint256 sharesAmount =
                    (specifiedAmount * stEthTotalShares) / stEthTotalSupply;
                uint256 stEthAmount =
                    (sharesAmount * stEthTotalSupply) / stEthTotalShares;

                trade.calculatedAmount = stEthAmount;
            }

            trade.price = Fraction(trade.calculatedAmount, specifiedAmount);
        }

        trade.gasUsed = gasBefore - gasleft();
        return trade;
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32, address sellToken, address buyToken)
        external
        view
        override
        checkInputTokens(sellToken, buyToken)
        returns (uint256[] memory limits)
    {
        limits = new uint256[](2);
        if (sellToken == address(stEth)) {
            limits[0] = stEth.totalSupply() * 99 / 100;
            limits[1] = stEth.getSharesByPooledEth(limits[0]);
        } else if (sellToken == address(wstEth)) {
            limits[0] = wstEth.totalSupply() * 99 / 100;
            limits[1] = stEth.getPooledEthByShares(limits[0]);
        } else {
            limits[0] = stEth.getCurrentStakeLimit() * 99 / 100;
            limits[1] = stEth.getSharesByPooledEth(limits[0]);
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
        tokens[1] = address(wstEth);
        tokens[2] = address(stEth);
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
    /// @param sellToken address of the token to sell
    /// @param buyToken address of the token to buy
    /// @return uint256 swap price
    function _getPriceAt(
        uint256 specifiedAmount,
        address sellToken,
        address buyToken
    ) internal view returns (Fraction memory) {
        if (sellToken == address(stEth)) {
            uint256 wstEthAmountOut =
                stEth.getSharesByPooledEth(specifiedAmount);
            return Fraction(wstEthAmountOut, specifiedAmount);
        } else if (sellToken == address(wstEth)) {
            uint256 stEthAmountOut = stEth.getPooledEthByShares(specifiedAmount);
            return Fraction(stEthAmountOut, specifiedAmount);
        } else {
            if (buyToken == address(stEth)) {
                uint256 stEthAmountOut =
                    getStEthAmountByEthAmount(specifiedAmount);
                return Fraction(stEthAmountOut, specifiedAmount);
            } else {
                uint256 wstEthAmountOut =
                    getWstEthAmountByEthAmount(specifiedAmount);
                return Fraction(wstEthAmountOut, specifiedAmount);
            }
        }
    }

    function getStEthAmountByEthAmount(uint256 ethAmountIn)
        public
        view
        returns (uint256 stEthAmountOut)
    {
        uint256 stEthTotalSupply = stEth.totalSupply();
        uint256 stEthTotalShares = stEth.getTotalShares();
        uint256 sharesAmount =
            (ethAmountIn * stEthTotalShares) / stEthTotalSupply;
        stEthAmountOut = (sharesAmount * stEthTotalSupply) / stEthTotalShares;

        return stEthAmountOut;
    }

    function getWstEthAmountByEthAmount(uint256 ethAmountIn)
        public
        view
        returns (uint256 wstEthAmountOut)
    {
        uint256 stEthAmount = getStEthAmountByEthAmount(ethAmountIn);
        wstEthAmountOut = stEth.getSharesByPooledEth(stEthAmount);

        return wstEthAmountOut;
    }
}

/// @dev Wrapped and extended interface for stEth
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

    function getTotalPooledEth() external view returns (uint256);

    function getTotalShares() external view returns (uint256);
}

/// @dev Wrapped interface for wstEth
interface IwstETH is IERC20 {
    function stEth() external view returns (IStETH);

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

    function totalSupply() external view returns (uint256);
}
