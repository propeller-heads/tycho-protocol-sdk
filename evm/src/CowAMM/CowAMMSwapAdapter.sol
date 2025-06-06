// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {console2} from "forge-std/console2.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {IERC20Metadata} from
    "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {IBPool} from "./interfaces/IBPool.sol";
import "src/libraries/FractionMath.sol";
import {SafeERC20} from
    "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title CowAMMSwapAdapter

/**
 * @title BNum
 * @notice Includes functions for arithmetic operations with fixed-point numbers.
 * @dev The arithmetic operations are implemented with a precision of BONE.
 */
library BNumLib {
  /// @notice The unit of precision used in the calculations.
  uint256 public constant BONE = 10 ** 18;
  /// @notice The minimum base value for the bpow calculation.
  uint256 public constant MIN_BPOW_BASE = 1 wei;
  /// @notice The maximum base value for the bpow calculation.
  uint256 public constant MAX_BPOW_BASE = (2 * BONE) - 1 wei;
  /// @notice The precision of the bpow calculation.
  uint256 public constant BPOW_PRECISION = BONE / 10 ** 10;
  /**
   * @notice Thrown when an overflow is encountered inside the add function
   */
  error BNum_AddOverflow();

  /**
   * @notice Thrown when an underflow is encountered inside the sub function
   */
  error BNum_SubUnderflow();

  /**
   * @notice Thrown when an overflow is encountered inside the mul function
   */
  error BNum_MulOverflow();

  /**
   * @notice Thrown when attempting to divide by zero
   */
  error BNum_DivZero();

  /**
   * @notice Thrown when an internal error occurs inside div function
   */
  error BNum_DivInternal();

  /**
   * @notice Thrown when the base is too low in the bpow function
   */
  error BNum_BPowBaseTooLow();

  /**
   * @notice Thrown when the base is too high in the bpow function
   */
  error BNum_BPowBaseTooHigh();

  function btoi(uint256 a) internal pure returns (uint256) {
    unchecked {
      return a / BONE;
    }
  }

  function bfloor(uint256 a) internal pure returns (uint256) {
    unchecked {
      return btoi(a) * BONE;
    }
  }

  function badd(uint256 a, uint256 b) internal pure returns (uint256) {
    unchecked {
      uint256 c = a + b;
      if (c < a) {
        revert BNum_AddOverflow();
      }
      return c;
    }
  }

  function bsub(uint256 a, uint256 b) internal pure returns (uint256) {
    unchecked {
      (uint256 c, bool flag) = bsubSign(a, b);
      if (flag) {
        revert BNum_SubUnderflow();
      }
      return c;
    }
  }

  function bsubSign(uint256 a, uint256 b) internal pure returns (uint256, bool) {
    unchecked {
      if (a >= b) {
        return (a - b, false);
      } else {
        return (b - a, true);
      }
    }
  }

  function bmul(uint256 a, uint256 b) internal pure returns (uint256) {
    unchecked {
      uint256 c0 = a * b;
      if (a != 0 && c0 / a != b) {
        revert BNum_MulOverflow();
      }
      // NOTE: using >> 1 instead of / 2
      uint256 c1 = c0 + (BONE >> 1);
      if (c1 < c0) {
        revert BNum_MulOverflow();
      }
      uint256 c2 = c1 / BONE;
      return c2;
    }
  }

  function bdiv(uint256 a, uint256 b) internal pure returns (uint256) {
    unchecked {
      if (b == 0) {
        revert BNum_DivZero();
      }
      uint256 c0 = a * BONE;
      if (a != 0 && c0 / a != BONE) {
        revert BNum_DivInternal(); // bmul overflow
      }
      // NOTE: using >> 1 instead of / 2
      uint256 c1 = c0 + (b >> 1);
      if (c1 < c0) {
        revert BNum_DivInternal(); //  badd require
      }
      uint256 c2 = c1 / b;
      return c2;
    }
  }

  // DSMath.wpow
  function bpowi(uint256 a, uint256 n) internal pure returns (uint256) {
    unchecked {
      uint256 z = n % 2 != 0 ? a : BONE;

      for (n /= 2; n != 0; n /= 2) {
        a = bmul(a, a);

        if (n % 2 != 0) {
          z = bmul(z, a);
        }
      }
      return z;
    }
  }

  // Compute b^(e.w) by splitting it into (b^e)*(b^0.w).
  // Use `bpowi` for `b^e` and `bpowK` for k iterations
  // of approximation of b^0.w
  function bpow(uint256 base, uint256 exp) internal pure returns (uint256) {
    unchecked {
      if (base < MIN_BPOW_BASE) {
        revert BNum_BPowBaseTooLow();
      }
      if (base > MAX_BPOW_BASE) {
        revert BNum_BPowBaseTooHigh();
      }

      uint256 whole = bfloor(exp);
      uint256 remain = bsub(exp, whole);

      uint256 wholePow = bpowi(base, btoi(whole));

      if (remain == 0) {
        return wholePow;
      }

      uint256 partialResult = bpowApprox(base, remain, BPOW_PRECISION);
      return bmul(wholePow, partialResult);
    }
  }

  function bpowApprox(uint256 base, uint256 exp, uint256 precision) internal pure returns (uint256) {
    unchecked {
      // term 0:
      uint256 a = exp;
      (uint256 x, bool xneg) = bsubSign(base, BONE);
      uint256 term = BONE;
      uint256 sum = term;
      bool negative = false;

      // term(k) = numer / denom
      //         = (product(a - i - 1, i=1-->k) * x^k) / (k!)
      // each iteration, multiply previous term by (a-(k-1)) * x / k
      // continue until term is less than precision
      for (uint256 i = 1; term >= precision; i++) {
        uint256 bigK = i * BONE;
        (uint256 c, bool cneg) = bsubSign(a, bsub(bigK, BONE));
        term = bmul(term, bmul(c, x));
        term = bdiv(term, bigK);
        if (term == 0) break;

        if (xneg) negative = !negative;
        if (cneg) negative = !negative;
        if (negative) {
          sum = bsub(sum, term);
        } else {
          sum = badd(sum, term);
        }
      }

      return sum;
    }
  }
}



