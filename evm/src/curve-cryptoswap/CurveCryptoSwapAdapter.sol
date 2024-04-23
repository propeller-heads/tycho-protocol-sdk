// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {
    IERC20,
    SafeERC20
} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

/// @dev custom RESERVE_LIMIT_FACTOR for limits for this adapter(underestimate)
uint256 constant RESERVE_LIMIT_FACTOR = 10;

/// @title Curve Finance CryptoSwap Adapter
contract CurveCryptoSwapAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    ICurveRegistry immutable registry;

    constructor(address _registry) {
        registry = ICurveRegistry(_registry);
    }

    /**
     * @dev It is not possible to reproduce the swap in a view mode (like
     * Bancor, Uniswap v2, etc..) as the swap produce a change of storage in
     * the Curve protocol, that impacts the price post trade. Due to the
     * architecture of Curve, it's not possible to calculate the storage
     * modifications of Curve inside the adapter.
     */
    function price(bytes32, address, address, uint256[] memory)
        external
        pure
        override
        returns (Fraction[] memory)
    {
        revert NotImplemented("CurveStableSwapAdapter.price");
    }

    /// @inheritdoc ISwapAdapter
    function swap(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external override returns (Trade memory trade) {
        if (specifiedAmount == 0) {
            return trade;
        }
        address poolAddress = address(bytes20(poolId));
        ICurveCryptoPool pool = ICurveCryptoPool(poolAddress);
        (int128 sellTokenIndex, int128 buyTokenIndex,) =
            registry.get_coin_indices(poolAddress, sellToken, buyToken);
        uint256 gasBefore = gasleft();
        uint256 sellTokenIndexFixed = uint256(uint128(sellTokenIndex));
        uint256 buyTokenIndexFixed = uint256(uint128(buyTokenIndex));

        if (side == OrderSide.Sell) {
            trade.calculatedAmount = sell(
                pool,
                IERC20(sellToken),
                IERC20(buyToken),
                sellTokenIndexFixed,
                buyTokenIndexFixed,
                specifiedAmount
            );
        } else {
            revert Unavailable(
                "OrderSide.Buy is not available for this adapter"
            );
        }
        trade.gasUsed = gasBefore - gasleft();
        trade.price = getPriceAt(pool, sellTokenIndexFixed, buyTokenIndexFixed);
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32 poolId, address sellToken, address buyToken)
        external
        view
        override
        returns (uint256[] memory limits)
    {
        address poolAddress = address(bytes20(poolId));
        (int128 sellTokenIndex, int128 buyTokenIndex,) =
            registry.get_coin_indices(poolAddress, sellToken, buyToken);
        uint256[8] memory poolBalances = registry.get_balances(poolAddress);

        limits = new uint256[](2);
        uint256 sellTokenIndexFixed = uint256(uint128(sellTokenIndex));
        uint256 buyTokenIndexFixed = uint256(uint128(buyTokenIndex));
        limits[0] = poolBalances[sellTokenIndexFixed] / RESERVE_LIMIT_FACTOR;
        limits[1] = poolBalances[buyTokenIndexFixed] / RESERVE_LIMIT_FACTOR;
    }

    /// @inheritdoc ISwapAdapter
    function getCapabilities(bytes32, address, address)
        external
        pure
        override
        returns (Capability[] memory capabilities)
    {
        capabilities = new Capability[](1);
        capabilities[0] = Capability.SellOrder;
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(bytes32 poolId)
        external
        view
        override
        returns (address[] memory tokens)
    {
        address[8] memory coins = registry.get_coins(address(bytes20(poolId)));
        uint256 coinsLength;
        for (uint256 i = 0; i < coins.length; i++) {
            if (coins[i] == address(0)) {
                break;
            }
            coinsLength++;
        }
        tokens = new address[](coinsLength);
        for (uint256 j = 0; j < coinsLength; j++) {
            tokens[j] = coins[j];
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
    /// @param sellTokenIndex The index of token in the pool being sold.
    /// @param buyTokenIndex The index of token being sold among the pool tokens
    /// @return The price as a fraction corresponding to the provided amount.
    function getPriceAt(
        ICurveCryptoPool pool,
        uint256 sellTokenIndex,
        uint256 buyTokenIndex
    ) internal view returns (Fraction memory) {
        uint256 amountIn = pool.balances(sellTokenIndex) / 100000;
        return Fraction(
            pool.get_dy(sellTokenIndex, buyTokenIndex, amountIn), amountIn
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
        ICurveCryptoPool pool,
        IERC20 sellToken,
        IERC20 buyToken,
        uint256 sellTokenIndex,
        uint256 buyTokenIndex,
        uint256 amount
    ) internal returns (uint256 calculatedAmount) {
        sellToken.safeTransferFrom(msg.sender, address(this), amount);
        sellToken.safeIncreaseAllowance(address(pool), amount);

        calculatedAmount =
            pool.exchange(sellTokenIndex, buyTokenIndex, amount, 0);
        buyToken.safeTransfer(address(msg.sender), calculatedAmount);
    }
}

/// @dev Wrapped ported version of Curve Plain Pool to Solidity
/// For params informations see:
/// https://docs.curve.fi/stableswap-exchange/stableswap/pools/plain_pools/
interface ICurveCryptoPool {
    function exchange(uint256 i, uint256 j, uint256 dx, uint256 min_dy)
        external
        returns (uint256);

    function get_dy(uint256 i, uint256 j, uint256 dx)
        external
        view
        returns (uint256);

    function balances(uint256 arg0) external view returns (uint256);

    function fee() external view returns (uint256);
}

/// @dev Wrapped ported version of CurveRegistry to Solidity
/// For params informations see: https://docs.curve.fi/registry/MetaRegistryAPI/
interface ICurveRegistry {
    function find_pool_for_coins(address _from, address _to, uint256 i)
        external
        view
        returns (address);

    function pool_count() external view returns (uint256);

    function pool_list(uint256 _index) external view returns (address);

    function get_fees(address _pool)
        external
        view
        returns (
            uint256 poolFee,
            uint256 adminFee,
            uint256 midFee,
            uint256 outFee,
            uint256,
            uint256,
            uint256,
            uint256,
            uint256,
            uint256
        );

    function get_coins(address _pool)
        external
        view
        returns (address[8] memory);

    function get_n_coins(address _pool) external view returns (uint256);

    function get_coin_indices(address _pool, address _from, address _to)
        external
        view
        returns (int128, int128, bool);

    function get_balances(address _pool)
        external
        view
        returns (uint256[8] memory);
}
