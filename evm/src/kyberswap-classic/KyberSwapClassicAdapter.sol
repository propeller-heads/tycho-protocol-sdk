// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {
    IERC20,
    SafeERC20
} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title Kyberswap Classic Adapter
contract KyberSwapClassicAdapter is ISwapAdapter {
    /// @dev Using trade params inside a struct to avoid stack too deep errors
    struct TradeParams {
        uint112 r0;
        uint112 r1;
        uint112 vr0;
        uint112 vr1;
        uint256 feeInPrecision;
    }

    using SafeERC20 for IERC20;

    // Kyberswap handles arbirary amounts, but we limit the amount to 10x just
    // in case
    uint256 constant RESERVE_LIMIT_FACTOR = 10;

    uint256 constant PRECISION = (10 ** 18);

    IFactory immutable factory;
    IRouter immutable router;

    constructor(address _router) {
        router = IRouter(_router);
        factory = IFactory(router.factory());
    }

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        uint256[] memory specifiedAmounts
    ) external view override returns (Fraction[] memory prices) {
        prices = new Fraction[](specifiedAmounts.length);
        IPair pair = IPair(address(bytes20(poolId)));
        uint112 r0;
        uint112 vr0;
        uint112 r1;
        uint112 vr1;
        uint256 feeInPrecision;
        if (sellToken < buyToken) {
            (r0, r1, vr0, vr1, feeInPrecision) = pair.getTradeInfo();
        } else {
            (r1, r0, vr1, vr0, feeInPrecision) = pair.getTradeInfo();
        }

        for (uint256 i = 0; i < specifiedAmounts.length; i++) {
            prices[i] = getPriceAt(
                specifiedAmounts[i], r0, r1, vr0, vr1, feeInPrecision
            );
        }
    }

    function swap(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external returns (Trade memory trade) {
        if (specifiedAmount == 0) {
            return trade;
        }

        IPair pair = IPair(address(bytes20(poolId)));
        TradeParams memory tradeParams;
        bool zero2one = sellToken < buyToken;
        if (zero2one) {
            (
                tradeParams.r0,
                tradeParams.r1,
                tradeParams.vr0,
                tradeParams.vr1,
                tradeParams.feeInPrecision
            ) = pair.getTradeInfo();
        } else {
            (
                tradeParams.r1,
                tradeParams.r0,
                tradeParams.vr1,
                tradeParams.vr0,
                tradeParams.feeInPrecision
            ) = pair.getTradeInfo();
        }
        uint256 gasBefore = gasleft();

        if (side == OrderSide.Sell) {
            trade.calculatedAmount =
                sell(pair, sellToken, buyToken, zero2one, specifiedAmount);
        } else {
            trade.calculatedAmount =
                buy(pair, sellToken, buyToken, zero2one, specifiedAmount);
        }

        trade.gasUsed = gasBefore - gasleft();

        if (zero2one) {
            (,, tradeParams.vr0, tradeParams.vr1,) = pair.getTradeInfo();
        } else {
            (,, tradeParams.vr1, tradeParams.vr0,) = pair.getTradeInfo();
        }

        /// @dev a fractional value (1/100000) of the trading liquidity, to
        /// impact as little as possible on the slippage
        trade.price = Fraction(
            tradeParams.vr1 * (PRECISION - tradeParams.feeInPrecision)
                / PRECISION,
            tradeParams.vr0
        );
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32 poolId, address sellToken, address buyToken)
        external
        view
        override
        returns (uint256[] memory limits)
    {
        IPair pair = IPair(address(bytes20(poolId)));
        limits = new uint256[](2);
        (uint256 r0, uint256 r1) = pair.getReserves();
        if (sellToken < buyToken) {
            limits[0] = r0 / RESERVE_LIMIT_FACTOR;
            limits[1] = r1 / RESERVE_LIMIT_FACTOR;
        } else {
            limits[0] = r1 / RESERVE_LIMIT_FACTOR;
            limits[1] = r0 / RESERVE_LIMIT_FACTOR;
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
        IPair pair = IPair(address(bytes20(poolId)));
        tokens[0] = pair.token0();
        tokens[1] = pair.token1();
    }

    /// @inheritdoc ISwapAdapter
    function getPoolIds(uint256 offset, uint256 limit)
        external
        view
        override
        returns (bytes32[] memory ids)
    {
        uint256 endIdx = offset + limit;
        if (endIdx > factory.allPoolsLength()) {
            endIdx = factory.allPoolsLength();
        }
        ids = new bytes32[](endIdx - offset);
        for (uint256 i = 0; i < ids.length; i++) {
            ids[i] = bytes20(factory.allPools(offset + i));
        }
    }

    /// @notice Executes a sell order on a given pool.
    /// @param pair The pair to trade on.
    /// @param sellToken The token being sold.
    /// @param buyToken The token being bought.
    /// @param amount The amount to be traded.
    /// @return calculatedAmount The amount of tokens received.
    function sell(
        IPair pair,
        address sellToken,
        address buyToken,
        bool zero2one,
        uint256 amount
    ) internal returns (uint256 calculatedAmount) {
        address[] memory poolsPath = new address[](1);
        IERC20[] memory tokensPath = new IERC20[](2);
        IERC20 sellTokenContract = IERC20(sellToken);

        poolsPath[0] = address(pair);
        tokensPath[0] = sellTokenContract;
        tokensPath[1] = IERC20(buyToken);

        uint256[] memory amountsOut =
            router.getAmountsOut(amount, poolsPath, tokensPath);

        sellTokenContract.safeTransferFrom(msg.sender, address(pair), amount);
        if (zero2one) {
            pair.swap(0, amountsOut[1], msg.sender, "");
        } else {
            pair.swap(amountsOut[1], 0, msg.sender, "");
        }
        return amountsOut[1];
    }

    /// @notice Execute a buy order on a given pool.
    /// @param pair The pair to trade on.
    /// @param sellToken The token being sold.
    /// @param buyToken The token being bought.
    /// @param zero2one Whether the sell token is token0 or token1.
    /// @return calculatedAmount The amount of tokens sold.
    function buy(
        IPair pair,
        address sellToken,
        address buyToken,
        bool zero2one,
        uint256 amountOut
    ) internal returns (uint256 calculatedAmount) {
        address[] memory poolsPath = new address[](1);
        IERC20[] memory tokensPath = new IERC20[](2);
        address pairAddress = address(pair);
        IERC20 sellTokenContract = IERC20(sellToken);

        poolsPath[0] = pairAddress;
        tokensPath[0] = sellTokenContract;
        tokensPath[1] = IERC20(buyToken);

        uint256[] memory amountsIn =
            router.getAmountsIn(amountOut, poolsPath, tokensPath);

        if (amountsIn[0] == 0) {
            return 0;
        }

        sellTokenContract.safeTransferFrom(
            msg.sender, pairAddress, amountsIn[0]
        );

        if (zero2one) {
            pair.swap(0, amountOut, msg.sender, "");
        } else {
            pair.swap(amountOut, 0, msg.sender, "");
        }

        return amountsIn[0];
    }

    /// @notice Given an input amount of an asset and pair reserves, returns the
    /// maximum output amount of the other asset
    /// @param amountIn The amount of the token being sold.
    /// @param vReserveIn The virtual reserve of the token being sold.
    /// @param vReserveOut The virtual reserve of the token being bought.
    /// @param feeInPrecision Fee in PRECISION points
    /// @return amountOut The amount of tokens received.
    function getAmountOut(
        uint256 amountIn,
        uint256 vReserveIn,
        uint256 vReserveOut,
        uint256 feeInPrecision
    ) internal pure returns (uint256 amountOut) {
        if (amountIn == 0) {
            return 0;
        }
        if (vReserveIn == 0 || vReserveOut == 0) {
            revert Unavailable("At least one reserve is zero!");
        }
        uint256 amountInWithFee =
            amountIn * (PRECISION - feeInPrecision) / PRECISION;
        uint256 numerator = amountInWithFee * vReserveOut;
        uint256 denominator = vReserveIn + amountInWithFee;
        amountOut = numerator / denominator;
    }

    /// @notice Calculates pool prices for specified amounts
    /// @param amountIn The amount of the token being sold.
    /// @param reserveIn The reserve of the token being sold.
    /// @param reserveOut The reserve of the token being bought.
    /// @param vReserveIn The (amplified) reserve of the token being sold.
    /// @param vReserveOut The (amplified) reserve of the token being bought.
    /// @param feeInPrecision Fee in Precision points
    /// @return The price as a fraction corresponding to the provided amount.
    function getPriceAt(
        uint256 amountIn,
        uint256 reserveIn,
        uint256 reserveOut,
        uint256 vReserveIn,
        uint256 vReserveOut,
        uint256 feeInPrecision
    ) internal pure returns (Fraction memory) {
        if (reserveIn == 0 || reserveOut == 0) {
            revert Unavailable("At least one reserve is zero!");
        }
        uint256 amountInWithFee =
            amountIn * (PRECISION - feeInPrecision) / PRECISION;
        uint256 numerator = amountInWithFee * vReserveOut;
        uint256 denominator = vReserveIn + amountInWithFee;
        uint256 amountOut = numerator / denominator;

        // get new amplified reserves
        uint256 newVReserveIn = vReserveIn + amountIn;
        uint256 newVReserveOut = vReserveOut - amountOut;

        return Fraction(
            newVReserveOut * (PRECISION - feeInPrecision) / PRECISION,
            newVReserveIn
        );
    }
}

interface IRouter {
    function factory() external view returns (address);
    function getAmountsIn(
        uint256 amountOut,
        address[] calldata poolsPath,
        IERC20[] calldata path
    ) external view returns (uint256[] memory amounts);
    function getAmountsOut(
        uint256 amounntIn,
        address[] calldata poolsPath,
        IERC20[] calldata path
    ) external view returns (uint256[] memory amounts);
}

interface IPair {
    function token0() external view returns (address);
    function token1() external view returns (address);
    function getReserves()
        external
        view
        returns (uint112 reserve0, uint112 reserve1);

    function swap(
        uint256 amount0Out,
        uint256 amount1Out,
        address to,
        bytes calldata data
    ) external;

    function ampBps() external view returns (uint32);

    function getTradeInfo()
        external
        view
        returns (
            uint112 _reserve0,
            uint112 _reserve1,
            uint112 _vReserve0,
            uint112 _vReserve1,
            uint256 feeInPrecision
        );

    function getVolumeTrendData()
        external
        view
        returns (
            uint128 _shortEMA,
            uint128 _longEMA,
            uint128 _currentBlockVolume,
            uint128 _lastTradeBlock
        );
}

interface IFactory {
    function allPools(uint256) external view returns (address);

    function allPoolsLength() external view returns (uint256);
}