/// @dev This is the CowAMM swap adapter.

// 50% and 33%?
uint256 constant MAX_IN_FACTOR = 50;
uint256 constant MAX_OUT_FACTOR = 33;

contract CowAMMSwapAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;
    using BNumLib for uint256; 
 
    uint256 constant BONE = 10 ** 18;

    IBPool immutable pool;

    constructor(address pool_) {
        pool =  IBPool(pool_);
    }

// we are calculating the price as a fraction of the amount we'll get out
    function price(
        bytes32,
        address sellToken,
        address buyToken,
    ) 
    if (specifiedAmount == 0) {
          revert "Specified amount cannot be zero!";
    }
    //scale the value by BONE (1e18)
    specifiedAmount = specifiedAmount.bmul(BONE);

    uint256 tokenBalanceIn = IERC20(sellToken).balanceOf(address(pool));
    uint256 tokenWeightIn = pool.getDenormalizedWeight(sellToken);

    uint256 tokenBalanceOut = IERC20(buyToken).balanceOf(address(pool));
    uint256 tokenWeightOut = pool.getDenormalizedWeight(buyToken);

    uint256 newTokenBalanceIn = tokenBalanceIn.badd(specifiedAmount);
    uint256 newTokenBalanceOut =  tokenBalanceOut.bsub(specifiedAmount);

    uint256 epsilon = 1e6; //small amount

    uint256 amountOut = calcOutGivenIn(
                    newTokenBalanceIn,
                    tokenWeightIn,
                    newTokenBalanceOut,
                    tokenWeightOut,
                    amountOut, 
                    0    
    ); 

    return Fraction(marginalOut, epsilon); 
}

    /// @notice Calculates pool prices for specified amounts
    /// @param specifiedAmount The amount of the token being sold.
    /// @param sellToken The address of the token being sold.
    /// @param buyToken The address of the token being bought 
    /// @return The price as a fraction corresponding to the provided amount.
   function getPriceAt(
        uint256 specifiedAmount,
        address sellToken,
        address buyToken
  ) public view returns (Fraction memory) {
      external 
      view 
      override 
    returns (Fraction[] memory calculatedPrices) {
        uint256[] memory specifiedAmounts
        calculatedPrices = new Fraction[](specifiedAmounts.length);
        for (uint256 i = 0; i < specifiedAmounts.length; i++) {
            calculatedPrices[i] = getPriceAt(specifiedAmounts[i], sellToken, buyToken);
        }
    }
/** @dev Computes how many tokens can be taken out of a pool if `tokenAmountIn` are sent, given the current balances and
     * price bounds. */

    function calcOutGivenIn(
        uint256 tokenBalanceIn,
        uint256 tokenWeightIn,
        uint256 tokenBalanceOut,
        uint256 tokenWeightOut,
        uint256 tokenAmountIn,
        uint256 swapFee
    ) internal pure returns (uint256 tokenAmountOut) {
        uint256 weightRatio = tokenWeightIn.bdiv(tokenWeightOut);
        uint256 adjustedIn = BONE.bsub(swapFee);
        adjustedIn = tokenAmountIn.bmul(adjustedIn);
        uint256 y = tokenBalanceIn.bdiv(tokenBalanceIn.badd(adjustedIn));
        uint256 foo = y.bpow(weightRatio);
        uint256 bar = BONE.bsub(foo);
        tokenAmountOut = tokenBalanceOut.bmul(bar);
        return tokenAmountOut;
    }
/** @dev Computes how many tokens must be sent to a pool in order to take `tokenAmountOut`, given the current balances
     * and price bounds. */

    function calcInGivenOut(
        uint256 tokenBalanceIn,
        uint256 tokenWeightIn,
        uint256 tokenBalanceOut,
        uint256 tokenWeightOut,
        uint256 tokenAmountOut,
        uint256 swapFee
    ) internal pure returns (uint256 tokenAmountIn) {
        uint256 weightRatio = tokenWeightOut.bdiv(tokenWeightIn);
        uint256 diff = tokenBalanceOut.bsub(tokenAmountOut);
        uint256 y = tokenBalanceOut.bdiv(diff);
        uint256 foo = y.bpow(weightRatio);  
        foo = foo.bsub(BONE); 
        tokenAmountIn = BONE.bsub(swapFee);
        tokenAmountIn = (tokenBalanceIn.bmul(foo)).bdiv(tokenAmountIn);
        return tokenAmountIn; 
    }
 
    //
    
    function swap(
        bytes32,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external returns (Trade memory trade) {
    require(sellToken != buyToken, "Tokens must be different");
    require(specifiedAmount != 0);

    uint256 gasBefore = gasleft();
    if (sellToken != address(pool) && buyToken != address(pool)) {
        // Standard Token-to-Token Swap
        if (side == OrderSide.Sell) {
            trade.calculatedAmount = sell(sellToken, buyToken, specifiedAmount);
            trade.price = getPriceAt(specifiedAmount, sellToken, buyToken);
        } else {
            uint256 amountIn = buy(sellToken, buyToken, specifiedAmount);
            trade.calculatedAmount = amountIn;
            trade.price = getPriceAt(trade.calculatedAmount, sellToken, buyToken);
        }
    } 
    
    else if (sellToken == address(pool) && buyToken != address(pool)) {
        // Exiting Pool (LP token is being sold)
        require(side == OrderSide.Sell, "Exiting pool must be OrderSide.Sell");
        uint256 totalSupply = pool.totalSupply();
        /**
        We have to get the proportion of the pools total supply the amount of LP tokens we want to buy is 
                              totalSupply 
         share_proportion =  ---------------
                              specifiedAmount 
        **/
        uint256 SHARE_PROPORTION = ((totalSupply.bdiv(specifiedAmount)));

        address[] memory tokens = pool.getFinalTokens();

        //get the other token in the pool
        address secondaryToken = tokens[0] == buyToken ? tokens[1] : tokens[0];
        
        uint256 token0Balance = IERC20(tokens[0]).balanceOf(address(pool));
        uint256 token1Balance = IERC20(tokens[1]).balanceOf(address(pool));
        /**
          we have to make sure that both expectedTokenOut satisfies the limit constraints 
          for the pool so that when we're selling the superfluous token into the same pool, 
          it doesnt throw an BPool_TokenAmountInAboveMaxRatio() error because (more than 50%
          of the pools current balance is trying to be sold into it)
          getLimits is not visible for some reason
        **/

        uint256 limit0 = token0Balance * MAX_IN_FACTOR / 100;
        uint256 limit1 = token1Balance * MAX_IN_FACTOR / 100;

        /**
            The minimum amount of each token we'll receive is gotten by calculating the
            equivalent proportion of each token balance by dividing it by the SHARE_PROPORTION

            When burning n pool shares, caller expects enough amount X of every token t
            should be sent to satisfy:
            Xt = n/BPT.totalSupply() * t.balanceOf(BPT)
        **/
        uint256 expectedToken0Out = token0Balance.bdiv(SHARE_PROPORTION);
        uint256 expectedToken1Out = token1Balance.bdiv(SHARE_PROPORTION);  //this 
        
        if (expectedToken0Out > limit0) {
          revert("The amount of expectedToken0Out surpasses the limits for the amount that can be swapped into the pool");
        }

        if (expectedToken1Out > limit1) {
          revert("The amount of expectedToken1Out surpasses the limits for the amount that can be swapped into the pool");
        }

        uint256[] memory minAmountsOut = new uint256[](2);
        minAmountsOut[0] = expectedToken0Out.bsub(expectedToken0Out.bmul(1e15));
        minAmountsOut[1] = expectedToken1Out.bsub(expectedToken1Out.bmul(1e15));
        pool.exitPool(specifiedAmount, minAmountsOut); 
        uint256 amountToSell = tokens[0] == buyToken ? expectedToken1Out : expectedToken0Out;
        trade.calculatedAmount = sell(secondaryToken, buyToken, amountToSell);
        trade.price = getPriceAt(amountToSell, secondaryToken, buyToken);
    } 
    
    else if (sellToken != address(pool) && buyToken == address(pool)) {
        // Joining Pool (LP token is being bought)
        require(side == OrderSide.Buy, "Joining pool must be OrderSide.Buy");
        uint256 totalSupply = pool.totalSupply();
        uint256 SHARE_PROPORTION = ((totalSupply.bdiv(specifiedAmount)));
        address[] memory tokens = pool.getFinalTokens();
        //get the other token in the pool
        address secondaryToken = tokens[0] == sellToken ? tokens[1] : tokens[0];

        uint256 token0Balance = IERC20(tokens[0]).balanceOf(address(pool));
        uint256 token1Balance = IERC20(tokens[1]).balanceOf(address(pool));

        uint256 limit0 = token0Balance * MAX_IN_FACTOR / 100;
        uint256 limit1 = token1Balance * MAX_IN_FACTOR / 100;

        // when minting n pool shares, enough amount X of every token t should be provided to statisfy
        // Xt = n/BPT.totalSupply() * t.balanceOf(BPT)
        uint256 requiredToken0Out = token0Balance.bdiv(SHARE_PROPORTION);
        uint256 requiredToken1Out = token1Balance.bdiv(SHARE_PROPORTION);

        uint256[] memory maxAmountsIn = new uint256[](2);
        //hacky fix
        //the tokenAmountIn calculation internally can be slightly higher than the 
        //maxAmountIn and throw a BPool_TokenAmountInAboveMaxAmountIn() error, 
        //so lets add 0.1% of each requiredTokenOut to both of them 

        //0.1% = 1 / 100 = 0.001 * 1e18 = 1e15
        requiredToken0Out = requiredToken0Out.badd(requiredToken0Out.bmul(1e15));
        requiredToken1Out = requiredToken1Out.badd(requiredToken1Out.bmul(1e15));

        if (requiredToken0Out > limit0) {
          revert("The amount of requiredToken0In surpasses the limits for the amount that can be swapped into the pool");
        }

        if (requiredToken1Out > limit1) {
          revert("The amount of requiredToken1In surpasses the limits for the amount that can be swapped into the pool");
        }

        maxAmountsIn[0] = requiredToken0Out;
        maxAmountsIn[1] = requiredToken1Out;

        //approve spending the tokens to send (join) them to the pool
        IERC20(tokens[0]).approve(address(pool), maxAmountsIn[0]);
        IERC20(tokens[1]).approve(address(pool), maxAmountsIn[1]);

        pool.joinPool(specifiedAmount, maxAmountsIn);
        uint256 amountToSell = tokens[0] == buyToken ? requiredToken0Out : requiredToken1Out;
        trade.calculatedAmount = buy(secondaryToken, sellToken, amountToSell);
        trade.price = getPriceAt(trade.calculatedAmount, secondaryToken, sellToken);
    } 
    
    else if (sellToken == address(pool) && buyToken == address(pool)) {
        // Invalid: Swapping LP token to LP token is not supported
        revert("Cannot swap between LP tokens"); 
    } 
    
    else {
        // Should never reach here
        revert("Invalid token and side combination");
    }
    trade.gasUsed = gasBefore - gasleft();
 }
       function getLimits(bytes32, address sellToken, address buyToken)
        external
        returns (uint256[] memory limits)
    {     
        uint256 sellTokenBal = pool.getBalance(sellToken);
        uint256 buyTokenBal = pool.getBalance(buyToken);
        limits = new uint256[](2);
        limits[0] = sellTokenBal * MAX_IN_FACTOR / 100;
        limits[1] = buyTokenBal * MAX_OUT_FACTOR / 100;
    }

    function getCapabilities(
        bytes32,
        address,
        address
    ) external returns (Capability[] memory capabilities) {
        capabilities = new Capability[](4);
        capabilities[0] = Capability.SellOrder;
        capabilities[1] = Capability.BuyOrder;
        capabilities[2] = Capability.PriceFunction;
        capabilities[3] = Capability.HardLimits;
    }

    function getTokens( 
        bytes32
    )
        external
        view
        returns (address[] memory tokens)
    {   
        address[] memory finalTokens = pool.getFinalTokens();

        tokens = new address[](3);
        tokens[0] = finalTokens[0];
        tokens[1] = finalTokens[1];
        tokens[2] = address(pool)
        tokens
    }

    function getPoolIds(uint256 offset, uint256 limit)
        external
        returns (bytes32[] memory ids)
    {
        revert NotImplemented("CowAMMSwapAdapter.getPoolIds");
    }

    /// @notice Executes a sell order on the contract.
    /// @param sellToken The token being sold.
    /// @param buyToken The token being bought.
    /// @param amount The amount to be sold.
    /// @return calculatedAmount The amount of tokens received.
    function sell(address sellToken, address buyToken, uint256 amount)
        internal
        returns (uint256 calculatedAmount)
    {    
        //scale the amount that we want by BONE so that very small values eg; amount < 5000 do not cause
        //zero division error in the bnum methods 
        amount = amount.bmul(BONE);

        uint256 tokenInBalance = IERC20(sellToken).balanceOf(address(pool));
        uint256 tokenInWeight = pool.getDenormalizedWeight(sellToken);

        uint256 tokenOutBalance = IERC20(buyToken).balanceOf(address(pool));
        uint256 tokenOutWeight = pool.getDenormalizedWeight(buyToken);

        if (amount > tokenInBalance.bmul(MAX_IN_FACTOR.bdiv(10))) {
          revert IBPool.BPool_TokenAmountInAboveMaxRatio();
        } 

        uint256 tokenAmountOut = calcOutGivenIn(
                    tokenInBalance,
                    tokenInWeight,
                    tokenOutBalance,
                    tokenOutWeight,
                    amount,
                    0
        );
        calculatedAmount = tokenAmountOut;
    }
    /// @notice Executes a buy order on the contract.
    /// @param sellToken The token being sold.
    /// @param buyToken The token being bought.
    /// @param amountOut The amount of buyTokens to buy. 
    /// @return calculatedAmount The amount of tokens received.
    function buy(address sellToken, address buyToken, uint256 amountOut)
        internal
        returns (uint256 calculatedAmount)
    {   
        //scale the amountOut that we want by BONE so that very small values eg; amountOut < 5000 do not cause
        //internal zero division error in the bnum methods 
        amountOut = amountOut.bmul(BONE); 

        uint256 tokenInBalance = IERC20(sellToken).balanceOf(address(pool));
        uint256 tokenInWeight = pool.getDenormalizedWeight(sellToken);

        uint256 tokenOutBalance = IERC20(buyToken).balanceOf(address(pool));
        uint256 tokenOutWeight = pool.getDenormalizedWeight(buyToken);

        if (amountOut > tokenOutBalance.bmul(MAX_OUT_FACTOR.bdiv(100))) {
          revert IBPool.BPool_TokenAmountOutAboveMaxOut();
        } 

        uint256 tokenAmountIn = calcInGivenOut(
                    tokenInBalance,
                    tokenInWeight,
                    tokenOutBalance,
                    tokenOutWeight, 
                    amountOut,  
                    0
        ); 
    
        calculatedAmount = tokenAmountIn;
    }
}

