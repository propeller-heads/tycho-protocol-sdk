// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {ISwapAdapter} from "../interfaces/ISwapAdapter.sol";
import {IERC20, ERC20} from "../../lib/openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import {SafeERC20} from "../../lib/openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import "../../lib/forge-std/src/Test.sol";

library FixedPointMathLib {
    uint256 internal constant MAX_UINT256 = 2 ** 256 - 1;

    function mulDivDown(
        uint256 x,
        uint256 y,
        uint256 denominator
    ) internal pure returns (uint256 z) {
        /// @solidity memory-safe-assembly
        assembly {
            // Equivalent to require(denominator != 0 && (y == 0 || x <=
            // type(uint256).max / y))
            if iszero(
                mul(denominator, iszero(mul(y, gt(x, div(MAX_UINT256, y)))))
            ) {
                revert(0, 0)
            }

            // Divide x * y by the denominator.
            z := div(mul(x, y), denominator)
        }
    }
}

/// @title FraxV3SFraxAdapter
/// @dev Adapter for FraxV3 protocol, supports Frax --> sFrax and sFrax --> Frax
contract FraxV3SFraxAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;
    using FixedPointMathLib for uint256;

    uint256 constant PRECISE_UNIT = 1e18;

    ISFrax immutable sFrax;
    IERC20 immutable frax;

    constructor(ISFrax _sFrax) {
        sFrax = _sFrax;
        frax = IERC20(address(sFrax.asset()));
    }

    /// @dev Check if tokens in input are supported
    modifier onlySupportedTokens(address sellToken, address buyToken) {
        if (
            (sellToken != address(frax) && sellToken != address(sFrax)) ||
            (buyToken != address(frax) && buyToken != address(sFrax)) ||
            buyToken == sellToken
        ) {
            revert Unavailable("This adapter only supports FRAX<->SFRAX swaps");
        }
        _;
    }

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32,
        address sellToken,
        address buyToken,
        uint256[] memory _specifiedAmounts
    )
        external
        view
        override
        onlySupportedTokens(sellToken, buyToken)
        returns (Fraction[] memory _prices)
    {
        _prices = new Fraction[](_specifiedAmounts.length);

        for (uint256 i = 0; i < _specifiedAmounts.length; i++) {
            _prices[i] = getPriceAt(
                sellToken == address(frax),
                _specifiedAmounts[i]
            );
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
        onlySupportedTokens(sellToken, buyToken)
        returns (Trade memory trade)
    {
        if (
            specifiedAmount == 0 ||
            (sellToken == address(frax) && specifiedAmount < 2)
        ) {
            return trade;
        }

        uint256 gasBefore = gasleft();
        if (side == OrderSide.Sell) {
            // sell
            trade.calculatedAmount = sell(sellToken, specifiedAmount);
        } else {
            // buy
            trade.calculatedAmount = buy(sellToken, specifiedAmount);
        }
        trade.gasUsed = gasBefore - gasleft();

        uint256 numerator = sellToken == address(frax)
            ? sFrax.previewDeposit(PRECISE_UNIT)
            : sFrax.previewRedeem(PRECISE_UNIT);

        trade.price = Fraction(numerator, PRECISE_UNIT);
    }

    /// @inheritdoc ISwapAdapter
    /// @dev there is no hard capped limit
    function getLimits(
        bytes32,
        address sellToken,
        address buyToken
    )
        external
        view
        override
        onlySupportedTokens(address(sellToken), address(buyToken))
        returns (uint256[] memory limits)
    {
        limits = new uint256[](2);

        if (sellToken == address(frax)) {
            // Frax --> sFrax
            limits[0] = frax.totalSupply() - frax.balanceOf(address(sFrax));
            limits[1] = sFrax.previewDeposit(limits[0]);
        } else {
            limits[0] = sFrax.totalSupply();
            limits[1] = sFrax.previewRedeem(limits[0]);
        }
    }

    /// @inheritdoc ISwapAdapter
    function getCapabilities(
        bytes32,
        address,
        address
    ) external pure override returns (Capability[] memory capabilities) {
        capabilities = new Capability[](4);
        capabilities[0] = Capability.SellOrder;
        capabilities[1] = Capability.BuyOrder;
        capabilities[2] = Capability.PriceFunction;
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(
        bytes32
    ) external view override returns (address[] memory tokens) {
        tokens = new address[](2);

        tokens[0] = address(frax);
        tokens[1] = address(sFrax);
    }

    /// @inheritdoc ISwapAdapter
    /// @dev Since FraxV3 is a single pool that supports FRAX and SFRAX, we
    /// return it directly
    function getPoolIds(
        uint256,
        uint256
    ) external view override returns (bytes32[] memory ids) {
        ids = new bytes32[](1);
        ids[0] = bytes32(bytes20(address(sFrax)));
    }

    /// @notice Executes a sell order on the contract.
    /// @param sellToken The token being sold.
    /// @param amount The amount to be traded.
    /// @return calculatedAmount The amount of tokens received.
    function sell(
        address sellToken,
        uint256 amount
    ) internal returns (uint256 calculatedAmount) {
        IERC20(sellToken).safeTransferFrom(msg.sender, address(this), amount);
        if (sellToken == address(sFrax)) {
            return sFrax.redeem(amount, msg.sender, address(this));
        } else {
            IERC20(sellToken).safeIncreaseAllowance(address(sFrax), amount);
            return sFrax.deposit(amount, msg.sender);
        }
    }

    /// @notice Executes a buy order on the contract.
    /// @param sellToken The token being sold.
    /// @param amount The amount of buyToken to receive.
    /// @return calculatedAmount The amount of tokens received.
    function buy(
        address sellToken,
        uint256 amount
    ) internal returns (uint256 calculatedAmount) {
        if (sellToken == address(sFrax)) {
            uint256 amountIn = sFrax.previewWithdraw(amount);
            IERC20(sellToken).safeTransferFrom(
                msg.sender,
                address(this),
                amountIn
            );
            return sFrax.withdraw(amount, msg.sender, address(this));
        } else {
            uint256 amountIn = sFrax.previewMint(amount);
            IERC20(sellToken).safeTransferFrom(
                msg.sender,
                address(this),
                amountIn
            );
            IERC20(sellToken).safeIncreaseAllowance(address(sFrax), amountIn);
            return sFrax.mint(amount, msg.sender);
        }
    }

    /// @notice Calculates prices for a specified amount
    /// @param isSellFrax True if selling frax, false if selling sFrax
    /// @param amountIn The amount of the token being sold.
    /// @return (fraction) price as a fraction corresponding to the provided
    /// amount.
    function getPriceAt(
        bool isSellFrax,
        uint256 amountIn
    ) internal view returns (Fraction memory) {
        if (isSellFrax == true) {
            if (amountIn < 2) {
                revert("Amount In must be greater than 1");
            }

            return Fraction(sFrax.previewDeposit(amountIn), amountIn);
        } else {
            return Fraction(sFrax.previewRedeem(amountIn), amountIn);
        }
    }
}

interface ISFrax {
    function previewDeposit(uint256 assets) external view returns (uint256);

    function previewMint(uint256 shares) external view returns (uint256);

    function previewRedeem(uint256 shares) external view returns (uint256);

    function previewWithdraw(uint256 assets) external view returns (uint256);

    function previewDistributeRewards()
        external
        view
        returns (uint256 _rewardToDistribute);

    function pricePerShare() external view returns (uint256);

    function asset() external view returns (ERC20); // FRAX

    function totalSupply() external view returns (uint256);

    function totalAssets() external view returns (uint256);

    function deposit(
        uint256 assets,
        address receiver
    ) external returns (uint256 shares);

    function mint(
        uint256 shares,
        address receiver
    ) external returns (uint256 assets);

    function storedTotalAssets() external view returns (uint256);

    function withdraw(
        uint256 assets,
        address receiver,
        address owner
    ) external returns (uint256 shares);

    function redeem(
        uint256 shares,
        address receiver,
        address owner
    ) external returns (uint256 assets);
}
