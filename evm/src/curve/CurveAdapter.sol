// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {IERC20, ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

/// @dev custom RESERVE_LIMIT_FACTOR for limits for this adapter(underestimate)
uint256 constant RESERVE_LIMIT_FACTOR = 10;

/// @title Curve Finance Adapter
contract CurveAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    ICurveRegistry public registry;

    constructor(address _registry) {
        registry = ICurveRegistry(_registry);
    }

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32 _poolId,
        IERC20 _sellToken,
        IERC20 _buyToken,
        uint256[] memory _specifiedAmounts
    ) external virtual view override returns (Fraction[] memory _prices) {
        address poolAddress = address(bytes20(_poolId));
        ICurvePlainPool pool = ICurvePlainPool(poolAddress);
        (int128 sellTokenIndex, int128 buyTokenIndex,) =
            registry.get_coin_indices(poolAddress, address(_sellToken), address(_buyToken));
        _prices = new Fraction[](_specifiedAmounts.length);

        for(uint256 i = 0; i < _specifiedAmounts.length; i++) {
            _prices[i] = getPriceAt(pool, _specifiedAmounts[i], sellTokenIndex, buyTokenIndex);
        }
    }

    // /// @inheritdoc ISwapAdapter
    // function swap(
    //     bytes32 poolId,
    //     IERC20 sellToken,
    //     IERC20 buyToken,
    //     OrderSide side,
    //     uint256 specifiedAmount
    // ) external virtual override returns (Trade memory trade) {
    //     if (specifiedAmount == 0) {
    //         return trade;
    //     }
    //     address poolAddress = address(bytes20(poolId));
    //     ICurvePlainPool pool = ICurvePlainPool(poolAddress);
    //     (int128 sellTokenIndex, int128 buyTokenIndex,) =
    //         registry.get_coin_indices(poolAddress, address(sellToken), address(buyToken));
    //     uint256 gasBefore = gasleft();

    //     if(side == OrderSide.Sell) {
    //         trade.calculatedAmount =
    //             sell(pool, sellToken, buyToken, sellTokenIndex, buyTokenIndex, specifiedAmount);
    //     }
    //     else {
    //         revert Unavailable("OrderSide.Buy is not available for this adapter");
    //     }
    //     trade.gasUsed = gasBefore - gasleft();
    //     trade.price = getPriceAt(pool, specifiedAmount, sellTokenIndex, buyTokenIndex);
    // }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32 poolId, IERC20 sellToken, IERC20 buyToken)
        external
        view
        override
        returns (uint256[] memory limits)
    {
        address poolAddress = address(bytes20(poolId));
        (int128 sellTokenIndex, int128 buyTokenIndex,) =
            registry.get_coin_indices(poolAddress, address(sellToken), address(buyToken));
        uint256[8] memory poolBalances = registry.get_balances(poolAddress);
        
        limits = new uint256[](2);
        uint256 sellTokenIndexFixed = uint256(uint128(sellTokenIndex));
        uint256 buyTokenIndexFixed = uint256(uint128(buyTokenIndex));
        limits[0] = poolBalances[sellTokenIndexFixed] / RESERVE_LIMIT_FACTOR;
        limits[1] = poolBalances[buyTokenIndexFixed] / RESERVE_LIMIT_FACTOR;
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
        capabilities[2] = Capability.PriceFunction;
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(bytes32 poolId)
        external
        view
        override
        returns (IERC20[] memory tokens)
    {
        address[8] memory coins = registry.get_coins(address(bytes20(poolId)));
        tokens = new IERC20[](coins.length);
        for(uint256 i = 0; i < coins.length; i++) {
            tokens[i] = IERC20(coins[i]);
        }
    }

    /// @inheritdoc ISwapAdapter
    function getPoolIds(uint256 offset, uint256 limit)
        external
        view
        override
        returns (bytes32[] memory ids)
    {
        uint256 endIdx = offset + limit;
        if (endIdx > registry.pool_count()) {
            endIdx = registry.pool_count();
        }
        ids = new bytes32[](endIdx - offset);
        for (uint256 i = 0; i < ids.length; i++) {
            ids[i] = bytes20(registry.pool_list(offset + i));
        }
    }

    /// @notice Calculates pool prices for specified amounts
    /// @param pool The pool to calculate token prices in.
    /// @param amountIn The amount of the token being sold.
    /// @param sellTokenIndex The index of token in the pool being sold.
    /// @param buyTokenIndex The index of token being sold among the pool tokens
    /// @return The price as a fraction corresponding to the provided amount.
    function getPriceAt(ICurvePlainPool pool, uint256 amountIn, int128 sellTokenIndex, int128 buyTokenIndex)
        internal
        view
        returns (Fraction memory)
    {
        return Fraction(
            pool.get_dy(sellTokenIndex, buyTokenIndex, amountIn),
            amountIn
        );

    }

    // /// @notice Executes a sell order on a given pool.
    // /// @param pool The pool to trade on.
    // /// @param sellToken IERC20 instance of the token being sold.
    // /// @param buyToken IERC20 instance of the token being bought.
    // /// @param sellTokenIndex The index of token in the pool being sold.
    // /// @param buyTokenIndex The index of token being sold among the pool tokens
    // /// @param amount The amount to be traded.
    // /// @return calculatedAmount The amount of tokens received.
    // function sell(
    //     ICurvePlainPool pool,
    //     IERC20 sellToken,
    //     IERC20 buyToken,
    //     int128 sellTokenIndex,
    //     int128 buyTokenIndex,
    //     uint256 amount
    // ) internal returns (uint256 calculatedAmount) {
    //     uint256 buyTokenBalBefore = buyToken.balanceOf(address(this));

    //     sellToken.approve(address(pool), amount);
    //     sellToken.safeTransferFrom(msg.sender, address(this), amount);

    //     pool.exchange(sellTokenIndex, buyTokenIndex, amount, 0);
    //     calculatedAmount = buyToken.balanceOf(address(this)) - buyTokenBalBefore;
    //     buyToken.safeTransfer(address(msg.sender), calculatedAmount);
    // }
}

/// @dev Wrapped ported version of Curve Plain Pool to Solidity
/// For params informations see: https://docs.curve.fi/stableswap-exchange/stableswap/pools/plain_pools/
interface ICurvePlainPool {

    function exchange(int128 i, int128 j, uint256 dx, uint256 min_dy) external;

    function get_dy(int128 i, int128 j, uint256 dx) external view returns (uint256);

    function balances(uint256 arg0) external view returns (uint256);

    function fee() external view returns (uint256);

}

/// @dev Wrapped ported version of CurveRegistry to Solidity
/// For params informations see: https://docs.curve.fi/registry/MetaRegistryAPI/
interface ICurveRegistry {

    function find_pool_for_coins(address _from, address _to, uint256 i) external view returns (address);

    function pool_count() external view returns (uint256);

    function pool_list(uint256 _index) external view returns (address);

    function get_fees(address _pool) external view returns (
        uint256 poolFee, uint256 adminFee, uint256 midFee, uint256 outFee,
        uint256, uint256, uint256, uint256, uint256, uint256
    );

    function get_coins(address _pool) external view returns (address[8] memory);

    function get_n_coins(address _pool) external view returns (uint256);

    function get_coin_indices(address _pool, address _from, address _to) external view returns (int128, int128, bool);

    function get_balances(address _pool) external view returns (uint256[8] memory);

}