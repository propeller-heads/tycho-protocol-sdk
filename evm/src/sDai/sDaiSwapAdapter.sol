// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {IERC20Metadata} from
    "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {
    IERC20,
    SafeERC20
} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title sDaiSwapAdapter

contract sDaiSwapAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    ISavingsDai immutable savingsDai;
    IDai immutable dai;

    constructor(address savingsDai_) {
        savingsDai = ISavingsDai(savingsDai_);
        dai = IDai(savingsDai.asset());
    }

    /// @dev Check if swap between provided sellToken and buyToken are supported
    /// by this adapter
    modifier checkInputTokens(address sellToken, address buyToken) {
        if (sellToken == buyToken) {
            revert Unavailable(
                "This pool only supports DAI<->sDAI swaps"
            );
        }
        if (sellToken == savingsDai.asset() && buyToken != address(savingsDai)) {
            revert Unavailable(
                "This pool only supports DAI<->sDAI swaps"
            );
        }
        if (sellToken == address(savingsDai) && buyToken != savingsDai.asset()) {
            revert Unavailable(
                "This pool only supports DAI<->sDAI swaps"
            );
        }
        
        _;
    }

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32,
        address sellToken,
        address buyToken,
        uint256[] memory specifiedAmounts
    ) external view override returns (Fraction[] memory prices) {
        prices = new Fraction[](specifiedAmounts.length);

        for (uint256 i = 0; i < specifiedAmounts.length; i++) {
            prices[i] = getPriceAt(sellToken, specifiedAmounts[i]);
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
        override
        checkInputTokens(sellToken, buyToken)
        returns (Trade memory trade) 
    {
        
        if (specifiedAmount == 0) {
            return trade;
        }
        uint256 gasBefore = gasleft();
        if (side == OrderSide.Sell) {
            trade.calculatedAmount = sell(IERC20(sellToken), specifiedAmount);
        } else {
            trade.calculatedAmount = buy(IERC20(sellToken), specifiedAmount);
        }

        trade.gasUsed = gasBefore - gasleft();

        if (side == OrderSide.Sell) {
            trade.price = getPriceAt(sellToken, specifiedAmount);
        } else {
            trade.price = getPriceAt(sellToken, trade.calculatedAmount);
        }
    }


    /// @notice Get Swap price only for testing purposes
    /// @param sellToken token to sell.
    /// @param amountIn The amount of the token being sold.
    function getPriceSwapAt(address sellToken, uint256 amountIn) external view returns (Fraction memory) {
        if (sellToken == savingsDai.asset()) {
            return Fraction(savingsDai.previewDeposit(amountIn), amountIn);
        } else {
            return Fraction(savingsDai.previewRedeem(amountIn), amountIn);
        }

    }

    /// @inheritdoc ISwapAdapter
    /// @dev Limits are underestimated to 90% of totalSupply as both Dai and sDai 
    // have no limits but revert in some cases
    function getLimits(bytes32, address sellToken, address buyToken)
        external
        view
        override
        returns (uint256[] memory limits)
    {
        limits = new uint256[](2);
        
        if (sellToken == savingsDai.asset()) {
            limits[0] = dai.totalSupply() * 90/100;
            limits[1] = savingsDai.previewDeposit(limits[0]);
        } else {
            limits[0] = savingsDai.totalSupply() * 90/100;
            limits[1] = savingsDai.previewRedeem(limits[0]);
        }
    }

    /// @inheritdoc ISwapAdapter
    function getCapabilities(bytes32, address, address)
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
        returns (address[] memory tokens)
    {
        tokens = new address[](2);
        tokens[0] = savingsDai.asset();
        tokens[1] = address(savingsDai);
    }

    /// @inheritdoc ISwapAdapter
    function getPoolIds(uint256, uint256)
        external
        view
        override
        returns (bytes32[] memory ids)
    {
        ids = new bytes32[](1);
        ids[0] = bytes20(address(savingsDai));
    }

    /// @notice Executes a sell order on the contract.
    /// @param sellToken The token being sold.
    /// @param amount The amount to be traded.
    /// @return calculatedAmount The amount of tokens received.
    function sell(IERC20 sellToken, uint256 amount)
        internal
        returns (uint256 calculatedAmount)
    {
    
        if (address(sellToken) == savingsDai.asset()) {
            sellToken.safeIncreaseAllowance(address(savingsDai), amount);
            sellToken.safeTransferFrom(msg.sender, address(this), amount);
            return savingsDai.deposit(amount, msg.sender);
        }

        if (address(sellToken) == address(savingsDai)) {
            sellToken.safeIncreaseAllowance(address(savingsDai), amount);
            sellToken.safeTransferFrom(msg.sender, address(this), amount);
            return savingsDai.redeem(amount, msg.sender, address(this));
        }
    }

    /// @notice Executes a buy order on the contract.
    /// @param sellToken The token being sold.
    /// @param amount The amount of buyToken to receive.
    /// @return calculatedAmount The amount of sellToken sold.
    function buy(IERC20 sellToken, uint256 amount)
        internal
        returns (uint256 calculatedAmount)
    {

        if (address(sellToken) == savingsDai.asset()) {
            uint256 amountIn = savingsDai.previewMint(amount);
            sellToken.safeIncreaseAllowance(address(savingsDai), amountIn);
            sellToken.safeTransferFrom(msg.sender, address(this), amountIn);
            return savingsDai.mint(amount, msg.sender);
        } else {
            uint256 amountIn = savingsDai.previewWithdraw(amount);
            sellToken.safeIncreaseAllowance(address(savingsDai), amountIn);
            sellToken.safeTransferFrom(msg.sender, address(this), amountIn);
            return savingsDai.withdraw(amount, msg.sender, address(this));
        }

    }

    /// @notice Get swap price
    /// @param sellToken token to sell
    /// @param amountIn The amount of the token being sold.
    function getPriceAt(address sellToken, uint256 amountIn) internal view returns (Fraction memory) {
        if (sellToken == savingsDai.asset()) {
            return Fraction(savingsDai.previewDeposit(amountIn), amountIn);
        } else {
            return Fraction(savingsDai.previewRedeem(amountIn), amountIn);
        }

    }

}

interface ISavingsDai {

    function asset() external view returns (address);

    function decimals() external view returns (uint8);

    function maxMint(address) external pure returns (uint256);

    function maxRedeem(address) external view returns (uint256);

    function previewMint(uint256 shares) external view returns (uint256);

    function previewWithdraw(uint256 assets) external view returns (uint256);

    function previewDeposit(uint256 assets) external view returns (uint256);

    function previewRedeem(uint256 shares) external view returns (uint256);

    function totalSupply() external pure returns (uint256);

    function deposit(uint256 assets, address receiver) external returns (uint256 shares);

    function mint(uint256 shares, address receiver) external returns (uint256 assets);

    function withdraw(uint256 assets, address receiver, address owner) external returns (uint256 shares);

    function redeem(uint256 shares, address receiver, address owner) external returns (uint256 assets);

}

interface IDai {

    function totalSupply() external pure returns (uint256);

}