// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

import {IAlgebraPool, IAlgebraFactory} from "./IAlgebra.sol";

/// @title SupernovaV3Adapter
/// @notice Adapter for swapping tokens on Supernova V3 (Algebra CL) pools.
contract SupernovaV3Adapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    IAlgebraFactory public immutable factory;

    constructor(address _factory) {
        factory = IAlgebraFactory(_factory);
    }

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        uint256[] memory specifiedAmounts
    ) external override returns (Fraction[] memory prices) {
        prices = new Fraction[](specifiedAmounts.length);
        IAlgebraPool pool = IAlgebraPool(address(bytes20(poolId)));
        
        for (uint256 i = 0; i < specifiedAmounts.length; i++) {
            prices[i] = priceAt(pool, sellToken, buyToken, specifiedAmounts[i]);
        }
    }

    /// @notice Calculate the marginal price at a specified amount.
    /// @dev This simulates a zero-amount swap to get the marginal price.
    function priceAt(
        IAlgebraPool pool,
        address sellToken,
        address buyToken,
        uint256 sellAmount
    ) public returns (Fraction memory) {
        bool zeroToOne = sellToken == pool.token0();
        
        if (sellAmount == 0) {
            (uint160 sqrtPriceX96,,,,,) = pool.globalState();
            uint256 priceX192 = uint256(sqrtPriceX96) * sqrtPriceX96;
            if (zeroToOne) {
                // price = amount1 / amount0 = sqrtPrice^2
                return Fraction(priceX192, 1 << 192);
            } else {
                // price = amount0 / amount1 = 1 / sqrtPrice^2
                return Fraction(1 << 192, priceX192);
            }
        }

        // For non-zero amounts, simulate the swap to find the marginal price after the trade.
        uint160 limitSqrtPrice = zeroToOne ? 4295128739 + 1 : 1461446703485210103287273052203988822378723970341 - 1;
        
        try pool.swap(
            address(this),
            zeroToOne,
            int256(sellAmount),
            limitSqrtPrice,
            ""
        ) returns (int256, int256) {
            (uint160 sqrtPriceX96After,,,,,) = pool.globalState();
            uint256 priceX192 = uint256(sqrtPriceX96After) * sqrtPriceX96After;
            if (zeroToOne) {
                return Fraction(priceX192, 1 << 192);
            } else {
                return Fraction(1 << 192, priceX192);
            }
        } catch {
            return Fraction(0, 0);
        }
    }

    /// @inheritdoc ISwapAdapter
    function swap(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external override returns (Trade memory trade) {
        if (specifiedAmount == 0) return trade;

        IAlgebraPool pool = IAlgebraPool(address(bytes20(poolId)));
        bool zeroToOne = sellToken == pool.token0();
        uint160 limitSqrtPrice = zeroToOne ? 4295128739 + 1 : 1461446703485210103287273052203988822378723970341 - 1;
        
        uint256 gasBefore = gasleft();
        
        (int256 amount0, int256 amount1) = pool.swap(
            address(this),
            zeroToOne,
            side == OrderSide.Sell ? int256(specifiedAmount) : -int256(specifiedAmount),
            limitSqrtPrice,
            abi.encode(msg.sender)
        );

        if (side == OrderSide.Sell) {
            trade.calculatedAmount = uint256(zeroToOne ? -amount1 : -amount0);
        } else {
            trade.calculatedAmount = uint256(zeroToOne ? amount0 : amount1);
        }

        trade.gasUsed = gasBefore - gasleft();
        trade.price = priceAt(pool, sellToken, buyToken, side == OrderSide.Sell ? specifiedAmount : trade.calculatedAmount);
        
        return trade;
    }

    /// @notice Callback for Algebra swap.
    function algebraSwapCallback(
        int256 amount0Delta,
        int256 amount1Delta,
        bytes calldata data
    ) external {
        address payer = abi.decode(data, (address));
        if (amount0Delta > 0) {
            IERC20(IAlgebraPool(msg.sender).token0()).safeTransferFrom(payer, msg.sender, uint256(amount0Delta));
        } else if (amount1Delta > 0) {
            IERC20(IAlgebraPool(msg.sender).token1()).safeTransferFrom(payer, msg.sender, uint256(amount1Delta));
        }
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32 poolId, address sellToken, address buyToken)
        external
        view
        override
        returns (uint256[] memory limits)
    {
        IAlgebraPool pool = IAlgebraPool(address(bytes20(poolId)));
        (uint128 reserve0, uint128 reserve1) = pool.getReserves();
        
        limits = new uint256[](2);
        if (sellToken == pool.token0()) {
            limits[0] = reserve0;
            limits[1] = reserve1;
        } else {
            limits[0] = reserve1;
            limits[1] = reserve0;
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
    function getTokens(bytes32 poolId)
        external
        view
        override
        returns (address[] memory tokens)
    {
        tokens = new address[](2);
        IAlgebraPool pool = IAlgebraPool(address(bytes20(poolId)));
        tokens[0] = pool.token0();
        tokens[1] = pool.token1();
    }

    /// @inheritdoc ISwapAdapter
    function getPoolIds(uint256 offset, uint256 limit)
        external
        view
        override
        returns (bytes32[] memory ids)
    {
        return new bytes32[](0);
    }
}
