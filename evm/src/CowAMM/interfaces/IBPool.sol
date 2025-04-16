// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity ^0.8.13;

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";

interface IBPool is IERC20 {
  /**
   * @dev Struct for token records.
   * @param bound If token is bound to pool.
   * @param index Internal index of token array.
   * @param denorm Denormalized weight of token.
   */
  struct Record {
    bool bound;
    uint256 index;
    uint256 denorm;
  }

  /**
   * @notice Emitted when a swap is executed
   * @param caller The caller of the swap function
   * @param tokenIn The address of the token being swapped in
   * @param tokenOut The address of the token being swapped out
   * @param tokenAmountIn The amount of tokenIn being swapped in
   * @param tokenAmountOut The amount of tokenOut being swapped out
   */
  event LOG_SWAP(
    address indexed caller,
    address indexed tokenIn,
    address indexed tokenOut,
    uint256 tokenAmountIn,
    uint256 tokenAmountOut
  );

  /**
   * @notice Emitted when a join operation is executed
   * @param caller The caller of the function
   * @param tokenIn The address of the token being sent to the pool
   * @param tokenAmountIn The balance of the token being sent to the pool
   */
  event LOG_JOIN(address indexed caller, address indexed tokenIn, uint256 tokenAmountIn);

  /**
   * @notice Emitted when a token amount is removed from the pool
   * @param caller The caller of the function
   * @param tokenOut The address of the token being removed from the pool
   * @param tokenAmountOut The amount of the token being removed from the pool
   */
  event LOG_EXIT(address indexed caller, address indexed tokenOut, uint256 tokenAmountOut);

  /**
   * @notice Emitted when a call is executed on the pool
   * @param sig The signature of the function selector being called
   * @param caller The caller of the function
   * @param data The complete data of the call
   */
  event LOG_CALL(bytes4 indexed sig, address indexed caller, bytes data) anonymous;

  /**
   * @notice Thrown when setting a variable to address zero
   */
  error BPool_AddressZero();

  /**
   * @notice Thrown when a reentrant call is made
   */
  error BPool_Reentrancy();

  /**
   * @notice Thrown when the pool is finalized
   */
  error BPool_PoolIsFinalized();

  /**
   * @notice Thrown when the caller is not the controller
   */
  error BPool_CallerIsNotController();

  /**
   * @notice Thrown when the pool is not finalized
   */
  error BPool_FeeBelowMinimum();

  /**
   * @notice Thrown when the fee to set is above the maximum
   */
  error BPool_FeeAboveMaximum();

  /**
   * @notice Thrown when the tokens array is below the minimum
   */
  error BPool_TokensBelowMinimum();

  /**
   * @notice Thrown when the token is already bound in the pool
   */
  error BPool_TokenAlreadyBound();

  /**
   * @notice Thrown when the tokens array is above the maximum
   */
  error BPool_TokensAboveMaximum();

  /**
   * @notice Thrown when the weight to set is below the minimum
   */
  error BPool_WeightBelowMinimum();

  /**
   * @notice Thrown when the weight to set is above the maximum
   */
  error BPool_WeightAboveMaximum();

  /**
   * @notice Thrown when the balance to add is below the minimum
   */
  error BPool_BalanceBelowMinimum();

  /**
   * @notice Thrown when the total weight is above the maximum
   */
  error BPool_TotalWeightAboveMaximum();

  /**
   * @notice Thrown when the ratio between the pool token amount and the total supply is zero
   */
  error BPool_InvalidPoolRatio();

  /**
   * @notice Thrown when the calculated token amount in is zero
   */
  error BPool_InvalidTokenAmountIn();

  /**
   * @notice Thrown when the token amount in is above maximum amount in allowed by the caller
   */
  error BPool_TokenAmountInAboveMaxAmountIn();

  /**
   * @notice Thrown when the calculated token amount out is zero
   */
  error BPool_InvalidTokenAmountOut();

