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
    using SafeERC20 for ISavingsDai;

    uint256 constant PRECISE_UNIT = 10 ** 18;

    ISavingsDai immutable savingsDai;
    IERC20 immutable dai;

    constructor(address savingsDai_) {
        savingsDai = ISavingsDai(savingsDai_);
        dai = IERC20(savingsDai.asset());
    }

    /// @dev Check if swap between provided sellToken and buyToken are supported
    /// by this adapter
    modifier checkInputTokens(address sellToken, address buyToken) {
        if (sellToken == buyToken) {
            revert Unavailable("This pool only supports DAI<->sDAI swaps");
        }
        if (sellToken == savingsDai.asset() && buyToken != address(savingsDai))
        {
            revert Unavailable("This pool only supports DAI<->sDAI swaps");
        }
        if (sellToken == address(savingsDai) && buyToken != savingsDai.asset())
        {
            revert Unavailable("This pool only supports DAI<->sDAI swaps");
        }

        _;
    }

    /// @inheritdoc ISwapAdapter
    /// @notice price doesn't change after swap for any given quantity
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

        Fraction memory outputPrice = getPriceAt(sellToken);

        for (uint256 i = 0; i < specifiedAmounts.length; i++) {
            prices[i] = outputPrice;
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
            trade.calculatedAmount = buy(IERC20(buyToken), specifiedAmount);
        }

        trade.gasUsed = gasBefore - gasleft();

        if (side == OrderSide.Sell) {
            trade.price = getPriceAt(sellToken);
        } else {
            trade.price = getPriceAt(buyToken);
        }
    }

    /// @inheritdoc ISwapAdapter
    /// @dev Limits are underestimated to 90% of totalSupply as both Dai and
    /// sDai
    // have no limits but revert in some cases
    function getLimits(bytes32, address sellToken, address)
        external
        view
        override
        returns (uint256[] memory limits)
    {
        limits = new uint256[](2);

        if (sellToken == address(dai)) {
            limits[0] = (dai.totalSupply() - dai.balanceOf(address(savingsDai)))
                * 90 / 100;
            limits[1] = savingsDai.previewDeposit(limits[0]);
        } else {
            limits[0] = savingsDai.totalSupply() * 90 / 100;
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
        tokens[0] = address(dai);
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
        sellToken.safeTransferFrom(msg.sender, address(this), amount);

        if (address(sellToken) == address(dai)) {
            sellToken.safeIncreaseAllowance(address(savingsDai), amount);
        }

        return address(sellToken) == address(dai)
            ? savingsDai.deposit(amount, msg.sender)
            : savingsDai.redeem(amount, msg.sender, address(this));
    }

    /// @notice Executes a buy order on the contract.
    /// @param buyToken The token being bought.
    /// @param amount The amount of buyToken to receive.
    /// @return calculatedAmount The amount of sellToken sold.
    function buy(IERC20 buyToken, uint256 amount)
        internal
        returns (uint256 calculatedAmount)
    {
        if (address(buyToken) == address(savingsDai)) {
            // DAI-sDAI
            uint256 amountIn = savingsDai.previewMint(amount);
            dai.safeTransferFrom(msg.sender, address(this), amountIn);
            dai.safeIncreaseAllowance(address(savingsDai), amountIn);
            return savingsDai.mint(amount, msg.sender);
        } else {
            // sDAI-DAI
            uint256 amountIn = savingsDai.previewWithdraw(amount);
            savingsDai.safeTransferFrom(msg.sender, address(this), amountIn);
            return savingsDai.withdraw(amount, msg.sender, address(this));
        }
    }

    /// @notice Get swap price
    /// @param sellToken token to sell
    function getPriceAt(address sellToken)
        internal
        view
        returns (Fraction memory)
    {
        if (sellToken == address(dai)) {
            return
                Fraction(savingsDai.previewDeposit(PRECISE_UNIT), PRECISE_UNIT);
        } else {
            return
                Fraction(savingsDai.previewRedeem(PRECISE_UNIT), PRECISE_UNIT);
        }
    }
}

interface ISavingsDai is IERC20 {
    function asset() external view returns (address);

    function decimals() external view returns (uint8);

    function maxMint(address) external pure returns (uint256);

    function maxRedeem(address) external view returns (uint256);

    function previewMint(uint256 shares) external view returns (uint256);

    function previewWithdraw(uint256 assets) external view returns (uint256);

    function previewDeposit(uint256 assets) external view returns (uint256);

    function previewRedeem(uint256 shares) external view returns (uint256);

    function totalSupply() external pure returns (uint256);

    function deposit(uint256 assets, address receiver)
        external
        returns (uint256 shares);

    function mint(uint256 shares, address receiver)
        external
        returns (uint256 assets);

    function withdraw(uint256 assets, address receiver, address owner)
        external
        returns (uint256 shares);

    function redeem(uint256 shares, address receiver, address owner)
        external
        returns (uint256 assets);
}
