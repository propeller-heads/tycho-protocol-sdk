// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.13;

import {IPriceOracleGetter} from "../interfaces/IPriceOracleGetter.sol";
import {ISwapPoolPermissioned} from "../interfaces/ISwapPool.sol";

interface IRouter {
    /**
     * Emitted on each swap
     */
    event Swap(
        address indexed sender,
        uint256 amountIn,
        uint256 amountOut,
        address tokenIn,
        address tokenOut,
        address indexed to
    );

    /** Emitted when a new pool is registered */
    event SwapPoolRegistered(
        address indexed sender,
        address pool,
        address asset
    );

    /** Emitted when pool is unregistered */
    event SwapPoolUnregistered(address indexed sender, address asset);

    function oracleByAsset(
        address asset
    ) external view returns (IPriceOracleGetter);

    function poolByAsset(
        address asset
    ) external view returns (ISwapPoolPermissioned);

    function swapExactTokensForTokens(
        uint256 _amountIn,
        uint256 _amountOutMin,
        address[] calldata _tokenInOut,
        address _to,
        uint256 _deadline
    ) external returns (uint256[] memory _amounts);

    function getAmountOut(
        uint256 _amountIn,
        address[] calldata _tokenInOut
    ) external view returns (uint256 _amountOut);
}

interface IRouterPermissioned is IRouter {
    function pause() external;

    function unpause() external;
}
