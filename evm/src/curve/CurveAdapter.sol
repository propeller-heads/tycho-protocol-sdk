// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {
    IERC20,
    SafeERC20
} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import "src/libraries/FractionMath.sol";

/// @dev custom RESERVE_LIMIT_FACTOR for limits for this adapter(underestimate)
uint256 constant RESERVE_LIMIT_FACTOR = 10;

/// @title Curve Finance Adapter
/// @dev This contract supports both CryptoSwap and StableSwap Curve pools
contract CurveAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;
    using FractionMath for Fraction;

    struct SellParamsCache {
        address poolAddress; // address of the pool to swap in
        address sellToken; // address of the token to sell
        address buyToken; // address of the token to buy
        int128 sellTokenIndex; // index of the token being sold
        int128 buyTokenIndex; // index of the token being bought
        uint256 specifiedAmount; // amount to trade
    }

    struct PoolCoins {
        address[8] addresses;
        uint256 coinsLength;
    }

    uint256 constant PRECISION = 10 ** 5;

    address constant WETH_ADDRESS = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
    address constant ETH_ADDRESS = 0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE;

    constructor() {}

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

        SellParamsCache memory sellParams;
        {
            sellParams.poolAddress = address(bytes20(poolId));
            sellParams.sellToken = sellToken;
            sellParams.buyToken = buyToken;
            sellParams.specifiedAmount = specifiedAmount;

            bool isEthPool; // pool is native ETH pool
            PoolCoins memory coins = getCoins(sellParams.poolAddress);

            /// @dev Support for Native ETH pools, ETH pools cannot be Meta
            /// therefore we can directly access coins without using underlying
            if (sellToken == address(0)) {
                for (uint256 i = 0; i < coins.coinsLength; i++) {
                    if (
                        coins.addresses[i] == ETH_ADDRESS
                            || coins.addresses[i] == WETH_ADDRESS
                    ) {
                        sellParams.sellToken = ETH_ADDRESS;
                        if (coins.addresses[i] == ETH_ADDRESS) {
                            isEthPool = true;
                        }
                        break;
                    }
                }
            } else if (buyToken == address(0)) {
                for (uint256 i = 0; i < coins.coinsLength; i++) {
                    if (
                        coins.addresses[i] == ETH_ADDRESS
                            || coins.addresses[i] == WETH_ADDRESS
                    ) {
                        sellParams.buyToken = ETH_ADDRESS;
                        if (coins.addresses[i] == ETH_ADDRESS) {
                            isEthPool = true;
                        }
                        break;
                    }
                }
            }

            (sellParams.sellTokenIndex, sellParams.buyTokenIndex) =
            getCoinsIndices(
                sellParams.poolAddress,
                sellParams.sellToken,
                sellParams.buyToken,
                coins,
                isEthPool
            );
        }

        uint256 gasBefore = gasleft();

        if (side == OrderSide.Sell) {
            trade.calculatedAmount = sell(sellParams);
        } else {
            revert Unavailable(
                "OrderSide.Buy is not available for this adapter"
            );
        }

        trade.gasUsed = gasBefore - gasleft();
        trade.price = getPriceAt(sellParams);
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32 poolId, address sellToken, address buyToken)
        external
        view
        override
        returns (uint256[] memory limits)
    {
        address poolAddress = address(bytes20(poolId));
        ICurveStableSwapPool pool = ICurveStableSwapPool(poolAddress);
        address sellToken_ = sellToken;
        address buyToken_ = buyToken;
        bool isEthPool;
        PoolCoins memory coins = getCoins(poolAddress);

        /// @dev Support for Native ETH pools, ETH pools cannot be Meta
        /// therefore we can directly access coins without using underlying
        if (sellToken == address(0)) {
            for (uint256 i = 0; i < coins.coinsLength; i++) {
                if (
                    coins.addresses[i] == ETH_ADDRESS
                        || coins.addresses[i] == WETH_ADDRESS
                ) {
                    sellToken_ = ETH_ADDRESS;
                    if (coins.addresses[i] == ETH_ADDRESS) {
                        isEthPool = true;
                    }
                    break;
                }
            }
        } else if (buyToken == address(0)) {
            for (uint256 i = 0; i < coins.coinsLength; i++) {
                if (
                    coins.addresses[i] == ETH_ADDRESS
                        || coins.addresses[i] == WETH_ADDRESS
                ) {
                    buyToken_ = ETH_ADDRESS;
                    if (coins.addresses[i] == ETH_ADDRESS) {
                        isEthPool = true;
                    }
                    break;
                }
            }
        }

        (int128 sellTokenIndex, int128 buyTokenIndex) = getCoinsIndices(
            poolAddress, sellToken_, buyToken_, coins, isEthPool
        );

        limits = new uint256[](2);
        uint256 sellTokenIndexUint = uint256(uint128(sellTokenIndex));
        uint256 buyTokenIndexUint = uint256(uint128(buyTokenIndex));
        limits[0] = pool.balances(sellTokenIndexUint) / RESERVE_LIMIT_FACTOR;
        limits[1] = pool.balances(buyTokenIndexUint) / RESERVE_LIMIT_FACTOR;
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
        PoolCoins memory coins = getCoins(address(bytes20(poolId)));
        address[] memory tokensTmp = new address[](coins.coinsLength);
        bool containsETH;
        for (uint256 j = 0; j < coins.coinsLength; j++) {
            if (coins.addresses[j] == WETH_ADDRESS) {
                containsETH = true;
            }
            if (coins.addresses[j] == ETH_ADDRESS) {
                continue;
            }
            tokensTmp[j] = coins.addresses[j];
        }

        if (containsETH) {
            tokens = new address[](coins.coinsLength + 1);
            for (uint256 k = 0; k < coins.coinsLength; k++) {
                tokens[k] = tokensTmp[k];
            }
            tokens[coins.coinsLength] = address(0);
        } else {
            tokens = tokensTmp;
        }
    }

    function getPoolIds(uint256, uint256)
        external
        view
        override
        returns (bytes32[] memory ids)
    {
        revert NotImplemented("CurveAdapter.getPoolIds");
    }

    /// @notice Calculates pool prices for specified amounts
    /// @param sellParams Params for the price(see: struct SellParamsCache).
    /// @return (Fraction) price as a fraction corresponding to the provided
    /// amount.
    function getPriceAt(SellParamsCache memory sellParams)
        internal
        view
        returns (Fraction memory)
    {
        uint256 amountIn;
        uint256 sellTokenIndexUint = uint256(uint128(sellParams.sellTokenIndex));
        uint256 buyTokenIndexUint = uint256(uint128(sellParams.buyTokenIndex));
        if (isInt128Pool(sellParams.poolAddress)) {
            amountIn = ICurveStableSwapPool(sellParams.poolAddress).balances(
                sellTokenIndexUint
            ) / PRECISION;

            return Fraction(
                ICurveStableSwapPool(sellParams.poolAddress).get_dy(
                    sellParams.sellTokenIndex,
                    sellParams.buyTokenIndex,
                    amountIn
                ),
                amountIn
            );
        } else {
            amountIn = ICurveCryptoSwapPool(sellParams.poolAddress).balances(
                sellTokenIndexUint
            ) / PRECISION;

            return Fraction(
                ICurveCryptoSwapPool(sellParams.poolAddress).get_dy(
                    sellTokenIndexUint, buyTokenIndexUint, amountIn
                ),
                amountIn
            );
        }
    }

    /// @notice Executes a sell order on a given pool.
    /// @dev Only metapools available (with LP token as counter pair) are
    /// Stable,
    /// but after some research we've found that Curve deployed some pools that
    /// are Crypto and use the int128 interface, therefore we optimistically
    /// support them too.
    /// @param sellParams Params for the trade(see: struct SellParamsCache).
    /// @return calculatedAmount The amount of tokens received.
    function sell(SellParamsCache memory sellParams)
        internal
        returns (uint256 calculatedAmount)
    {
        IERC20 buyToken = IERC20(sellParams.buyToken);
        IERC20 sellToken = IERC20(sellParams.sellToken);
        uint256 buyTokenBalBefore = (sellParams.buyToken == ETH_ADDRESS)
            ? address(this).balance
            : buyToken.balanceOf(address(this));

        if (isInt128Pool(sellParams.poolAddress)) {
            if (sellParams.sellToken == ETH_ADDRESS) {
                // ETH Pool
                ICurveStableSwapPoolEth(sellParams.poolAddress).exchange{
                    value: sellParams.specifiedAmount
                }(
                    sellParams.sellTokenIndex,
                    sellParams.buyTokenIndex,
                    sellParams.specifiedAmount,
                    0,
                    address(this)
                );
            } else {
                sellToken.safeTransferFrom(
                    msg.sender, address(this), sellParams.specifiedAmount
                );
                sellToken.safeIncreaseAllowance(
                    sellParams.poolAddress, sellParams.specifiedAmount
                );
                ICurveStableSwapPool(sellParams.poolAddress).exchange(
                    sellParams.sellTokenIndex,
                    sellParams.buyTokenIndex,
                    sellParams.specifiedAmount,
                    0
                );
            }
        } else {
            uint256 sellTokenIndexUint =
                uint256(uint128(sellParams.sellTokenIndex));
            uint256 buyTokenIndexUint =
                uint256(uint128(sellParams.buyTokenIndex));
            if (sellParams.sellToken == ETH_ADDRESS) {
                ICurveCryptoSwapPoolEth(sellParams.poolAddress).exchange{
                    value: sellParams.specifiedAmount
                }(
                    sellTokenIndexUint,
                    buyTokenIndexUint,
                    sellParams.specifiedAmount,
                    0,
                    true,
                    address(this)
                );
            } else {
                sellToken.safeTransferFrom(
                    msg.sender, address(this), sellParams.specifiedAmount
                );
                sellToken.safeIncreaseAllowance(
                    sellParams.poolAddress, sellParams.specifiedAmount
                );
                ICurveCryptoSwapPool(sellParams.poolAddress).exchange(
                    sellTokenIndexUint,
                    buyTokenIndexUint,
                    sellParams.specifiedAmount,
                    0
                );
            }
        }

        if (sellParams.buyToken == ETH_ADDRESS) {
            calculatedAmount = address(this).balance - buyTokenBalBefore;
            (bool sent,) = address(msg.sender).call{value: calculatedAmount}("");
            require(sent, "Eth transfer failed");
        } else {
            calculatedAmount =
                buyToken.balanceOf(address(this)) - buyTokenBalBefore;
            buyToken.safeTransfer(address(msg.sender), calculatedAmount);
        }
    }

    /// @dev Check whether a pool is a StableSwap pool or CryptoSwap pool
    /// @param poolAddress address of the pool
    function isInt128Pool(address poolAddress) internal view returns (bool) {
        ICurveStableSwapPool pool = ICurveStableSwapPool(poolAddress);
        uint256 reserve0 = pool.balances(0);
        try ICurveCryptoSwapPool(poolAddress).get_dy(
            0, 1, reserve0 / PRECISION
        ) returns (uint256) {
            return false;
        } catch {
            return true;
        }
    }

    /// @notice Get coins inside a pool
    /// @param poolAddress The address of the pool
    function getCoins(address poolAddress)
        internal
        view
        returns (PoolCoins memory output)
    {
        ICurveStableSwapPool pool = ICurveStableSwapPool(poolAddress);
        uint256 len;

        /// @dev as of registry, max addresses that can be included in a pool is
        /// always 8, therefore we limit the loop to it.
        for (len; len < 8; len++) {
            try pool.coins(len) returns (address coin) {
                output.addresses[len] = coin;
            } catch {
                break;
            }
        }

        output.coinsLength = len + 1;
    }

    /// @notice Get indices of coins to swap
    /// @dev If the pool is meta the registry.get_coin_indices includes the
    /// underlying addresses (appended to the array from index 1 to length-1)
    /// @param poolAddress The address of the pool
    /// @param sellToken The token being sold
    /// @param buyToken The token being bought
    /// @param coins output of getCoins()
    /// @param isEthPool determine if pool has native ETH inside
    function getCoinsIndices(
        address poolAddress,
        address sellToken,
        address buyToken,
        PoolCoins memory coins,
        bool isEthPool
    ) internal view returns (int128 sellTokenIndex, int128 buyTokenIndex) {
        address sellToken_ = sellToken;
        address buyToken_ = buyToken;
        if (sellToken == ETH_ADDRESS && !isEthPool) {
            sellToken_ = WETH_ADDRESS;
        }
        if (buyToken == ETH_ADDRESS && !isEthPool) {
            buyToken_ = WETH_ADDRESS;
        }
        for (uint256 i; i < coins.coinsLength; i++) {
            if (coins.addresses[i] == sellToken_) {
                sellTokenIndex = int128(uint128(i));
            } else if (coins.addresses[i] == buyToken_) {
                buyTokenIndex = int128(uint128(i));
            }
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
        payable;

    function balances(uint256 arg0) external view returns (uint256);

    function fee() external view returns (uint256);
}

interface ICurveCryptoSwapPoolEth is ICurveCryptoSwapPool {
    function exchange(
        uint256 i,
        uint256 j,
        uint256 dx,
        uint256 min_dy,
        bool use_eth,
        address receiver
    ) external payable;
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

    function coins(uint256 i) external view returns (address);
}

interface ICurveStableSwapPoolEth {
    function exchange(
        int128 i,
        int128 j,
        uint256 dx,
        uint256 min_dy,
        address receiver
    ) external payable;
}

interface ICurveStableSwapMetaPool is ICurveStableSwapPool {
    function get_dy_underlying(int128 i, int128 j, uint256 dx)
        external
        view
        returns (uint256);

    function exchange_underlying(int128 i, int128 j, uint256 dx, uint256 min_dy)
        external
        returns (uint256);
}

/// @dev Wrapped ported version of CurveRegistry to Solidity
/// For params informations see: https://docs.curve.fi/registry/MetaRegistryAPI/
interface ICurveRegistry {
    function is_meta(address _pool) external view returns (bool);

    function get_pool_asset_type(address _pool)
        external
        view
        returns (uint256);

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

    function get_underlying_coins(address _pool)
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

    function get_underlying_balances(address _pool)
        external
        view
        returns (uint256[8] memory);
}
