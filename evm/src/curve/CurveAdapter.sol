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

/// @title Curve Finance Adapter
/// @dev This contract supports both CryptoSwap and StableSwap Curve pools
contract CurveAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    ICurveRegistry immutable registry;
    address constant WETH_ADDRESS = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;

    constructor(address _registry) {
        registry = ICurveRegistry(_registry);
    }

    /// @dev enable receive as this contract supports ETH
    receive() external payable {}

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
        revert NotImplemented("CurveAdapter.price");
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
        bool isMetaPool = registry.is_meta(poolAddress);

        (int128 sellTokenIndex, int128 buyTokenIndex,) =
            registry.get_coin_indices(poolAddress, sellToken, buyToken);

        uint256 gasBefore = gasleft();

        if (side == OrderSide.Sell) {
            trade.calculatedAmount = sell(
                poolAddress,
                IERC20(sellToken),
                IERC20(buyToken),
                sellTokenIndex,
                buyTokenIndex,
                specifiedAmount,
                isMetaPool
            );
        } else {
            revert Unavailable(
                "OrderSide.Buy is not available for this adapter"
            );
        }

        trade.gasUsed = gasBefore - gasleft();
        trade.price = getPriceAt(poolAddress, sellTokenIndex, buyTokenIndex, isMetaPool);
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
        address[] memory tokensTmp = new address[](coinsLength);
        bool containsETH = false;
        for (uint256 j = 0; j < coinsLength; j++) {
            if(coins[j] == WETH_ADDRESS) {
                containsETH = true;
            }
            tokens[j] = coins[j];
        }
        
        if(containsETH) {
            tokens = new address[](coinsLength+1);
            tokens[coinsLength] = address(0);
        }
        else {
            tokens = tokensTmp;
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
    /// @param isMetaPool Determine if the pool is a MetaPool
    /// @return The price as a fraction corresponding to the provided amount.
    function getPriceAt(
        address pool,
        int128 sellTokenIndex,
        int128 buyTokenIndex,
        bool isMetaPool
    ) internal view returns (Fraction memory) {
        uint256 amountIn;
        uint256 sellTokenIndexFixed = uint256(uint128(sellTokenIndex));
        if (isStablePool(pool)) {
            amountIn =
                ICurveStablePool(pool).balances(sellTokenIndexFixed) / 100000;
            return Fraction(
                ICurveStablePool(pool).get_dy(
                    sellTokenIndex, buyTokenIndex, amountIn
                ),
                amountIn
            );
        } else {
            amountIn =
                ICurveCryptoPool(pool).balances(sellTokenIndexFixed) / 100000;
            return Fraction(
                ICurveCryptoPool(pool).get_dy(
                    uint256(uint128(sellTokenIndex)),
                    uint256(uint128(buyTokenIndex)),
                    amountIn
                ),
                amountIn
            );
        }
    }

    /// @notice Executes a sell order on a given pool.
    /// @param pool The pool to trade on.
    /// @param sellToken IERC20 instance of the token being sold.
    /// @param buyToken IERC20 instance of the token being bought.
    /// @param sellTokenIndex The index of token in the pool being sold.
    /// @param buyTokenIndex The index of token being sold among the pool tokens
    /// @param amount The amount to be traded.
    /// @param isMetaPool Determine if the pool is a MetaPool
    /// @return calculatedAmount The amount of tokens received.
    function sell(
        address pool,
        IERC20 sellToken,
        IERC20 buyToken,
        int128 sellTokenIndex,
        int128 buyTokenIndex,
        uint256 amount,
        bool isMetaPool
    ) internal returns (uint256 calculatedAmount) {
        uint256 buyTokenBalBefore = buyToken.balanceOf(address(this));
        sellToken.safeTransferFrom(msg.sender, address(this), amount);
        sellToken.safeIncreaseAllowance(address(pool), amount);

        if (isStablePool(pool)) {
            ICurveStablePool(pool).exchange(
                sellTokenIndex, buyTokenIndex, amount, 0
            );
        } else {
            ICurveCryptoPool(pool).exchange(
                uint256(uint128(sellTokenIndex)),
                uint256(uint128(buyTokenIndex)),
                amount,
                0
            );
        }

        calculatedAmount = buyToken.balanceOf(address(this)) - buyTokenBalBefore;
        buyToken.safeTransfer(address(msg.sender), calculatedAmount);
    }

    /// @dev Check whether a pool is a StableSwap pool or CryptoSwap pool
    /// @param poolAddress address of the pool
    function isStablePool(address poolAddress) internal view returns (bool) {
        try ICurveCryptoPool(poolAddress).get_dy(0, 1, 10 ** 6) returns (
            uint256
        ) {
            return false;
        } catch {
            return true;
        }
    }
}

/// @dev Wrapped ported version of Curve Plain Pool to Solidity
/// For params informations see:
/// https://docs.curve.fi/stableswap-exchange/stableswap/pools/plain_pools/
interface ICurveCryptoPool {
    function exchange(uint256 i, uint256 j, uint256 dx, uint256 min_dy)
        external;

    function get_dy(uint256 i, uint256 j, uint256 dx)
        external
        view
        returns (uint256);

    function balances(uint256 arg0) external view returns (uint256);

    function fee() external view returns (uint256);
}

/// @dev Wrapped ported version of Curve Plain Pool to Solidity
/// For params informations see:
/// https://docs.curve.fi/stableswap-exchange/stableswap/pools/plain_pools/
interface ICurveStablePool {
    function exchange(int128 i, int128 j, uint256 dx, uint256 min_dy)
        external;

    function get_dy(int128 i, int128 j, uint256 dx)
        external
        view
        returns (uint256);

    function balances(uint256 arg0) external view returns (uint256);

    function fee() external view returns (uint256);
}

interface ICurveStableSwapMetaPool {
    function get_dy_underlying(int128 i, int128 j, uint256 dx)
        external
        view
        returns (uint256); 
}

/// @dev Wrapped ported version of CurveRegistry to Solidity
/// For params informations see: https://docs.curve.fi/registry/MetaRegistryAPI/
interface ICurveRegistry {

    function is_meta(address  _pool) external view returns (bool);

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
