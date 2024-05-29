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

    uint256 constant PRECISION = (10 ** 6);

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

        (int128 sellTokenIndex, int128 buyTokenIndex) =
            getCoinsIndices(poolAddress, sellToken, buyToken);

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
        trade.price = getPriceAt(
            poolAddress,
            sellToken,
            buyToken,
            sellTokenIndex,
            buyTokenIndex,
            isMetaPool
        );
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
        uint256 sellTokenIndexUint = uint256(uint128(sellTokenIndex));
        uint256 buyTokenIndexUint = uint256(uint128(buyTokenIndex));
        limits[0] = poolBalances[sellTokenIndexUint] / RESERVE_LIMIT_FACTOR;
        limits[1] = poolBalances[buyTokenIndexUint] / RESERVE_LIMIT_FACTOR;
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
            if (coins[j] == WETH_ADDRESS) {
                containsETH = true;
            }
            tokensTmp[j] = coins[j];
        }

        if (containsETH) {
            tokens = new address[](coinsLength + 1);
            for (uint256 k = 0; k < coinsLength; k++) {
                tokens[k] = tokensTmp[k];
            }
            tokens[coinsLength] = address(0);
        } else {
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
    /// @param poolAddress The pool to calculate token prices in.
    /// @param sellTokenIndex The index of token in the pool being sold.
    /// @param buyTokenIndex The index of token being sold among the pool tokens
    /// @param isMetaPool Determine if the pool is a MetaPool
    /// @return The price as a fraction corresponding to the provided amount.
    function getPriceAt(
        address poolAddress,
        address sellToken,
        address buyToken,
        int128 sellTokenIndex,
        int128 buyTokenIndex,
        bool isMetaPool
    ) internal view returns (Fraction memory) {
        uint256 amountIn;
        uint256 sellTokenIndexUint = uint256(uint128(sellTokenIndex));
        uint256 buyTokenIndexUint = uint256(uint128(buyTokenIndex));

        if (isStablePool(poolAddress)) {
            if (isMetaPool) {
                amountIn = ICurveStableSwapMetaPool(poolAddress).balances(
                    sellTokenIndexUint
                ) / PRECISION;

                return Fraction(
                    ICurveStableSwapMetaPool(poolAddress).get_dy_underlying(
                        sellTokenIndex, buyTokenIndex, amountIn
                    ),
                    amountIn
                );
            } else {
                amountIn = ICurveStableSwapPool(poolAddress).balances(
                    sellTokenIndexUint
                ) / PRECISION;

                return Fraction(
                    ICurveStableSwapPool(poolAddress).get_dy(
                        sellTokenIndex, buyTokenIndex, amountIn
                    ),
                    amountIn
                );
            }
        } else {
            if (
                address(sellToken) == WETH_ADDRESS
                    || address(buyToken) == WETH_ADDRESS
            ) {
                amountIn = ICurveCryptoSwapMetaPool(poolAddress).balances(
                    sellTokenIndexUint
                ) / PRECISION;

                return Fraction(
                    ICurveCryptoSwapMetaPool(poolAddress).get_dy_underlying(
                        sellTokenIndex, buyTokenIndex, amountIn
                    ),
                    amountIn
                );
            } else {
                amountIn = ICurveCryptoSwapPool(poolAddress).balances(
                    sellTokenIndexUint
                ) / PRECISION;

                return Fraction(
                    ICurveCryptoSwapPool(poolAddress).get_dy(
                        sellTokenIndexUint, buyTokenIndexUint, amountIn
                    ),
                    amountIn
                );
            }
        }
    }

    /// @notice Executes a sell order on a given pool.
    /// @param poolAddress The address of the pool to trade on.
    /// @param sellToken IERC20 instance of the token being sold.
    /// @param buyToken IERC20 instance of the token being bought.
    /// @param sellTokenIndex The index of token in the pool being sold.
    /// @param buyTokenIndex The index of token being sold among the pool tokens
    /// @param amount The amount to be traded.
    /// @return calculatedAmount The amount of tokens received.
    function sell(
        address poolAddress,
        IERC20 sellToken,
        IERC20 buyToken,
        int128 sellTokenIndex,
        int128 buyTokenIndex,
        uint256 amount,
        bool isMetaPool
    ) internal returns (uint256 calculatedAmount) {
        uint256 buyTokenBalBefore = buyToken.balanceOf(address(this));

        uint256 sellTokenIndexUint = uint256(uint128(sellTokenIndex));
        uint256 buyTokenIndexUint = uint256(uint128(buyTokenIndex));

        sellToken.safeTransferFrom(msg.sender, address(this), amount);
        // Why is it casting again a poolAddress into address?
        sellToken.safeIncreaseAllowance(address(poolAddress), amount);

        if (isStablePool(poolAddress)) {
            if (isMetaPool) {
                ICurveStableSwapMetaPool(poolAddress).exchange_underlying(
                    sellTokenIndex, buyTokenIndex, amount, 0
                );
            } else {
                ICurveStableSwapPool(poolAddress).exchange(
                    sellTokenIndex, buyTokenIndex, amount, 0
                );
            }
        } else {
            if (
                address(sellToken) == WETH_ADDRESS
                    || address(buyToken) == WETH_ADDRESS
            ) {
                ICurveCryptoSwapMetaPool(poolAddress).exchange_underlying(
                    sellTokenIndexUint, buyTokenIndexUint, amount, 0
                );
            } else {
                ICurveCryptoSwapPool(poolAddress).exchange(
                    sellTokenIndexUint, buyTokenIndexUint, amount, 0
                );
            }
        }

        calculatedAmount = buyToken.balanceOf(address(this)) - buyTokenBalBefore;
        buyToken.safeTransfer(address(msg.sender), calculatedAmount);
    }

    /// @dev Check whether a pool is a StableSwap pool or CryptoSwap pool
    /// @param poolAddress address of the pool
    function isStablePool(address poolAddress) internal view returns (bool) {
        try ICurveCryptoSwapPool(poolAddress).get_dy(0, 1, 10 ** 6) returns (
            uint256
        ) {
            return false;
        } catch {
            return true;
        }
    }

    /// @param buyToken The token being bought
    /// @param sellTokenIndex The index of the sellToken in the specified pool
    /// @param buyTokenIndex The index of the buyToken in the specified pool
    function getCoinsIndices(
        address poolAddress,
        address sellToken,
        address buyToken
    ) internal view returns (int128 sellTokenIndex, int128 buyTokenIndex) {
        if (sellToken == address(0)) {
            sellToken == WETH_ADDRESS;
            (sellTokenIndex, buyTokenIndex,) =
                registry.get_coin_indices(poolAddress, sellToken, buyToken);
        } else if (buyToken == address(0)) {
            buyToken == WETH_ADDRESS;
            (sellTokenIndex, buyTokenIndex,) =
                registry.get_coin_indices(poolAddress, sellToken, buyToken);
        } else {
            (sellTokenIndex, buyTokenIndex,) =
                registry.get_coin_indices(poolAddress, sellToken, buyToken);
        }
    }
}