  /**
   * @notice Thrown when the token amount out is below minimum amount out allowed by the caller
   */
  error BPool_TokenAmountOutBelowMinAmountOut();

  /**
   * @notice Thrown when the token is not bound in the pool
   */
  error BPool_TokenNotBound();

  /**
   * @notice Thrown when the pool is not finalized
   */
  error BPool_PoolNotFinalized();

  /**
   * @notice Thrown when the token amount in surpasses the maximum in ratio allowed by the pool
   */
  error BPool_TokenAmountInAboveMaxRatio();

  /**
   * @notice Thrown when the spot price before or after the swap is above the max allowed by the caller
   */
  error BPool_SpotPriceAboveMaxPrice();

  /**
   * @notice Thrown when the token amount out is below the minimum out allowed by the caller
   */
  error BPool_TokenAmountOutBelowMinOut();

  /**
   * @notice Thrown when the spot price after the swap is below the spot price before the swap
   */
  error BPool_SpotPriceAfterBelowSpotPriceBefore();

  /**
   * @notice Thrown when the spot price before the swap is above the ratio between the two tokens in the pool
   */
  error BPool_SpotPriceBeforeAboveTokenRatio();

  /**
   * @notice Thrown when the token amount out surpasses the maximum out allowed by the pool
   */
  error BPool_TokenAmountOutAboveMaxOut();

  /**
   * @notice Thrown when the pool token amount out is below the minimum pool token amount out allowed by the caller
   */
  error BPool_PoolAmountOutBelowMinPoolAmountOut();

  /**
   * @notice Thrown when the calculated pool token amount in is zero
   */
  error BPool_InvalidPoolAmountIn();

  /**
   * @notice Thrown when the pool token amount in is above the maximum amount in allowed by the caller
   */
  error BPool_PoolAmountInAboveMaxPoolAmountIn();

  /**
   * @notice Sets the new swap fee
   * @param swapFee The new swap fee
   */
  function setSwapFee(uint256 swapFee) external;

  /**
   * @notice Sets the new controller
   * @param manager The new controller
   */
  function setController(address manager) external;

  /**
   * @notice Finalize the pool, removing the restrictions on the pool
   */
  function finalize() external;

  /**
   * @notice Binds a token to the pool
   * @param token The address of the token to bind
   * @param balance The balance of the token to bind
   * @param denorm The denormalized weight of the token to bind
   */
  function bind(address token, uint256 balance, uint256 denorm) external;

  /**
   * @notice Unbinds a token from the pool
   * @param token The address of the token to unbind
   */
  function unbind(address token) external;

  /**
   * @notice Joins a pool, providing each token in the pool with a proportional amount
   * @param poolAmountOut The amount of pool tokens to mint
   * @param maxAmountsIn The maximum amount of tokens to send to the pool
   */
  function joinPool(uint256 poolAmountOut, uint256[] calldata maxAmountsIn) external;

  /**
   * @notice Exits a pool, receiving each token in the pool with a proportional amount
   * @param poolAmountIn The amount of pool tokens to burn
   * @param minAmountsOut The minimum amount of tokens to receive from the pool
   */
  function exitPool(uint256 poolAmountIn, uint256[] calldata minAmountsOut) external;

  /**
   * @notice Swaps an exact amount of tokens in for an amount of tokens out
   * @param tokenIn The address of the token to swap in
   * @param tokenAmountIn The amount of token to swap in
   * @param tokenOut The address of the token to swap out
   * @param minAmountOut The minimum amount of token to receive from the swap
   * @param maxPrice The maximum price to pay for the swap
   * @return tokenAmountOut The amount of token swapped out
   * @return spotPriceAfter The spot price after the swap
   */
  function swapExactAmountIn(
    address tokenIn,
    uint256 tokenAmountIn,
    address tokenOut,
    uint256 minAmountOut,
    uint256 maxPrice
  ) external returns (uint256 tokenAmountOut, uint256 spotPriceAfter);

