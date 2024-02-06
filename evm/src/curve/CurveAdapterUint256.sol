// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {IERC20, ISwapAdapter, CurveAdapter, ICurveRegistry} from "src/curve/CurveAdapter.sol";
import {ERC20} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title Curve Finance Adapter (For V2 pools, with uint256)
/// @dev Adapter version supporting get_dy and other functions with uint256 instead of int128
contract CurveAdapterUint256 is CurveAdapter {
    using SafeERC20 for IERC20;

    constructor(address _registry) CurveAdapter(_registry) {}

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32 _poolId,
        IERC20 _sellToken,
        IERC20 _buyToken,
        uint256[] memory _specifiedAmounts
    ) external view override returns (Fraction[] memory _prices) {
        address poolAddress = address(bytes20(_poolId));
        ICurvePlainPool pool = ICurvePlainPool(poolAddress);
        (int128 sellTokenIndex, int128 buyTokenIndex,) =
            registry.get_coin_indices(poolAddress, address(_sellToken), address(_buyToken));
        _prices = new Fraction[](_specifiedAmounts.length);
        uint256 sellTokenIndexFixed = uint256(uint128(sellTokenIndex));
        uint256 buyTokenIndexFixed = uint256(uint128(buyTokenIndex));

        for(uint256 i = 0; i < _specifiedAmounts.length; i++) {
            _prices[i] = getPriceAtUint256(pool, _specifiedAmounts[i], sellTokenIndexFixed, buyTokenIndexFixed);
        }
    }

    /// @inheritdoc ISwapAdapter
    function swap(
        bytes32 poolId,
        IERC20 sellToken,
        IERC20 buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external override returns (Trade memory trade) {
        if (specifiedAmount == 0) {
            return trade;
        }
        address poolAddress = address(bytes20(poolId));
        ICurvePlainPool pool = ICurvePlainPool(poolAddress);
        (int128 sellTokenIndex, int128 buyTokenIndex,) =
            registry.get_coin_indices(poolAddress, address(sellToken), address(buyToken));
        uint256 gasBefore = gasleft();
        uint256 sellTokenIndexFixed = uint256(uint128(sellTokenIndex));
        uint256 buyTokenIndexFixed = uint256(uint128(buyTokenIndex));

        if(side == OrderSide.Sell) {
            trade.calculatedAmount =
                sell(pool, sellToken, buyToken, sellTokenIndexFixed, buyTokenIndexFixed, specifiedAmount);
        }
        else {
            revert Unavailable("OrderSide.Buy is not available for this adapter");
        }
        trade.gasUsed = gasBefore - gasleft();
        trade.price = getPriceAtUint256(pool, specifiedAmount, sellTokenIndexFixed, buyTokenIndexFixed);
    }

    /// @notice Calculates pool prices for specified amounts
    /// @param pool The pool to calculate token prices in.
    /// @param amountIn The amount of the token being sold.
    /// @param sellTokenIndex The index of token in the pool being sold.
    /// @param buyTokenIndex The index of token being sold among the pool tokens
    /// @return The price as a fraction corresponding to the provided amount.
    function getPriceAtUint256(ICurvePlainPool pool, uint256 amountIn, uint256 sellTokenIndex, uint256 buyTokenIndex)
        internal
        view
        returns (Fraction memory)
    {
        return Fraction(
            pool.get_dy(sellTokenIndex, buyTokenIndex, amountIn),
            amountIn
        );
    }

    /// @notice Executes a sell order on a given pool.
    /// @param pool The pool to trade on.
    /// @param sellToken IERC20 instance of the token being sold.
    /// @param buyToken IERC20 instance of the token being bought.
    /// @param sellTokenIndex The index of token in the pool being sold.
    /// @param buyTokenIndex The index of token being sold among the pool tokens
    /// @param amount The amount to be traded.
    /// @return calculatedAmount The amount of tokens received.
    function sell(
        ICurvePlainPool pool,
        IERC20 sellToken,
        IERC20 buyToken,
        uint256 sellTokenIndex,
        uint256 buyTokenIndex,
        uint256 amount
    ) internal returns (uint256 calculatedAmount) {
        uint256 buyTokenBalBefore = buyToken.balanceOf(address(this));

        sellToken.approve(address(pool), amount);
        sellToken.safeTransferFrom(msg.sender, address(this), amount);

        pool.exchange(sellTokenIndex, buyTokenIndex, amount, 0);
        calculatedAmount = buyToken.balanceOf(address(this)) - buyTokenBalBefore;
        buyToken.safeTransfer(address(msg.sender), calculatedAmount);
    }
}

/// @dev Wrapped ported version of Curve Plain Pool to Solidity
/// For params informations see: https://docs.curve.fi/stableswap-exchange/stableswap/pools/plain_pools/
interface ICurvePlainPool {

    function exchange(uint256 i, uint256 j, uint256 dx, uint256 min_dy) external;

    function get_dy(uint256 i, uint256 j, uint256 dx) external view returns (uint256);

    function balances(uint256 arg0) external view returns (uint256);

    function fee() external view returns (uint256);

}