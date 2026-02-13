// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";

uint256 constant TOKEN_DECIMALS = 10 ** 18;

/// @title InceptionSwapAdapter
/// @notice This is a Inception adapter implementation compatible with all vaults.
contract InceptionSwapAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    IInceptionVault immutable vault;
    uint256 internal minLimit;
    uint256 internal maxLimit;

    constructor(IInceptionVault _vault, uint256 _minLimit, uint256 _maxLimit) {
        vault = IInceptionVault(_vault);
        minLimit = _minLimit;
        maxLimit = _maxLimit;
    }

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32,
        address,
        address,
        uint256[] memory specifiedAmounts
    ) external view override returns (Fraction[] memory prices) {
        prices = new Fraction[](specifiedAmounts.length);
        uint256 ratio = vault.ratio();

        for (uint256 i = 0; i < specifiedAmounts.length; i++) {
            prices[i] = Fraction((ratio / TOKEN_DECIMALS) * specifiedAmounts[i], 1);
        }
    }

    /// @inheritdoc ISwapAdapter
    function swap(
        bytes32,
        address sellToken,
        address,
        OrderSide,
        uint256 specifiedAmount
    ) external returns (Trade memory trade) {
        if (specifiedAmount == 0) {
            return trade;
        }

        uint256 gasBefore = gasleft();
        trade.calculatedAmount = sell(sellToken, specifiedAmount);
        trade.gasUsed = gasBefore - gasleft();

        trade.price = Fraction(specifiedAmount * vault.ratio(), TOKEN_DECIMALS);
    }

    /// @notice Executes a sell order (vault deposit).
    /// @param amount The amount to be deposited.
    /// @return uint256 The amount of tokens received.
    function sell(address sellToken, uint256 amount)
        internal
        returns (uint256)
    {
        IERC20(sellToken).approve(address(vault), amount);
        uint256 shares = vault.deposit(amount, address(this));
        if (shares == 0) {
            revert Unavailable("Shares is zero!");
        }

        return shares;
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32, address, address)
        external
        view
        override
        returns (uint256[] memory limits)
    {
        limits = new uint256[](2);
        limits[0] = minLimit;
        limits[1] = maxLimit;
    }

    /// @inheritdoc ISwapAdapter
    function getCapabilities(
        bytes32,
        address,
        address
    ) external pure returns (Capability[] memory capabilities) {
        capabilities = new Capability[](2);
        capabilities[0] = Capability.SellOrder;
        capabilities[1] = Capability.PriceFunction;
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(bytes32)
        external
        pure
        returns (address[] memory)
    {
        revert NotImplemented("InceptionStEthSwapAdapter.getTokens");
    }

    /// @inheritdoc ISwapAdapter
    function getPoolIds(uint256, uint256)
        external
        pure
        returns (bytes32[] memory)
    {
        revert NotImplemented("InceptionStEthSwapAdapter.getPoolIds");
    }
}

interface IInceptionVault {
    function ratio() external view returns (uint256);
    function deposit(uint256 amount, address receiver) external returns (uint256);
}

