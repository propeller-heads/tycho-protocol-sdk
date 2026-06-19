// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {IERC20, ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {SafeERC20} from
    "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

// FraxSwapV2 handles arbirary amounts, but we limit the amount to 10x just in
// case
uint256 constant RESERVE_LIMIT_FACTOR = 10;

/// @title Frax Swap Adapter
/// @dev The fee() variable in FraxSwapV2 is ( 100%(10000) - fee(e.g. 5) ), so
/// it is a net amount and not a fee
contract FraxSwapV2SwapAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    IUniswapV2FactoryV5 immutable factory;

    constructor(address factory_) {
        factory = IUniswapV2FactoryV5(factory_);
    }

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32 _poolId,
        IERC20 _sellToken,
        IERC20,
        uint256[] memory _specifiedAmounts
    ) external view override returns (Fraction[] memory _prices) {
        _prices = new Fraction[](_specifiedAmounts.length);
        IUniswapV2PairPartialV5 pair =
            IUniswapV2PairPartialV5(address(bytes20(_poolId)));
        uint112 r0;
        uint112 r1;
        uint256 unitLessFee = pair.fee();
        if (address(_sellToken) == pair.token0()) {
            // sell
            (r0, r1,) = pair.getReserves();
        } else {
            // buy
            (r1, r0,) = pair.getReserves();
        }

        for (uint256 i = 0; i < _specifiedAmounts.length; i++) {
            _prices[i] = getPriceAt(_specifiedAmounts[i], r0, r1, unitLessFee);
        }
    }

    /// @inheritdoc ISwapAdapter
    function swap(
        bytes32 poolId,
        IERC20 sellToken,
        IERC20,
        OrderSide side,
        uint256 specifiedAmount
    ) external returns (Trade memory trade) {
        if (specifiedAmount == 0) {
            return trade;
        }

        IUniswapV2PairPartialV5 pair =
            IUniswapV2PairPartialV5(address(bytes20(poolId)));
        uint112 r0;
        uint112 r1;
        uint256 unitLessFee = pair.fee();
        bool zero2one = address(sellToken) == pair.token0();
        if (zero2one) {
            (r0, r1,) = pair.getReserves();
        } else {
            (r1, r0,) = pair.getReserves();
        }
        uint256 gasBefore = gasleft();
        if (side == OrderSide.Sell) {
            trade.calculatedAmount = sell(
                pair, sellToken, zero2one, r0, r1, specifiedAmount, unitLessFee
            );
        } else {
            trade.calculatedAmount = buy(
                pair, sellToken, zero2one, r0, r1, specifiedAmount, unitLessFee
            );
        }
        trade.gasUsed = gasBefore - gasleft();
        if (side == OrderSide.Sell) {
            trade.price = getPriceAt(specifiedAmount, r0, r1, unitLessFee);
        } else {
            trade.price =
                getPriceAt(trade.calculatedAmount, r0, r1, unitLessFee);
        }
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32 poolId, IERC20 sellToken, IERC20)
        external
        view
        override
        returns (uint256[] memory limits)
    {
        IUniswapV2PairPartialV5 pair =
            IUniswapV2PairPartialV5(address(bytes20(poolId)));
        limits = new uint256[](2);
        (uint256 r0, uint256 r1,) = pair.getReserves();
        if (address(sellToken) == pair.token0()) {
            limits[0] = r0 / RESERVE_LIMIT_FACTOR;
            limits[1] = r1 / RESERVE_LIMIT_FACTOR;
        } else {
            limits[0] = r1 / RESERVE_LIMIT_FACTOR;
            limits[1] = r0 / RESERVE_LIMIT_FACTOR;
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
    function getTokens(bytes32 poolId)
        external
        view
        override
        returns (IERC20[] memory tokens)
    {
        tokens = new IERC20[](2);
        IUniswapV2PairPartialV5 pair =
            IUniswapV2PairPartialV5(address(bytes20(poolId)));
        tokens[0] = IERC20(pair.token0());
        tokens[1] = IERC20(pair.token1());
    }

    /// @inheritdoc ISwapAdapter
    function getPoolIds(uint256 offset, uint256 limit)
        external
        view
        override
        returns (bytes32[] memory ids)
    {
        uint256 endIdx = offset + limit;
        if (endIdx > factory.allPairsLength()) {
            endIdx = factory.allPairsLength();
        }
        ids = new bytes32[](endIdx - offset);
        for (uint256 i = 0; i < ids.length; i++) {
            ids[i] = bytes20(factory.allPairs(offset + i));
        }
    }

    /// @notice Executes a sell order on a given pool.
    /// @param pair The pair to trade on.
    /// @param sellToken The token being sold.
    /// @param zero2one Whether the sell token is token0 or token1.
    /// @param reserveIn The reserve of the token being sold.
    /// @param reserveOut The reserve of the token being bought.
    /// @param amount The amount to be traded.
    /// @param unitLessFee The 100% - fee for this pair
    /// @return calculatedAmount The amount of tokens received.
    function sell(
        IUniswapV2PairPartialV5 pair,
        IERC20 sellToken,
        bool zero2one,
        uint112 reserveIn,
        uint112 reserveOut,
        uint256 amount,
        uint256 unitLessFee
    ) internal returns (uint256) {
        uint256 amountOut =
            getAmountOut(amount, reserveIn, reserveOut, unitLessFee);

        sellToken.safeTransferFrom(msg.sender, address(pair), amount);
        if (zero2one) {
            pair.swap(0, amountOut, msg.sender, "");
        } else {
            pair.swap(amountOut, 0, msg.sender, "");
        }
        return amountOut;
    }

    /// @notice Execute a buy order on a given pool.
    /// @param pair The pair to trade on.
    /// @param sellToken The token being sold.
    /// @param zero2one Whether the sell token is token0 or token1.
    /// @param reserveIn The reserve of the token being sold.
    /// @param reserveOut The reserve of the token being bought.
    /// @param amountOut The amount of tokens to be bought.
    /// @param unitLessFee The 100% - fee for this pair
    /// @return calculatedAmount The amount of tokens sold.
    function buy(
        IUniswapV2PairPartialV5 pair,
        IERC20 sellToken,
        bool zero2one,
        uint112 reserveIn,
        uint112 reserveOut,
        uint256 amountOut,
        uint256 unitLessFee
    ) internal returns (uint256) {
        uint256 amountIn =
            getAmountIn(amountOut, reserveIn, reserveOut, unitLessFee);

        sellToken.safeTransferFrom(msg.sender, address(pair), amountIn);
        if (zero2one) {
            pair.swap(0, amountOut, msg.sender, "");
        } else {
            pair.swap(amountOut, 0, msg.sender, "");
        }
        return amountIn;
    }

    /// @notice Calculates pool prices after trade for specified amounts
    /// @param amountIn The amount of the token being sold.
    /// @param reserveIn The reserve of the token being sold.
    /// @param reserveOut The reserve of the token being bought.
    /// @param unitLessFee The 100% - fee for this pair
    /// @dev Basis points(BP) is 10,000 for Frax
    function getPriceAt(
        uint256 amountIn,
        uint256 reserveIn,
        uint256 reserveOut,
        uint256 unitLessFee
    ) internal pure returns (Fraction memory) {
        if (reserveIn == 0 || reserveOut == 0) {
            revert Unavailable("At least one reserve is zero!");
        }

        uint256 amountInWithFee = amountIn * unitLessFee;
        uint256 numerator = amountInWithFee * reserveOut;
        uint256 denominator = (reserveIn * 10000) + amountInWithFee;
        uint256 amountOut = numerator / denominator;
        uint256 newReserveOut = reserveOut - amountOut;
        uint256 newReserveIn = reserveIn + amountIn;

        return Fraction(newReserveOut * 10000, newReserveIn * unitLessFee);
    }

    /// @notice Given an input amount of an asset and pair reserves, returns the
    /// maximum output amount of the other asset
    /// @param amountIn The amount of the token being sold.
    /// @param reserveIn The reserve of the token being sold.
    /// @param reserveOut The reserve of the token being bought.
    /// @param unitLessFee The 100% - fee for this pair
    /// @return amountOut The amount of tokens received.
    function getAmountOut(
        uint256 amountIn,
        uint256 reserveIn,
        uint256 reserveOut,
        uint256 unitLessFee
    ) internal pure returns (uint256 amountOut) {
        if (amountIn == 0) {
            return 0;
        }
        if (reserveIn == 0 || reserveOut == 0) {
            revert Unavailable("At least one reserve is zero!");
        }
        uint256 amountInWithFee = amountIn * unitLessFee;
        uint256 numerator = amountInWithFee * reserveOut;
        uint256 denominator = (reserveIn * 10000) + amountInWithFee;
        amountOut = numerator / denominator;
    }

    /// @notice Given an output amount of an asset and pair reserves, returns a
    /// required input amount of the other asset
    /// @param amountOut The amount of the token being bought.
    /// @param reserveIn The reserve of the token being sold.
    /// @param reserveOut The reserve of the token being bought.
    /// @param unitLessFee The 100% - fee for this pair
    function getAmountIn(
        uint256 amountOut,
        uint256 reserveIn,
        uint256 reserveOut,
        uint256 unitLessFee
    ) internal pure returns (uint256 amountIn) {
        if (amountOut == 0) {
            return 0;
        }
        if (reserveIn == 0) {
            revert Unavailable("reserveIn is zero");
        }
        if (reserveOut == 0) {
            revert Unavailable("reserveOut is zero");
        }
        uint256 numerator = reserveIn * amountOut * 10000;
        uint256 denominator = (reserveOut - amountOut) * unitLessFee;
        amountIn = (numerator / denominator) + 1;
    }
}

interface IUniswapV2FactoryV5 {
    event PairCreated(
        address indexed token0, address indexed token1, address pair, uint256
    );

    function feeTo() external view returns (address);
    function feeToSetter() external view returns (address);
    function globalPause() external view returns (bool);

    function getPair(address tokenA, address tokenB)
        external
        view
        returns (address pair);
    function allPairs(uint256) external view returns (address pair);
    function allPairsLength() external view returns (uint256);

    function createPair(address tokenA, address tokenB)
        external
        returns (address pair);
    function createPair(address tokenA, address tokenB, uint256 fee)
        external
        returns (address pair);

    function setFeeTo(address) external;
    function setFeeToSetter(address) external;
    function toggleGlobalPause() external;
}

interface IUniswapV2PairPartialV5 {
    event Mint(address indexed sender, uint256 amount0, uint256 amount1);
    event Burn(
        address indexed sender,
        uint256 amount0,
        uint256 amount1,
        address indexed to
    );
    event Swap(
        address indexed sender,
        uint256 amount0In,
        uint256 amount1In,
        uint256 amount0Out,
        uint256 amount1Out,
        address indexed to
    );
    event Sync(uint112 reserve0, uint112 reserve1);

    function MINIMUM_LIQUIDITY() external pure returns (uint256);
    function factory() external view returns (address);
    function token0() external view returns (address);
    function token1() external view returns (address);
    function getReserves()
        external
        view
        returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
    function price0CumulativeLast() external view returns (uint256);
    function price1CumulativeLast() external view returns (uint256);
    function kLast() external view returns (uint256);
    function fee() external view returns (uint256);

    function mint(address to) external returns (uint256 liquidity);
    function burn(address to)
        external
        returns (uint256 amount0, uint256 amount1);
    function swap(
        uint256 amount0Out,
        uint256 amount1Out,
        address to,
        bytes calldata data
    ) external;
    function skim(address to) external;
    function sync() external;
    function initialize(address, address, uint256) external;

    // TWAMM

    function longTermSwapFrom0To1(
        uint256 amount0In,
        uint256 numberOfTimeIntervals
    ) external returns (uint256 orderId);
    function longTermSwapFrom1To0(
        uint256 amount1In,
        uint256 numberOfTimeIntervals
    ) external returns (uint256 orderId);
    function cancelLongTermSwap(uint256 orderId) external;
    function withdrawProceedsFromLongTermSwap(uint256 orderId)
        external
        returns (bool is_expired, address rewardTkn, uint256 totalReward);
    function executeVirtualOrders(uint256 blockTimestamp) external;

    function getAmountOut(uint256 amountIn, address tokenIn)
        external
        view
        returns (uint256);
    function getAmountIn(uint256 amountOut, address tokenOut)
        external
        view
        returns (uint256);

    function orderTimeInterval() external returns (uint256);
    function getTWAPHistoryLength() external view returns (uint256);
    function getTwammReserves()
        external
        view
        returns (
            uint112 _reserve0,
            uint112 _reserve1,
            uint32 _blockTimestampLast,
            uint112 _twammReserve0,
            uint112 _twammReserve1,
            uint256 _fee
        );
    function getReserveAfterTwamm(uint256 blockTimestamp)
        external
        view
        returns (
            uint112 _reserve0,
            uint112 _reserve1,
            uint256 lastVirtualOrderTimestamp,
            uint112 _twammReserve0,
            uint112 _twammReserve1
        );
    function getNextOrderID() external view returns (uint256);
    function getOrderIDsForUser(address user)
        external
        view
        returns (uint256[] memory);
    function getOrderIDsForUserLength(address user)
        external
        view
        returns (uint256);
    //    function getDetailedOrdersForUser(address user, uint256 offset,
    // uint256 limit) external view returns (LongTermOrdersLib.Order[] memory
    // detailed_orders);
    function twammUpToDate() external view returns (bool);
    function getTwammState()
        external
        view
        returns (
            uint256 token0Rate,
            uint256 token1Rate,
            uint256 lastVirtualOrderTimestamp,
            uint256 orderTimeInterval_rtn,
            uint256 rewardFactorPool0,
            uint256 rewardFactorPool1
        );
    function getTwammSalesRateEnding(uint256 _blockTimestamp)
        external
        view
        returns (
            uint256 orderPool0SalesRateEnding,
            uint256 orderPool1SalesRateEnding
        );
    function getTwammRewardFactor(uint256 _blockTimestamp)
        external
        view
        returns (
            uint256 rewardFactorPool0AtTimestamp,
            uint256 rewardFactorPool1AtTimestamp
        );
    function getTwammOrder(uint256 orderId)
        external
        view
        returns (
            uint256 id,
            uint256 creationTimestamp,
            uint256 expirationTimestamp,
            uint256 saleRate,
            address owner,
            address sellTokenAddr,
            address buyTokenAddr
        );
    function getTwammOrderProceedsView(uint256 orderId, uint256 blockTimestamp)
        external
        view
        returns (bool orderExpired, uint256 totalReward);
    function getTwammOrderProceeds(uint256 orderId)
        external
        returns (bool orderExpired, uint256 totalReward);

    function togglePauseNewSwaps() external;
}
