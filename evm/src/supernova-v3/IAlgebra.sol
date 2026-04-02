// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

interface IAlgebraPool {
    function token0() external view returns (address);
    function token1() external view returns (address);
    function factory() external view returns (address);
    function globalState() external view returns (
        uint160 price,
        int24 tick,
        uint16 lastFee,
        uint8 pluginConfig,
        uint16 communityFee,
        bool unlocked
    );
    function liquidity() external view returns (uint128);
    function getReserves() external view returns (uint128, uint128);
    function swap(
        address recipient,
        bool zeroToOne,
        int256 amountRequired,
        uint160 limitSqrtPrice,
        bytes calldata data
    ) external returns (int256 amount0, int256 amount1);
}

interface IAlgebraFactory {
    function poolByPair(address tokenA, address tokenB) external view returns (address pool);
}