  /**
   * @notice Swaps as many tokens in as needed for an exact amount of tokens out
   * @param tokenIn The address of the token to swap in
   * @param maxAmountIn The maximum amount of token to swap in
   * @param tokenOut The address of the token to swap out
   * @param tokenAmountOut The amount of token to swap out
   * @param maxPrice The maximum price to pay for the swap
   * @return tokenAmountIn The amount of token swapped in
   * @return spotPriceAfter The spot price after the swap
   */
  function swapExactAmountOut(
    address tokenIn,
    uint256 maxAmountIn,
    address tokenOut,
    uint256 tokenAmountOut,
    uint256 maxPrice
  ) external returns (uint256 tokenAmountIn, uint256 spotPriceAfter);

  /**
   * @notice Gets the spot price of tokenIn in terms of tokenOut
   * @param tokenIn The address of the token to swap in
   * @param tokenOut The address of the token to swap out
   * @return spotPrice The spot price of the swap
   */
  function getSpotPrice(address tokenIn, address tokenOut) external view returns (uint256 spotPrice);

  /**
   * @notice Gets the spot price of tokenIn in terms of tokenOut without the fee
   * @param tokenIn The address of the token to swap in
   * @param tokenOut The address of the token to swap out
   * @return spotPrice The spot price of the swap without the fee
   */
  function getSpotPriceSansFee(address tokenIn, address tokenOut) external view returns (uint256 spotPrice);

  /**
   * @notice Gets the finalized status of the pool
   * @return isFinalized True if the pool is finalized, False otherwise
   */
  function isFinalized() external view returns (bool isFinalized);

  /**
   * @notice Gets the bound status of a token
   * @param t The address of the token to check
   * @return isBound True if the token is bound, False otherwise
   */
  function isBound(address t) external view returns (bool isBound);

  /**
   * @notice Gets the number of tokens in the pool
   * @return numTokens The number of tokens in the pool
   */
  function getNumTokens() external view returns (uint256 numTokens);

  /**
   * @notice Gets the current array of tokens in the pool, while the pool is not finalized
   * @return tokens The array of tokens in the pool
   */
  function getCurrentTokens() external view returns (address[] memory tokens);

  /**
   * @notice Gets the final array of tokens in the pool, after finalization
   * @return tokens The array of tokens in the pool
   */
  function getFinalTokens() external view returns (address[] memory tokens);

  /**
   * @notice Gets the denormalized weight of a token in the pool
   * @param token The address of the token to check
   * @return denormWeight The denormalized weight of the token in the pool
   */
  function getDenormalizedWeight(address token) external view returns (uint256 denormWeight);

  /**
   * @notice Gets the total denormalized weight of the pool
   * @return totalDenormWeight The total denormalized weight of the pool
   */
  function getTotalDenormalizedWeight() external view returns (uint256 totalDenormWeight);

  /**
   * @notice Gets the normalized weight of a token in the pool
   * @param token The address of the token to check
   * @return normWeight The normalized weight of the token in the pool
   */
  function getNormalizedWeight(address token) external view returns (uint256 normWeight);

  /**
   * @notice Gets the Pool's ERC20 balance of a token
   * @param token The address of the token to check
   * @return balance The Pool's ERC20 balance of the token
   */
  function getBalance(address token) external view returns (uint256 balance);

  /**
   * @notice Gets the swap fee of the pool
   * @return swapFee The swap fee of the pool
   */
  function getSwapFee() external view returns (uint256 swapFee);

  /**
   * @notice Gets the controller of the pool
   * @return controller The controller of the pool
   */
  function getController() external view returns (address controller);

  /**
   * @notice Gets the BFactory address that deployed the pool
   * @return factory The address of the factory
   */
  // solhint-disable-next-line style-guide-casing
  function FACTORY() external view returns (address factory);
}