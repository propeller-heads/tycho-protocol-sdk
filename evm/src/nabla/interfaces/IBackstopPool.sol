// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.13;

import {IPool} from "./IPool.sol";
import {IRouter} from "./IRouter.sol";

/**
 * @notice Public functions of the backstop pool.
 */
interface IBackstopPool is IPool {
    /**
     * @notice emitted on every withdrawal
     * @notice special case withdrawal using swap liquidiity: amountPrincipleWithdrawn = 0
     */
    event Burn(
        address indexed sender,
        uint256 poolSharesBurned,
        uint256 amountPrincipleWithdrawn
    );

    /**
     * @notice emitted when a swap pool LP withdraws from backstop pool
     */
    event CoverSwapWithdrawal(
        address indexed owner,
        address swapPool,
        uint256 amountSwapShares,
        uint256 amountSwapTokens,
        uint256 amountBackstopTokens
    );

    /**
     * @notice emitted on every deposit
     */
    event Mint(
        address indexed sender,
        uint256 poolSharesMinted,
        uint256 amountPrincipleDeposited
    );

    /**
     * @notice emitted when a backstop pool LP withdraws liquidity from swap pool
     */
    event WithdrawSwapLiquidity(
        address indexed owner,
        address swapPool,
        uint256 amountSwapTokens,
        uint256 amountBackstopTokens
    );

    function redeemSwapPoolShares(
        address swapPool,
        uint256 shares,
        uint256 minAmount
    ) external returns (uint256 amount);

    function withdrawExcessSwapLiquidity(
        address swapPool,
        uint256 shares,
        uint256 minAmount
    ) external returns (uint256 amount);

    function getBackedPool(uint256 index)
        external
        view
        returns (address swapPool);

    function getBackedPoolCount() external view returns (uint256 count);

    function getInsuranceFee(address swapPool)
        external
        view
        returns (uint256 feeBps);

    function getTotalPoolWorth() external view returns (int256 value);

    function router() external view returns (IRouter _router);
}

interface IBackstopPoolPermissioned is IBackstopPool {
    function addSwapPool(address swapPool, uint256 insuranceFeeBps) external;
}
