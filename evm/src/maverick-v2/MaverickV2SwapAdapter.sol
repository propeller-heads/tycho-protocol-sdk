// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {
    IERC20,
    SafeERC20
} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

uint256 constant RESERVE_LIMIT_FACTOR = 10;
/// @title MaverickV2SwapAdapter
/// @dev This is a template for a swap adapter.

contract MaverickV2SwapAdapter is ISwapAdapter {
    IMaverickV2Factory public immutable factory;

    constructor(address factory_) {
        factory = IMaverickV2Factory(factory_);
    }

    function price(
        bytes32 poolId,
        address sellToken,
        address _buyToken,
        uint256[] memory specifiedAmounts
    ) external override returns (Fraction[] memory calculatedPrices) {
        calculatedPrices = new Fraction[](specifiedAmounts.length);
        for (uint256 i = 0; i < specifiedAmounts.length; i++) {
            calculatedPrices[i] = priceAt(
                IMaverickV2Pool(address(bytes20(poolId))),
                sellToken,
                specifiedAmounts[i]
            );
        }
    }

    function priceAt(
        IMaverickV2Pool pool,
        address sellToken,
        uint256 sellAmount
    ) public returns (Fraction memory calculatedPrice) {
        bool isTokenAIn = (sellToken == address(pool.tokenA()));

        IMaverickV2Pool.SwapParams memory swapParams = IMaverickV2Pool
            .SwapParams({
            amount: sellAmount,
            tokenAIn: isTokenAIn,
            exactOutput: false,
            tickLimit: isTokenAIn ? type(int32).max : type(int32).min
        });

        (uint256 amountIn, uint256 amountOut) =
            pool.swap(address(this), swapParams, "");

        calculatedPrice =
            Fraction({numerator: amountOut, denominator: amountIn});
    }

    function maverickV2SwapCallback(
        IERC20 tokenIn,
        uint256 amountIn,
        uint256 amountOut,
        bytes calldata data
    ) external {
        // todo: check caller
        (address caller, address sellToken, address buyToken) =
            abi.decode(data, (address, address, address));

        tokenIn.transferFrom(caller, msg.sender, amountIn);
    }

    function swap(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external returns (Trade memory trade) {
        IMaverickV2Pool pool = IMaverickV2Pool(address(bytes20(poolId)));
        bool isTokenAIn = sellToken == address(pool.tokenA());
        IMaverickV2Pool.SwapParams memory swapParams = IMaverickV2Pool
            .SwapParams({
            amount: specifiedAmount,
            tokenAIn: isTokenAIn,
            exactOutput: side == OrderSide.Buy,
            tickLimit: 0
        });
        uint256 initialGas = gasleft();
        IERC20(sellToken).approve(address(pool), 0); // todo amount??
        bytes memory data = abi.encode(msg.sender, sellToken, buyToken);
        (uint256 amountIn, uint256 amountOut) =
            pool.swap(msg.sender, swapParams, data);

        uint256 gasUsed = initialGas - gasleft();
        trade.calculatedAmount = (side == OrderSide.Sell) ? amountIn : amountOut;
        trade.gasUsed = gasUsed;
        if (side == OrderSide.Sell) {
            // Price = amountOut / amountIn
            trade.price = Fraction(amountOut, amountIn);
        } else {
            // Price = amountIn / amountOut
            trade.price = Fraction(amountIn, amountOut);
        }
    }

    function getLimits(bytes32 poolId, address sellToken, address buyToken)
        external
        view
        returns (uint256[] memory limits)
    {
        IMaverickV2Pool pool = IMaverickV2Pool(address(bytes20(poolId)));
        IMaverickV2Pool.State memory state = pool.getState();

        limits = new uint256[](2);
        uint256 r0 = state.reserveA;
        uint256 r1 = state.reserveB;
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
        capabilities = new Capability[](4);
        capabilities[0] = Capability.SellOrder;
        capabilities[1] = Capability.BuyOrder;
        capabilities[2] = Capability.PriceFunction;
        capabilities[3] = Capability.MarginalPrice;
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(bytes32 poolId)
        external
        view
        override
        returns (address[] memory tokens)
    {
        tokens = new address[](2);
        IMaverickV2Pool pool = IMaverickV2Pool(address(bytes20(poolId)));
        tokens[0] = address(pool.tokenA());
        tokens[1] = address(pool.tokenB());
    }

    /// @inheritdoc ISwapAdapter
    function getPoolIds(uint256 offset, uint256 limit)
        external
        view
        override
        returns (bytes32[] memory ids)
    {
        IMaverickV2Pool[] memory pools = factory.lookup(offset, offset + limit);
        ids = new bytes32[](pools.length);
        for (uint256 i = 0; i < pools.length; i++) {
            ids[i] = bytes20((address(pools[i])));
        }
    }
}

interface IMaverickV2Pool {
    struct SwapParams {
        uint256 amount;
        bool tokenAIn;
        bool exactOutput;
        int32 tickLimit;
    }

    struct State {
        uint128 reserveA;
        uint128 reserveB;
        int64 lastTwaD8;
        int64 lastLogPriceD8;
        uint40 lastTimestamp;
        int32 activeTick;
        bool isLocked;
        uint32 binCounter;
        uint8 protocolFeeRatioD3;
    }

    function tokenA() external view returns (IERC20);
    function tokenB() external view returns (IERC20);
    function factory() external view returns (IMaverickV2Factory);
    function getState() external view returns (State memory);
    function swap(
        address recipient,
        SwapParams memory params,
        bytes calldata data
    ) external returns (uint256 amountIn, uint256 amountOut);
}

interface IMaverickV2Factory {
    function lookup(uint256 startIndex, uint256 endIndex)
        external
        view
        returns (IMaverickV2Pool[] memory pools);
}
