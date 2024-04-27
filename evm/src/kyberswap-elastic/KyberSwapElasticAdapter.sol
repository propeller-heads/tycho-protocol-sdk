// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {ERC20} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import {IERC20, SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

struct PoolData {
    uint160 sqrtP;
    int24 nearestCurrentTick;
    int24 currentTick;
    uint128 baseL;
    uint128 reinvestL;
    uint128 reinvestLLast;
}

/// @title KyberSwap Elastic Adapter
contract KyberSwapElasticAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    /// @dev custom limit factor for limits/reserves
    uint256 RESERVE_LIMIT_FACTOR = 10;

    /// @dev The minimum value that can be returned from #getSqrtRatioAtTick. Equivalent to getSqrtRatioAtTick(MIN_TICK)
    uint160 constant MIN_SQRT_RATIO = 4295128739;
    /// @dev The maximum value that can be returned from #getSqrtRatioAtTick. Equivalent to getSqrtRatioAtTick(MAX_TICK)
    uint160 constant MAX_SQRT_RATIO = 1461446703485210103287273052203988822378723970342;

    IElasticFactory elasticFactory;

    constructor(address _elasticFactory) {
        elasticFactory = IElasticFactory(_elasticFactory);
    }

    /// @inheritdoc ISwapAdapter
    /// @dev Price(tick) in KyberSwap Elastic is obtained externally from oracle, which only serves an average Price/Tick between timespans
    /// Therefore is not accurate.
    function price(
        bytes32 _poolId,
        address _sellToken,
        address _buyToken,
        uint256[] memory _specifiedAmounts
    ) external view override returns (Fraction[] memory _prices) {
        revert NotImplemented("KyberSwapElasticAdapter.price");
    }

    /// @inheritdoc ISwapAdapter
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
        address poolAddress = address(bytes20(poolId));
        IElasticPool pool = IElasticPool(poolAddress);

        uint256 gasBefore = gasleft();
        if (side == OrderSide.Sell) {
            trade.calculatedAmount =
                sell(pool, sellToken, buyToken, specifiedAmount);
        } else {
            trade.calculatedAmount =
                buy(pool, sellToken, buyToken, specifiedAmount);
        }
        trade.gasUsed = gasBefore - gasleft();
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(
        bytes32 poolId,
        address sellToken,
        address buyToken
    ) external returns (uint256[] memory limits) {
        address poolAddress = address(bytes20(poolId));
        limits = new uint256[](2);
        limits[0] =
            IERC20(sellToken).balanceOf(poolAddress) /
            RESERVE_LIMIT_FACTOR;
        limits[1] =
            IERC20(buyToken).balanceOf(poolAddress) /
            RESERVE_LIMIT_FACTOR;
    }

    /// @inheritdoc ISwapAdapter
    function getCapabilities(bytes32, address, address)
        external
        pure
        override
        returns (Capability[] memory capabilities)
    {
        capabilities = new Capability[](2);
        capabilities[0] = Capability.SellOrder;
        capabilities[1] = Capability.BuyOrder;
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(
        bytes32 poolId
    ) external view override returns (address[] memory tokens) {
        tokens = new address[](2);
        IElasticPool pool = IElasticPool(address(bytes20(poolId)));
        tokens[0] = address(pool.token0());
        tokens[1] = address(pool.token1());
    }

    function getPoolIds(
        uint256 offset,
        uint256 limit
    ) external returns (bytes32[] memory ids) {
        revert NotImplemented("KyberSwapElasticAdapter.getPoolIds");
    }

    /// @notice Execute a sell order on a given pool
    /// @param sellToken token to sell
    /// @param buyToken token to buy
    /// @param specifiedAmount amount of sellToken to sell
    /// @return (uint256) buyToken amount received
    function sell(IElasticPool pool, address sellToken, address buyToken, uint256 specifiedAmount) internal returns (uint256) {
        bool isToken0 = pool.token0() == sellToken;
        uint160 limitSqrtP = isToken0 ? MIN_SQRT_RATIO+1 : MAX_SQRT_RATIO-1;
        uint256 balBefore = IERC20(buyToken).balanceOf(msg.sender);
        pool.swap(msg.sender, int256(specifiedAmount), isToken0, limitSqrtP, "");
        return IERC20(buyToken).balanceOf(msg.sender) - balBefore;
    }

    /// @notice Execute a buy order on a given pool
    /// @param sellToken token to sell
    /// @param buyToken token to buy
    /// @param specifiedAmount amount of buyToken to buy
    /// @return (uint256) sellToken amount spent
    function buy(IElasticPool pool, address sellToken, address buyToken, uint256 specifiedAmount) internal returns (uint256) {
        bool isToken0 = pool.token0() == sellToken;
        uint160 limitSqrtP = isToken0 ? MIN_SQRT_RATIO+1 : MAX_SQRT_RATIO-1;
        uint256 balBefore = IERC20(sellToken).balanceOf(address(this));
        pool.swap(msg.sender, - int256(specifiedAmount), isToken0, limitSqrtP, "");
        return balBefore - IERC20(sellToken).balanceOf(address(this));
    }
}

interface IElasticFactory {

    function getPool(
        address token0,
        address token1,
        uint24 swapFee
    ) external view returns (address);
}

interface IElasticPool {
    function token0() external view returns (address);
    function token1() external view returns (address);
    function getPoolState()
        external
        view
        returns (
            uint160 sqrtP,
            int24 currentTick,
            int24 nearestCurrentTick,
            bool locked
        );
    function getLiquidityState()
        external
        view
        returns (uint128 baseL, uint128 reinvestL, uint128 reinvestLLast);
    function initializedTicks(int24 i) external view returns (int24 previous, int24 next);
    function swap(
        address recipient,
        int256 swapQty,
        bool isToken0,
        uint160 limitSqrtP, // MAX_SQRT_RATIO-1 when swapping 1 -> 0 and MIN_SQRT_RATIO+1 when swapping 0 -> 1 for no limit swap
        bytes calldata data
    ) external returns (int256 deltaQty0, int256 deltaQty1);
}
