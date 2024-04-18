// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {
    IERC20,
    SafeERC20
} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title Kyberswap Classic Adapter
contract KyberSwapClassicAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    // Kyberswap handles arbirary amounts, but we limit the amount to 10x just in case
    uint256 constant RESERVE_LIMIT_FACTOR = 10;

    IFactory factory;

    constructor(address _factory) {
        factory = IFactory(_factory);
    }

    function price(
        bytes32 _poolId,
        address _sellToken,
        address _buyToken,
        uint256[] memory _specifiedAmounts
    ) external view override returns (Fraction[] memory _prices) {
        revert NotImplemented("TemplateSwapAdapter.price");
    }

    function swap(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external returns (Trade memory trade) {
        revert NotImplemented("TemplateSwapAdapter.swap");
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32 poolId, address sellToken, address buyToken)
        external
        view
        override
        returns (uint256[] memory limits)
    {
        IUniswapV2Pair pair = IUniswapV2Pair(address(bytes20(poolId)));
        limits = new uint256[](2);
        (uint256 r0, uint256 r1,) = pair.getReserves();
        if (sellToken < buyToken) {
            limits[0] = r0 / RESERVE_LIMIT_FACTOR;
            limits[1] = r1 / RESERVE_LIMIT_FACTOR;
        } else {
            limits[0] = r1 / RESERVE_LIMIT_FACTOR;
            limits[1] = r0 / RESERVE_LIMIT_FACTOR;
        }
    }

    function getCapabilities(
        bytes32 poolId,
        address sellToken,
        address buyToken
    ) external returns (Capability[] memory capabilities) {
        revert NotImplemented("TemplateSwapAdapter.getCapabilities");
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(bytes32 poolId)
        external
        returns (address[] memory tokens)
    {
        tokens = new address[](2);
        IUniswapV2Pair pair = IUniswapV2Pair(address(bytes20(poolId)));
        tokens[0] = address(pair.token0());
        tokens[1] = address(pair.token1());
    }

    /// @inheritdoc ISwapAdapter
    function getPoolIds(uint256 offset, uint256 limit)
        external
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
}


interface IUniswapV2Pair {

    function token0() external view returns (address);
    function token1() external view returns (address);
    function getReserves()
        external
        view
        returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);

    function swap(
        uint256 amount0Out,
        uint256 amount1Out,
        address to,
        bytes calldata data
    ) external;

}

interface IFactory {

    function allPools(uint256) external view returns (address);

    function allPoolsLength() external view returns (uint256);

}
