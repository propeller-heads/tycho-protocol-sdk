// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {IERC20, SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {Math} from "openzeppelin-contracts/contracts/utils/Math/Math.sol";

/// @title StakeWise Adapter
/// @dev This Adapter supports ETH<->osETH swaps
contract StakeWiseAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    IEthGenesisVault immutable vault;
    IERC20 constant osETH = IERC20(0xf1C9acDc66974dFB6dEcB12aA385b9cD01190E38);

    constructor(address _vault) {
        vault = IEthGenesisVault(_vault);
    }

    /// @dev Check input tokens
    modifier checkInputTokens(address sellToken, address buyToken) {
        if(sellToken == address(0) && buyToken == address(osETH) || sellToken == address(osETH) && buyToken == address(0)) {   
        }
        else {
            revert Unavailable("This adapter only supports ETH<->osETH swaps");
        }
        _;
    }

    /// @dev enable receive to receive ETH
    receive() external payable {}

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        uint256[] memory specifiedAmounts
    ) external view override checkInputTokens(sellToken, buyToken) returns (Fraction[] memory _prices) {
        _prices = new Fraction[](specifiedAmounts.length);

        for (uint256 i = 0; i < specifiedAmounts.length; i++) {
            _prices[i] = getPriceAt(
                sellToken,
                buyToken,
                specifiedAmounts[i],
                true
            );
        }
    }

    /// @inheritdoc ISwapAdapter
    function swap(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external checkInputTokens(sellToken, buyToken) returns (Trade memory trade) {
        if (specifiedAmount == 0) {
            return trade;
        }

        uint256 gasBefore = gasleft();
        if (side == OrderSide.Sell) {
            // sell
            trade.calculatedAmount = sell(sellToken, specifiedAmount);
        } else {
            // buy
            trade.calculatedAmount = buy(buyToken, specifiedAmount);
        }
        trade.gasUsed = gasBefore - gasleft();
        trade.price = getPriceAt(
            sellToken,
            buyToken,
            side == OrderSide.Sell ? specifiedAmount : trade.calculatedAmount,
            false
        );
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(
        bytes32,
        address sellToken,
        address buyToken
    ) external view override checkInputTokens(sellToken, buyToken) returns (uint256[] memory limits) {
        limits = new uint256[](2);
        if (sellToken == address(osETH)) {
            limits[1] = vault.withdrawableAssets();
            limits[0] = vault.convertToShares(limits[1]);
        } else {
            limits[0] = vault.capacity() - vault.totalAssets();
            limits[1] = vault.convertToShares(limits[0]);
        }
    }

    function getCapabilities(
        bytes32 poolId,
        address sellToken,
        address buyToken
    ) external returns (Capability[] memory capabilities) {
        revert NotImplemented("TemplateSwapAdapter.getCapabilities");
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(
        bytes32 poolId
    ) external returns (address[] memory tokens) {
        tokens = new address[](2);
        tokens[0] = address(0);
        tokens[1] = address(osETH);
    }

    function getPoolIds(
        uint256 offset,
        uint256 limit
    ) external returns (bytes32[] memory ids) {
        revert NotImplemented("TemplateSwapAdapter.getPoolIds");
    }

    /// @notice Get swap price
    /// @param sellToken token to sell
    /// @param buyToken token to buy
    /// @param amount amount to swap
    /// @param simulateTrade determine if trade should be simulated (used for price function)
    function getPriceAt(
        address sellToken,
        address buyToken,
        uint256 amount,
        bool simulateTrade
    ) internal view returns (Fraction memory) {
        uint256 numerator;
        uint256 amountToSwap = amount;
        if (!simulateTrade) {
            if (sellToken == address(osETH)) {
                // redeem, amount is osETH to spend
                return Fraction(vault.convertToAssets(amountToSwap), amountToSwap);
            } else {
                // mint, amount is ETH to spend
                return Fraction(vault.convertToShares(amountToSwap), amountToSwap);
            }
        }

        /// @dev we use small unit of amount to swap for post-trade price to avoid rounding errors
        amountToSwap = 10**18 / 100000;

        if (sellToken == address(osETH)) {
            // redeem, amount is osETH to spend
            uint256 sharesAfter = vault.totalShares() - amountToSwap;
            uint256 assetsAfter = vault.totalAssets() -
                vault.convertToAssets(amountToSwap);
            uint256 numerator = Math.mulDiv(amountToSwap, assetsAfter, sharesAfter);
            return Fraction(numerator, amountToSwap);
        } else {
            // mint, amount is ETH to spend
            uint256 assetsAfter = vault.totalAssets() + amountToSwap;
            uint256 totalSharesBefore = vault.totalShares();
            uint256 mintedShares = vault.convertToShares(amountToSwap);
            uint256 sharesAfter = totalSharesBefore +
                Math.mulDiv(
                    assetsAfter,
                    totalSharesBefore + mintedShares,
                    assetsAfter,
                    Math.Rounding.Ceil
                );
            uint256 numerator = Math.mulDiv(
                assetsAfter,
                sharesAfter,
                assetsAfter,
                Math.Rounding.Ceil
            );
            return Fraction(numerator, amountToSwap);
        }
    }

    /// @notice Executes a sell order on a given pool.
    /// @param sellToken The address of the token being sold.
    /// @param amount The amount to be traded.
    /// @return uint256 The amount of tokens received.
    function sell(
        address sellToken,
        uint256 amount
    ) internal returns (uint256) {
        if (sellToken == address(0)) {
            // ETH->osETH
            uint256 sharesBefore = vault.getShares(address(this));

            (bool sent_, ) = address(vault).call{value: amount}("");
            if(!sent_) { revert Unavailable("Ether transfer failed"); }

            uint256 amountOut = vault.getShares(address(this)) - sharesBefore;
            vault.mintOsToken(msg.sender, amountOut, address(0));
            return amountOut;
        } else {
            // osETH->ETH
            osETH.safeTransferFrom(msg.sender, address(this), amount);
            uint256 balBefore = address(this).balance;
            vault.redeemOsToken(amount, address(this), msg.sender);
            uint256 amountOut = address(this).balance - balBefore;

            (bool sent_, ) = address(msg.sender).call{value: amountOut}("");
            if(!sent_) { revert Unavailable("Ether transfer failed"); }

            return amountOut;
        }
    }

    /// @notice Executes a buy order on a given pool.
    /// @param buyToken The address of the token being bought.
    /// @param amountBought The amount of buyToken tokens to buy.
    /// @return uint256 The amount of tokens received.
    function buy(
        address buyToken,
        uint256 amountBought
    ) internal returns (uint256) {
        if (buyToken != address(0)) {
            // ETH->osETH
            uint256 amountIn = vault.convertToAssets(amountBought);

            (bool sent_, ) = address(vault).call{value: amountIn}("");
            if(!sent_) { revert Unavailable("Ether transfer failed"); }

            vault.mintOsToken(msg.sender, amountBought, address(0));
            return amountIn;
        } else {
            // osETH->ETH
            uint256 amountIn = vault.convertToShares(amountBought);
            osETH.safeTransferFrom(msg.sender, address(this), amountIn);

            vault.redeemOsToken(amountIn, address(this), msg.sender);

            (bool sent_, ) = address(msg.sender).call{value: amountBought}("");
            if(!sent_) { revert Unavailable("Ether transfer failed"); }

            return amountIn;
        }
    }
}

interface IEthGenesisVault {
    function convertToShares(uint256 assets) external view returns (uint256);
    function convertToAssets(uint256 shares) external view returns (uint256);
    function getShares(address account) external view returns (uint256);
    function totalAssets() external view returns (uint256);
    function totalShares() external view returns (uint256);
    function withdrawableAssets() external view returns (uint256);
    function deposit(
        address receiver,
        address referrer
    ) external;
    function redeem(
        uint256 shares,
        address receiver
    ) external;
    function redeemOsToken(
        uint256 osTokenShares,
        address owner,
        address receiver
    ) external;
    function mintOsToken(
        address receiver,
        uint256 osTokenShares,
        address referrer
    ) external view returns (uint256);
    function capacity() external view returns (uint256);
}
