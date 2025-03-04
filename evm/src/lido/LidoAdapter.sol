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

        // Buy order: User specifies how much of buyToken they want to receive
        if (side == OrderSide.Buy) {
            if (sellToken == address(stEth)) {
                // stEth -> wstEth (Buy)
                // Calculate exact amount of stEth needed to get the specified
                // amount of wstEth
                uint256 neededStEth = stEth.getPooledEthByShares(specifiedAmount);

                // Add a small buffer to account for potential rounding errors
                // (0.0001%)
                neededStEth = neededStEth + (neededStEth / BUFFER);

                // Transfer stEth from user to adapter
                IERC20(stEth).safeTransferFrom(
                    msg.sender, address(this), neededStEth
                );

                // Approve wstEth contract to use our stEth
                IERC20(stEth).safeIncreaseAllowance(
                    address(wstEth), neededStEth
                );

                // Wrap stEth to get wstEth
                uint256 receivedWstEth = wstEth.wrap(neededStEth);

                // Ensure we received enough wstEth
                if (receivedWstEth < specifiedAmount) {
                    revert Unavailable("Insufficient wstEth received");
                }

                // Transfer exactly the specified amount to the user
                IERC20(wstEth).safeTransfer(msg.sender, specifiedAmount);

                // Return the total stEth used for the trade
                trade.calculatedAmount = neededStEth;
            } else if (
                sellToken == address(wstEth)
            ) {
                // wstEth -> stEth (Buy)
                // calculate the amount of wstEth needed to get the specified
                // amount of stEth
                uint256 neededWstEth = stEth.getSharesByPooledEth(specifiedAmount);

                // Add a small buffer to account for potential rounding errors
                // (0.0001%)
                neededWstEth = neededWstEth + (neededWstEth / BUFFER);

                // Transfer wstEth from user to adapter
                IERC20(wstEth).safeTransferFrom(
                    msg.sender, address(this), neededWstEth
                );

                // Unwrap wstEth to get stEth
                uint256 receivedStEth = wstEth.unwrap(neededWstEth);

                // Ensure we received enough stEth
                if (receivedStEth < specifiedAmount) {
                    revert Unavailable("Insufficient stEth received");
                }

                // Transfer exactly the specified amount to the user
                IERC20(stEth).safeTransfer(msg.sender, specifiedAmount);

                // Return the total wstEth used for the trade
                trade.calculatedAmount = neededWstEth;
            } else if (sellToken == address(0) && buyToken == address(stEth)) {
                // Eth -> stEth (Buy)
                // Submit Eth to get stEth
                // Add a small buffer to account for potential rounding errors
                // (0.0001%)
                uint256 neededEth = specifiedAmount + (specifiedAmount / BUFFER);
                uint256 receivedStEth =
                    stEth.submit{value: neededEth}(address(0));

                // Calculate the stEth amount from shares
                uint256 stEthTotalSupply = stEth.totalSupply();
                uint256 stEthTotalShares = stEth.getTotalShares();
                uint256 calculatedReceivedStEth = ( receivedStEth *
                        stEthTotalSupply) / stEthTotalShares;

                // Ensure we received enough stEth
                if (calculatedReceivedStEth < specifiedAmount) {
                    revert Unavailable("Insufficient stEth received");
                }

                // Transfer stEth to the user
                IERC20(stEth).safeTransfer(msg.sender, specifiedAmount);

                // Return the total Eth used for the trade
                trade.calculatedAmount = neededEth;
            } else if (sellToken == address(0) && buyToken == address(wstEth)) {
                // Eth -> wstEth (Buy)
                // Calculate the exact amount of shares needed for the specified
                // wstEth amount
                // wstEth amount = shares amount, as per the wrap function
                uint256 neededEth = stEth.getPooledEthByShares(specifiedAmount);
                neededEth = neededEth + (neededEth / BUFFER);
                console.log("A | neededEth", neededEth);
                // Submit Eth to get stEth
                console.log("A | Eth balance before submit", (address(this)).balance);
                uint256 receivedShares = stEth.submit{value: neededEth}(address(0));
                console.log("A | Eth balance after submit", (address(this)).balance);

                // Calculate the amount of stEth needed based on the shares
                uint256 stEthTotalSupply = stEth.totalSupply();
                uint256 stEthTotalShares = stEth.getTotalShares();

                uint256 calculatedReceivedStEth = ( receivedShares *
                        stEthTotalSupply) / stEthTotalShares;

                // Approve wstEth contract to use our stEth
                IERC20(stEth).safeIncreaseAllowance(
                    address(wstEth), calculatedReceivedStEth
                );

                console.log("A | receivedStEth", stEth.balanceOf(address(this)));
                // Wrap stEth to get wstEth
                uint256 receivedWstEth = wstEth.wrap(calculatedReceivedStEth);
                console.log("A | neededEth", neededEth);
                console.log("A | calculatedReceivedStEth", calculatedReceivedStEth);
                console.log("A | receivedWstEth", receivedWstEth);
                console.log("A | specifiedAmount", specifiedAmount);
                // Ensure we received enough wstEth
                if (receivedWstEth < specifiedAmount) {
                    revert Unavailable("Insufficient wstEth received");
                }

                // Transfer exactly the specified amount to the user
                IERC20(wstEth).safeTransfer(msg.sender, specifiedAmount);

                // Return the total Eth used for the trade
                trade.calculatedAmount = neededEth;
            }

            // Price is always calculated as buyTokenAmount / sellTokenAmount
            trade.price = Fraction(specifiedAmount, trade.calculatedAmount);
        }
        // Sell order: User specifies how much of sellToken they want to sell
        else {
            if (sellToken == address(stEth) && buyToken == address(wstEth)) {
                // stEth -> wstEth (Sell)
                // Transfer stEth from user to adapter
                IERC20(stEth).safeTransferFrom(
                    msg.sender, address(this), specifiedAmount
                );

                // Approve wstEth contract to use our stEth
                IERC20(stEth).safeIncreaseAllowance(
                    address(wstEth), specifiedAmount
                );

                // Calculate expected wstEth amount
                uint256 expectedWstETH =
                    wstEth.getWstETHByStETH(specifiedAmount);

                // Wrap stEth to get wstEth
                uint256 receivedWstEth = wstEth.wrap(specifiedAmount);

                // Ensure we received enough wstEth
                if (receivedWstEth < expectedWstETH) {
                    revert Unavailable("Insufficient wstEth received");
                }

                // Transfer wstEth to the user
                IERC20(wstEth).safeTransfer(msg.sender, receivedWstEth);

                // Return the total wstEth received
                trade.calculatedAmount = receivedWstEth;
            } else if (
                sellToken == address(wstEth) && buyToken == address(stEth)
            ) {
                // wstEth -> stEth (Sell)
                // Transfer wstEth from user to adapter
                IERC20(wstEth).safeTransferFrom(
                    msg.sender, address(this), specifiedAmount
                );

                // Approve wstEth contract for unwrapping
                IERC20(wstEth).safeIncreaseAllowance(
                    address(wstEth), specifiedAmount
                );

                // Calculate expected stEth amount
                uint256 expectedStETH = wstEth.getStETHByWstETH(specifiedAmount);

                // Unwrap wstEth to get stEth
                uint256 receivedStEth = wstEth.unwrap(specifiedAmount);

                // Transfer stEth to the user
                IERC20(stEth).safeTransfer(msg.sender, receivedStEth);

                // Return the total stEth received
                trade.calculatedAmount = receivedStEth;
            } else if (sellToken == address(0) && buyToken == address(wstEth)) {
                // Eth -> wstEth (Sell)
                // Submit Eth to get stEth
                uint256 receivedShares =
                    stEth.submit{value: specifiedAmount}(address(0));

                // Calculate the stEth amount from shares
                uint256 stEthTotalSupply = stEth.totalSupply();
                uint256 stEthTotalShares = stEth.getTotalShares();
                uint256 stETHAmount =
                    (receivedShares * stEthTotalSupply) / stEthTotalShares;

                // Calculate expected wstEth amount
                uint256 expectedWstETH = stEth.getSharesByPooledEth(stETHAmount);

                // Approve wstEth contract to use our stEth
                IERC20(stEth).safeIncreaseAllowance(
                    address(wstEth), stETHAmount
                );

                // Wrap stEth to get wstEth
                uint256 receivedWstEth = wstEth.wrap(stETHAmount);

                // Ensure we received enough wstEth
                if (receivedWstEth < expectedWstETH - 3) {
                    revert Unavailable("Insufficient wstEth received");
                }

                // Transfer wstEth to the user
                IERC20(wstEth).safeTransfer(msg.sender, receivedWstEth);

                // Return the total wstEth received
                trade.calculatedAmount = receivedWstEth;
            } else if (sellToken == address(0) && buyToken == address(stEth)) {
                // Eth -> stEth (Sell)
                // Submit Eth to get stEth
                (bool sent,) = address(stEth).call{value: specifiedAmount}("");
                if (!sent) {
                    revert Unavailable(
                        "Ether transfer to stEth contract failed"
                    );
                }

                // Transfer stEth to the user
                IERC20(stEth).safeTransfer(msg.sender, specifiedAmount);

                // Calculate the stEth amount
                uint256 stEthTotalSupply = stEth.totalSupply();
                uint256 stEthTotalShares = stEth.getTotalShares();
                uint256 sharesAmount =
                    (specifiedAmount * stEthTotalShares) / stEthTotalSupply;
                uint256 stETHAmount =
                    (sharesAmount * stEthTotalSupply) / stEthTotalShares;

                // Return the total stEth received
                trade.calculatedAmount = stETHAmount;
            }

            // Price is always calculated as buyTokenAmount / sellTokenAmount
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
        uint256 currentStakeLimitStETH = 10 * 1e18; //  stEth.getCurrentStakeLimit();
        // // same
        // as Eth stake limit
        uint256 currentStakeLimitWstETH = 10 * 1e18;
        // wstEth.getWstETHByStETH(currentStakeLimitStETH);

        limits = new uint256[](2);
        if (sellToken == address(stEth)) {
            // stEth-wstEth
            limits[0] = currentStakeLimitStETH;
            limits[1] = currentStakeLimitWstETH;
        } else if (sellToken == address(wstEth)) {
            // wstEth-stEth
            limits[0] = currentStakeLimitWstETH;
            limits[1] = currentStakeLimitStETH;
        } else {
            // Eth-wstEth and Eth-stEth

            /// @dev Fix for side == Buy, because getTotalPooledEthByShares
            /// would be higher than currentStakeLimit if using the limit as
            /// amount
            uint256 pooledEthByShares_ =
                stEth.getPooledEthByShares(currentStakeLimitStETH);
            if (pooledEthByShares_ > currentStakeLimitStETH) {
                currentStakeLimitStETH = currentStakeLimitStETH
                    - (pooledEthByShares_ - currentStakeLimitStETH);
            }

            limits[0] = currentStakeLimitStETH;
            if (buyToken == address(stEth)) {
                limits[1] = currentStakeLimitStETH;
            } else {
                limits[1] = currentStakeLimitWstETH;
            }
        }
    }

    function getAmountInForWstETHSpecifiedAmount(uint256 wstETHAmount)
        public
        view
        returns (uint256 ethAmountIn)
    {
        ethAmountIn = wstEth.getStETHByWstETH(wstETHAmount) + 100;
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
    /// @param sellTokenAddress address of the token to sell
    /// @param buyTokenAddress address of the token to buy
    /// @return uint256 swap price
    function _getPriceAt(
        uint256 specifiedAmount,
        address sellTokenAddress,
        address buyTokenAddress
    ) internal view returns (Fraction memory) {
        uint256 amount0;
        uint256 amount1;

        if (sellTokenAddress == address(stEth)) {
            // stEth-wstEth, Eth is not possible as checked through
            // checkInputTokens
            amount0 = wstEth.getWstETHByStETH(specifiedAmount);
            amount1 = wstEth.getStETHByWstETH(amount0);
        } else if (sellTokenAddress == address(wstEth)) {
            // wstEth-stEth, Eth is not possible as checked through
            // checkInputTokens
            amount0 = wstEth.getStETHByWstETH(specifiedAmount);
            amount1 = wstEth.getWstETHByStETH(amount0);
        } else {
            // Eth (address(0))
            if (buyTokenAddress == address(stEth)) {
                amount0 = stEth.getSharesByPooledEth(specifiedAmount);
                amount1 = stEth.getPooledEthByShares(amount0);
            } else {
                uint256 stETHAmount =
                    stEth.getSharesByPooledEth(specifiedAmount);
                amount0 = wstEth.getWstETHByStETH(stETHAmount);
                amount1 =
                    stEth.getPooledEthByShares(wstEth.getStETHByWstETH(amount0));
            }
        }

        return Fraction(amount0, amount1);
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
}