/// @dev Wrapped ported version of Curve Plain Pool to Solidity
/// For params informations see:
/// https://docs.curve.fi/stableswap-exchange/stableswap/pools/plain_pools/
interface ICurveCryptoSwapPool {
    function get_dy(uint256 i, uint256 j, uint256 dx)
        external
        view
        returns (uint256);

    function exchange(uint256 i, uint256 j, uint256 dx, uint256 min_dy)
        external
        returns (uint256);

    function balances(uint256 arg0) external view returns (uint256);

    function fee() external view returns (uint256);
}
/// @dev
/// A title that should describe the contract/interface
/// The name of the author

interface ICurveCryptoSwapMetaPool {
    function get_dy(uint256 i, uint256 j, uint256 dx)
        external
        view
        returns (uint256);

    function get_dy_underlying(int128 i, int128 j, uint256 dx)
        external
        view
        returns (uint256);

    function exchange_underlying(
        uint256 i,
        uint256 j,
        uint256 dx,
        uint256 min_dy
    ) external returns (uint256);

    function balances(uint256 arg0) external view returns (uint256);
}

/// @dev Wrapped ported version of Curve Plain Pool to Solidity
/// For params informations see:
/// https://docs.curve.fi/stableswap-exchange/stableswap/pools/plain_pools/
interface ICurveStableSwapPool {
    function get_dy(int128 i, int128 j, uint256 dx)
        external
        view
        returns (uint256);

    function exchange(int128 i, int128 j, uint256 dx, uint256 min_dy)
        external;

    function balances(uint256 arg0) external view returns (uint256);

    function fee() external view returns (uint256);
}

interface ICurveStableSwapMetaPool {
    function get_dy_underlying(int128 i, int128 j, uint256 dx)
        external
        view
        returns (uint256);

    function exchange_underlying(int128 i, int128 j, uint256 dx, uint256 min_dy)
        external
        returns (uint256);

    function balances(uint256 arg0) external view returns (uint256);
}

/// @dev Wrapped ported version of CurveRegistry to Solidity
/// For params informations see: https://docs.curve.fi/registry/MetaRegistryAPI/
interface ICurveRegistry {
    function is_meta(address _pool) external view returns (bool);

    function find_pool_for_coins(address _from, address _to, uint256 i)
        external
        view
        returns (address);

    function pool_count() external view returns (uint256);

    function pool_list(uint256 arg0) external view returns (address);

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
