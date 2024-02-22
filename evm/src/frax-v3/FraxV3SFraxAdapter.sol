// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {IERC20, ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {ERC20} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

library FixedPointMathLib {

    uint256 internal constant MAX_UINT256 = 2**256 - 1;

    function mulDivDown(
        uint256 x,
        uint256 y,
        uint256 denominator
    ) internal pure returns (uint256 z) {
        /// @solidity memory-safe-assembly
        assembly {
            // Equivalent to require(denominator != 0 && (y == 0 || x <= type(uint256).max / y))
            if iszero(mul(denominator, iszero(mul(y, gt(x, div(MAX_UINT256, y)))))) {
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

    ISFrax immutable sFrax;
    IERC20 immutable frax;

    constructor(ISFrax _sFrax) {
        sFrax = _sFrax;
        frax = IERC20(address(sFrax.asset()));
    }

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32,
        IERC20 sellToken,
        IERC20,
        uint256[] memory _specifiedAmounts
    ) external view override returns (Fraction[] memory _prices) {
        if (address(sellToken) != address(frax) && address(sellToken) != address(sFrax)) {
            revert("Sell token not supported");
        }
        
        _prices = new Fraction[](_specifiedAmounts.length);

        bool isSellFrax;
        if(address(sellToken) == address(frax)) {
            isSellFrax = true;
        }
        
        for(uint256 i = 0; i < _specifiedAmounts.length; i++) {
            _prices[i] = getPriceAt(isSellFrax, _specifiedAmounts[i]);
        }
    }

    function swap(
        bytes32,
        IERC20 sellToken,
        IERC20,
        OrderSide side,
        uint256 specifiedAmount
    ) external returns (Trade memory trade) {
        if (specifiedAmount == 0) {
            return trade;
        }

        uint256 gasBefore = gasleft();
        if (side == OrderSide.Sell) { // sell
            trade.calculatedAmount =
                sell(sellToken, specifiedAmount);
        } else { // buy
            trade.calculatedAmount =
                buy(sellToken, specifiedAmount);
        }
        trade.gasUsed = gasBefore - gasleft();
        if(sellToken == frax) {
            trade.price = getPriceAt(true, specifiedAmount);
        }
        else {
            trade.price = getPriceAt(false, specifiedAmount);
        }
    }

    /// @inheritdoc ISwapAdapter
    /// @dev there is no hard capped limit 
    function getLimits(bytes32, IERC20 sellToken, IERC20 buyToken)
        external
        view
        returns (uint256[] memory limits)
    {
        limits = new uint256[](2);

        if(address(sellToken) == address(frax)) { // Frax --> sFrax
            limits[0] = frax.totalSupply();
            limits[1] = sFrax.previewDeposit(limits[0]);
        } else {
            limits[0] = sFrax.totalSupply(); 
            limits[1] = sFrax.previewRedeem(limits[0]);
        }
    }

    /// @inheritdoc ISwapAdapter
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
        returns (IERC20[] memory tokens)
    {
        tokens = new IERC20[](2);

        tokens[0] = frax;
        tokens[1] = IERC20(address(sFrax));
    }

    function getPoolIds(uint256, uint256)
        external
        pure
        returns (bytes32[] memory)
    {
        revert NotImplemented("FraxV3Adapter.getPoolIds");
    }


    /// @notice Get amountIn
    /// @param sellToken token to sell(frax or sfrax)
    /// @param amountOut the amount of buyToken to buy
    /// @return amountIn of sellToken to spend
    function getAmountIn(address sellToken, uint256 amountOut) internal view returns (uint256) {

        if(sellToken == address(frax)) { // FRAX-SFRAX
            return sFrax.previewMint(amountOut);
        }
        else { // SFRAX-FRAX
            return sFrax.previewWithdraw(amountOut);
        }

    }

    /// @notice Get amountOut
    /// @param sellToken token to sell(frax or sfrax)
    /// @param amountIn the amount sellToken to spend
    /// @return amountOut of buyToken to buy(received)
    function getAmountOut(address sellToken, uint256 amountIn) internal view returns (uint256) {

        if(sellToken == address(frax)) { // FRAX-SFRAX
            return sFrax.previewDeposit(amountIn);
        }
        else { // SFRAX-FRAX
            return sFrax.previewRedeem(amountIn);
        }

    }

    /// @notice Calculates prices for a specified amount
    /// @param isSellFrax True if selling frax, false if selling sFrax
    /// @param amountIn The amount of the token being sold.
    /// @return (fraction) price as a fraction corresponding to the provided amount.
    /// @dev change from internal to public for debugging purposes
    function getPriceAt(bool isSellFrax, uint256 amountIn)
        public
        view
        returns (Fraction memory)
    {
        if(isSellFrax = true) {

            uint256 totStoredAssets = sFrax.storedTotalAssets();
            uint256 totMintedShares = sFrax.totalSupply();
            uint256 rewards = sFrax.previewDistributeRewards();
            uint256 newMintedShares = sFrax.previewDeposit(amountIn);

            totStoredAssets += amountIn + rewards;
            totMintedShares += newMintedShares;

            uint256 numerator = amountIn.mulDivDown(totMintedShares, totStoredAssets);

            return Fraction(
                numerator,
                amountIn
            );
        }
        else {

            uint256 totStoredAssets = sFrax.storedTotalAssets();
            uint256 totMintedShares = sFrax.totalSupply();
            uint256 rewards = sFrax.previewDistributeRewards();
            uint256 fraxAmountRedeemed = sFrax.previewRedeem(amountIn);

            totStoredAssets = totStoredAssets + rewards - fraxAmountRedeemed;
            totMintedShares -= amountIn;

            uint256 numerator = amountIn.mulDivDown(totStoredAssets, totMintedShares);
            

            return Fraction(
                numerator,
                amountIn
            );
        }
    }

    /// @notice Executes a sell order on the contract.
    /// @param sellToken The token being sold.
    /// @param amount The amount to be traded.
    /// @return calculatedAmount The amount of tokens received.
    function sell(
        IERC20 sellToken,
        uint256 amount
    ) internal returns (uint256 calculatedAmount) {
        uint256 amountOut = getAmountOut(address(sellToken), amount);

        sellToken.safeTransferFrom(msg.sender, address(this), amount);
        if(address(sellToken) == address(sFrax)) {
            sFrax.redeem(amount, msg.sender, address(this));
        }
        else {
            sellToken.approve(address(sFrax), amount);
            sFrax.deposit(amount, msg.sender);
        }
        return amountOut;
    }

    /// @notice Executes a buy order on the contract.
    /// @param sellToken The token being sold.
    /// @param amount The amount of buyToken to receive.
    /// @return calculatedAmount The amount of tokens received.
    function buy(
        IERC20 sellToken,
        uint256 amount
    ) internal returns (uint256 calculatedAmount) {
        uint256 amountIn = getAmountIn(address(sellToken), amount);

        sellToken.safeTransferFrom(msg.sender, address(this), amount);
        if(address(sellToken) == address(sFrax)) {
            sFrax.withdraw(amount, msg.sender, address(this));
        }
        else {
            sellToken.approve(address(sFrax), amount);
            sFrax.mint(amount, msg.sender);
        }
        return amountIn;
    }

}

interface ISFrax {

    function previewDeposit(uint256 assets) external view returns (uint256);

    function previewMint(uint256 shares) external view returns (uint256);

    function previewRedeem(uint256 shares) external view returns (uint256);

    function previewWithdraw(uint256 assets) external view returns (uint256);

    function previewDistributeRewards() external view returns (uint256 _rewardToDistribute);

    function pricePerShare() external view returns (uint256);

    function asset() external view returns (ERC20); // FRAX

    function totalSupply() external view returns (uint256);

    function totalAssets() external view returns (uint256);

    function deposit(uint256 assets, address receiver) external returns (uint256 shares);

    function mint(uint256 shares, address receiver) external returns (uint256 assets);

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
