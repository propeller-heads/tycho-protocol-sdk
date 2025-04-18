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

uint256 constant MAX_IN_FACTOR = 5;
uint256 constant MAX_OUT_FACTOR = 33;

contract CowAMMSwapAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;
    using BNumLib for uint256; 
 
    uint256 constant BONE = 10 ** 18;

    IBPool immutable pool;

    constructor(address pool_) {
        pool =  IBPool(pool_);
    }

    function price(
        bytes32,
        address sellToken,
        address buyToken,
        uint256[] memory specifiedAmounts
    ) 
      external 
      view 
      override 
    returns (Fraction[] memory calculatedPrices) {
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
        console2.log("weightratio", weightRatio); //1e18 
        console2.log("this is the token balance out", tokenBalanceOut); 
        console2.log("this is the token amount out", tokenAmountOut); //why is token amount out 1.544e21? instead of 
        uint256 diff = tokenBalanceOut.bsub(tokenAmountOut);
        console2.log("this is the diff", diff);
        uint256 y = tokenBalanceOut.bdiv(diff);
        console2.log("this is the base y", y);  
        uint256 foo = y.bpow(weightRatio);  
        console2.log("this is foo 1", foo);
        foo = foo.bsub(BONE); //1
        console2.log("this is foo 2", foo);
        tokenAmountIn = BONE.bsub(swapFee);
        console2.log("this is tokenAmountIn 1", tokenAmountIn);
        tokenAmountIn = (tokenBalanceIn.bmul(foo)).bdiv(tokenAmountIn);// 1e17 . 1 / 1e18 
        console2.log("this is tokenAmountIn 2", tokenAmountIn);
        return tokenAmountIn; 
    }
     function calcSpotPrice(
      uint256 tokenInBalance,
      uint256 tokenInWeight,
      uint256 tokenOutBalance,
      uint256 tokenOutWeight
    ) internal pure returns (uint256 spotPrice) {
       spotPrice = tokenInBalance.bdiv(tokenInWeight).bdiv(
             tokenOutBalance.bdiv(tokenOutWeight)
       ); 
    }
    //we are calculating the price as a fraction of the amount we'll get out
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
    if (specifiedAmount == 0) {
          revert Unavailable("Specified amount cannot be zero!");
    }
    uint256 tokenBalanceIn = IERC20(sellToken).balanceOf(address(pool));
    uint256 tokenWeightIn = pool.getDenormalizedWeight(sellToken);

    uint256 tokenBalanceOut = IERC20(buyToken).balanceOf(address(pool));
    uint256 tokenWeightOut = pool.getDenormalizedWeight(buyToken);

    uint256 amountOut = calcOutGivenIn(
                    tokenBalanceIn,
                    tokenWeightIn,
                    tokenBalanceOut,
                    tokenWeightOut,
                    specifiedAmount,
                    0
    );
    uint256 amountIn = calcInGivenOut(
                    tokenBalanceIn,
                    tokenWeightIn,
                    tokenBalanceOut,
                    tokenWeightOut,
                    amountOut, 
                    0    
    ); 
  console2.log("this is amount Out : ", amountOut);
  console2.log("this is amount in : ", amountIn);
    return Fraction(amountOut, amountIn); 
}
 
    function swap(
        bytes32,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external returns (Trade memory trade) {
    require(sellToken != buyToken, "Tokens must be different");
    require(specifiedAmount != 0);

    if (specifiedAmount == 0) {
          return trade;
    }

    // specifiedAmount = specifiedAmount.bmul(BONE);

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
        //We have to get the proportion of the pools total supply the amount of LP tokens we want to sell is 
        uint256 totalSupply = pool.totalSupply();
        /**
          What percentage of totalSupply is specifiedAmount? 
          We have to get the proportion of the pools total supply the amount of LP tokens we want to buy is 
          since the percentage is scaled by BONE (1e18), to get this we'll have to divide it by BONE.

          The percentage also gets approximated, in an example of (5.983e19 / 1.99e20 ) * 100, its actually 29.8% using normal arithmetic but is 30% using BONE math 
        **/
        // uint256 percentage = ((specifiedAmount.bdiv(totalSupply)).bmul(100)).bdiv(BONE);
        // // We want the share proportion, which is less than 100, to get that we divide it by the percentage we get 
        // uint256 SHARE_PROPORTION = uint256(100).bdiv(percentage); 
        uint256 SHARE_PROPORTION = ((totalSupply.bdiv(specifiedAmount)));
        console2.log("this is the pools share proportion", SHARE_PROPORTION);
        /**
                        totalSupply 
         percentage =  ---------------
                        specifiedAmount 

         Share Proportion = 100 / percentage 

         or go straight and say Share Proportion = totalSupply / specifiedAmount
        **/
        address[] memory tokens = pool.getCurrentTokens();

        //get the other token in the pool
        address secondaryToken = tokens[0] == buyToken ? tokens[1] : tokens[0];
        
        uint256 token0Balance = IERC20(tokens[0]).balanceOf(address(pool));
        uint256 token1Balance = IERC20(tokens[1]).balanceOf(address(pool));
        /**
         The minimum amount of each token we'll receive is gotten by calculating the
         equivalent proportion of each token balance by dividing it by the SHARE_PROPORTION

         When burning n pool shares, caller expects enough amount X of every token t
         should be sent to satisfy:
         Xt = n/BPT.totalSupply() * t.balanceOf(BPT)
         **/
        uint256 expectedToken0Out = token0Balance.bdiv(SHARE_PROPORTION);
        uint256 expectedToken1Out = token1Balance.bdiv(SHARE_PROPORTION);  //this 

        console2.log("this is the expectedToken0Out", expectedToken0Out);
        console2.log("this is the expectedToken1Out", expectedToken1Out);

        uint256[] memory minAmountsOut = new uint256[](2);
        minAmountsOut[0] = expectedToken0Out;
        minAmountsOut[1] = expectedToken1Out;
        pool.exitPool(specifiedAmount, minAmountsOut); //COW to wstETH
        //amountIn is [5.983e19], minAmountsOut are [4.641e20, 3e16]
        //balance of token a is [1.547e21] 
        //balance of token b is [1e17]  -> 7.008e16 + 2.991e16 = 9.999e16 
        //the balance of wstETH in the pool is now 7.008e16 , now we want to sell 2.991e16 back into the pool for COW
        //2.991e16 is now 42% of the new pool balance - 7.008e16 
        //swapped the extra token in the pool to buyToken, that is the token we wanted to burn the LPtokens for 
        //we obviously want to swap the amount of the other token that we got so thats what we want 
        //The problem now is, if we successfully 
        //from the lp tokens, we get the expected amount out and they each must be < that the limits for the current balance 
        //so that when we want to swap the superfluous token it'll be possible and not cause issues 
        // it has to be both of them that has the limits because they are in the same proportion
        uint256 amountToSell = tokens[0] == buyToken ? expectedToken1Out : expectedToken0Out;
        console2.log("amount To sell", amountToSell); // sell 3e16 wsteth correct
        console2.log("secondaryToken", secondaryToken); // should be wsteth
        console2.log("buyToken",buyToken); // should be COW
        
        uint256 swappedAmount = sell(secondaryToken, buyToken, amountToSell);
        trade.calculatedAmount = specifiedAmount + swappedAmount;
        trade.price = getPriceAt(amountToSell, secondaryToken, buyToken);
    } 
    
    else if (sellToken != address(pool) && buyToken == address(pool)) {
        // Joining Pool (LP token is being bought)
        require(side == OrderSide.Buy, "Joining pool must be OrderSide.Buy");
        uint256 totalSupply = pool.totalSupply();
        uint256 SHARE_PROPORTION = ((totalSupply.bdiv(specifiedAmount)));
        console2.log("this is the pools share proportion", SHARE_PROPORTION);
        address[] memory tokens = pool.getCurrentTokens();

        //get the other token in the pool
        address secondaryToken = tokens[0] == sellToken ? tokens[1] : tokens[0];

        uint256 token0Balance = IERC20(tokens[0]).balanceOf(address(pool));
        uint256 token1Balance = IERC20(tokens[1]).balanceOf(address(pool));
        // when minting n pool shares, enough amount X of every token t should be provided to statisfy
        // Xt = n/BPT.totalSupply() * t.balanceOf(BPT)
        uint256 requiredToken0Out = token0Balance.bdiv(SHARE_PROPORTION);
        uint256 requiredToken1Out = token1Balance.bdiv(SHARE_PROPORTION);

        uint256[] memory maxAmountsIn = new uint256[](2);
        maxAmountsIn[0] = requiredToken0Out;
        maxAmountsIn[1] = requiredToken1Out;

        pool.joinPool(specifiedAmount, maxAmountsIn);
        uint256 swappedAmount = sell(secondaryToken, sellToken, specifiedAmount);
        trade.calculatedAmount = specifiedAmount + swappedAmount;
        trade.price = getPriceAt(specifiedAmount, sellToken, buyToken);
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
        limits[0] = sellTokenBal * MAX_IN_FACTOR / 10;
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
        tokens = pool.getFinalTokens();
    }

    function getPoolIds(uint256 offset, uint256 limit)
        external
        returns (bytes32[] memory ids)
    {
        revert NotImplemented("TemplateSwapAdapter.getPoolIds");
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
        //scale the amountOut that we want by BONE so that very small values do not cause
        //zero division in the bnum methods 
        amount = amount.bmul(BONE);
        IERC20(sellToken).safeTransferFrom(msg.sender, address(this), amount);
        IERC20(sellToken).approve(address(pool), amount);

        uint256 tokenInBalance = IERC20(sellToken).balanceOf(address(pool));
        uint256 tokenInWeight = pool.getDenormalizedWeight(sellToken);

        uint256 tokenOutBalance = IERC20(buyToken).balanceOf(address(pool));
        uint256 tokenOutWeight = pool.getDenormalizedWeight(buyToken);

        // Our limits cover this case but add just in case 
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
        //scale the amountOut that we want by BONE so that very small values do not cause
        //internal zero division error in the bnum methods 
        amountOut = amountOut.bmul(BONE); 
        uint256 tokenInBalance = IERC20(sellToken).balanceOf(address(pool));
        uint256 tokenInWeight = pool.getDenormalizedWeight(sellToken);

        uint256 tokenOutBalance = IERC20(buyToken).balanceOf(address(pool));
        uint256 tokenOutWeight = pool.getDenormalizedWeight(buyToken);

        if (amountOut > tokenOutBalance.bmul(MAX_OUT_FACTOR.bdiv(100))) {
          revert IBPool.BPool_TokenAmountOutAboveMaxOut();
        } 

        console2.log("This is the tokenBalanceIn from buy", tokenInBalance);
        console2.log("This is the tokenWeightIn from buy", tokenInWeight);
        console2.log("This is the tokenBalanceOut from buy", tokenOutBalance);
        console2.log("This is the tokenWeightOut from buy", tokenOutWeight);
        console2.log("This is the amountOut from buy", amountOut);
        //maxAmountIn

        uint256 tokenAmountIn = calcInGivenOut(
                    tokenInBalance,
                    tokenInWeight,
                    tokenOutBalance,
                    tokenOutWeight, 
                    amountOut,  
                    0
        ); 
        console2.log("this is the amountin from buy method", tokenAmountIn);
        IERC20(sellToken).safeTransferFrom(
            msg.sender, address(this), tokenAmountIn
        );
        IERC20(sellToken).approve(address(pool), tokenAmountIn);

        console2.log("this is the max amountIn", tokenAmountIn);
    
        calculatedAmount = tokenAmountIn;
    }
}

