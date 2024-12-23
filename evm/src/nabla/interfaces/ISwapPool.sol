//SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import {IBackstopPool} from "./IBackstopPool.sol";
import {IPool} from "./IPool.sol";
import {IRouter} from "./IRouter.sol";
import {ISlippageCurve} from "./ISlippageCurve.sol";

/**
 * @notice Public functions of the SwapPool.
 */
interface ISwapPool is IPool {
    /**
     * @notice emitted on every withdrawal
     * @notice special case withdrawal using backstop liquidiity: amountPrincipleWithdrawn = 0
     */
    event Burn(
        address indexed sender,
        uint256 poolSharesBurned,
        uint256 amountPrincipleWithdrawn
    );

    /**
     * @notice Tracks the exact amounts of individual fees paid during a swap
     */
    event ChargedSwapFees(
        uint256 lpFees,
        uint256 backstopFees,
        uint256 protocolFees
    );

    /**
     * @notice emitted on every deposit
     */
    event Mint(
        address indexed sender,
        uint256 poolSharesMinted,
        uint256 amountPrincipleDeposited
    );

    function coverage()
        external
        view
        returns (uint256 reserves, uint256 liabilities);

    function totalLiabilities()
        external
        view
        returns (uint256 totalLiabilities);

    function reserve() external view returns (uint256 reserve);

    function reserveWithSlippage()
        external
        view
        returns (uint256 reserveWithSlippage);

    function slippageCurve()
        external
        view
        returns (ISlippageCurve slippageCurve);

    function backstop() external view returns (IBackstopPool _pool);

    function protocolTreasury() external view returns (address);

    function quoteSwapInto(uint256 _amount)
        external
        view
        returns (uint256 _effectiveAmount);

    function quoteSwapOut(uint256 _amount)
        external
        view
        returns (uint256 _effectiveAmount);

    function router() external view returns (IRouter _router);

    function sharesTargetWorth(uint256 _shares)
        external
        view
        returns (uint256 _amount);

    /// @notice get swap fees (applied when swapping liquidity out), in 0.0001%
    function swapFees()
        external
        view
        returns (uint256 _lpFee, uint256 _backstopFee, uint256 _protocolFee);

    function getExcessLiquidity()
        external
        view
        returns (int256 _excessLiquidity);
}

/**
 * @notice Access-restricted functions of the SwapPool.
 */
interface ISwapPoolPermissioned is ISwapPool {
    /**
     * @notice emitted when a backstop pool LP withdraws liquidity from swap pool
     * @notice only possible if swap pool coverage ratio remains >= 100%
     */
    event BackstopDrain(address recipient, uint256 amountSwapTokens);

    /// @notice for swap pool LP backstop withdrawal
    /// @param shares    number of lp tokens to burn
    function backstopBurn(address owner, uint256 shares)
        external
        returns (uint256 amount);

    /// @notice for backstop pool to withdraw liquidity if swap pool's coverage ratio > 100%
    /// @param amount   amount of swap pool reserves to withdraw
    function backstopDrain(uint256 amount, address recipient)
        external
        returns (uint256 swapAmount);

    /// @notice update the fees that the pool charges on every swap
    /// @param lpFeeBps         fee that benefits the pool's LPers, in basis points
    /// @param backstopFeeBps   fee that benefits the backstop pool, in basis points
    /// @param protocolFeeBps   fee that benefits the protocol, in basis points
    function setSwapFees(
        uint256 lpFeeBps,
        uint256 backstopFeeBps,
        uint256 protocolFeeBps
    ) external;

    function swapIntoFromRouter(uint256 amount)
        external
        returns (uint256 effectiveAmount);

    function swapOutFromRouter(uint256 amount)
        external
        returns (uint256 effectiveAmount);

    function pause() external;

    function unpause() external;
}
