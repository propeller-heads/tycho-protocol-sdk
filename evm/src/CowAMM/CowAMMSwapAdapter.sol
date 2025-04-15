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
  function calcSpotPriceConstraints(
    uint256 tokenAmountIn,
    uint256 tokenAmountOut,
    uint256 tokenInBalance,
    uint256 tokenInWeight,
    uint256 tokenOutBalance,
    uint256 tokenOutWeight,
    uint256 maxPrice
  ) internal {
      uint256 spotPriceBefore = (tokenInBalance.bdiv(tokenInWeight)).bdiv(tokenOutBalance.bdiv(tokenOutWeight));
      if (spotPriceBefore > maxPrice) {
          revert IBPool.BPool_SpotPriceAboveMaxPrice();
      }
      //everything above is before calcInGivenOut but problem is getting maxPrice depends on calcOut , which comes after so its sort like a circular dep, this version is okay tbh

      tokenInBalance = tokenInBalance.badd(tokenAmountIn);
      tokenOutBalance = tokenOutBalance.bsub(tokenAmountOut);

      //  We are simulating with zero fees
      uint256 spotPriceAfter = (tokenInBalance.bdiv(tokenInWeight)).bdiv(tokenOutBalance.bdiv(tokenOutWeight)); 
      if (spotPriceAfter < spotPriceBefore) {
        revert IBPool.BPool_SpotPriceAfterBelowSpotPriceBefore();
      }
      if (spotPriceAfter > maxPrice) {
        revert IBPool.BPool_SpotPriceAboveMaxPrice();
      }
      console2.log("spotpricebefore:",spotPriceBefore);
      console2.log("tokenAmountIn:", tokenAmountIn);
      console2.log("tokenAmountOut:", tokenAmountOut);
      console2.log("tokenAmountIn / tokenAmountOut:", tokenAmountIn.bdiv(tokenAmountOut));
      if (spotPriceBefore > tokenAmountIn.bdiv(tokenAmountOut)) {
        revert IBPool.BPool_SpotPriceBeforeAboveTokenRatio();
      }
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
    // sellToken OrderSide.sell exitPool
    // buyToken OrderSide.sell exitPool
    // sellToken OrderSide.buy joinPool 
    // buyToken OrderSide.buy joinPool

    // we have to check if the input are LP Tokens, how do we do that 
    uint256 gasBefore = gasleft();
    if (side == OrderSide.Sell) {
            // Standard Token Swap
        if (sellToken != address(pool) && buyToken != address(pool)) {
            uint256 amountOut = sell(sellToken, buyToken, specifiedAmount);
            trade.calculatedAmount = amountOut;
            trade.price = getPriceAt(specifiedAmount, sellToken, buyToken); // Example price calculation redoes the getBalance calculations all over
        } 
          //LP Providing (Join Pool)
        else {
          // BCoW50COW50wstETH is sellToken, wstETH is buy Token
          if (sellToken == address(pool)) { 
            uint256[] memory maxAmountsIn = new uint256[](2);
            maxAmountsIn[0] = specifiedAmount;
            maxAmountsIn[1] = specifiedAmount;
            pool.exitPool(specifiedAmount, maxAmountsIn);
            address[] memory tokens = pool.getCurrentTokens();
            address newSellToken;
            // get the other token in the pool and sell the amount for buyToken eg;
            if (tokens[0] != buyToken) {
              newSellToken = tokens[1];
            } else {
              newSellToken = tokens[0];
            }
            // not using swap here we just need to estimate the output so, don't want to recurse
            uint256 extraTokenAmount = sell(newSellToken, buyToken, specifiedAmount);
            // the amount of token redeemed + the superfluous token swapped 
            trade.calculatedAmount = specifiedAmount + extraTokenAmount; 
            trade.price = Fraction(0, 1); // No direct price change
          } else {
            uint256[] memory maxAmountsIn = new uint256[](2);
            maxAmountsIn[0] = specifiedAmount;
            maxAmountsIn[1] = specifiedAmount;
            pool.exitPool(specifiedAmount, maxAmountsIn); //amountOut
            uint256 extraTokenAmount = sell(sellToken, buyToken, specifiedAmount);
            trade.calculatedAmount = specifiedAmount + extraTokenAmount; // Assuming 1:1 LP deposit
            trade.price = Fraction(0, 1); // No direct price change
          }
        }
    } 
    else {
        if (sellToken != address(pool) && buyToken != address(pool)) {
            // Buy Side Swap
            uint256 amountOut = buy(sellToken, buyToken, specifiedAmount);
            trade.calculatedAmount = amountOut;
            trade.price = getPriceAt(trade.calculatedAmount, sellToken, buyToken); 
        } 
        else { 
          if (buyToken == address(pool)) {
             // Single sided LP Withdrawal (Burn) (Exit Pool)
            uint256[] memory maxAmountsIn = new uint256[](2);
            maxAmountsIn[0] = specifiedAmount;
            maxAmountsIn[1] = specifiedAmount;
            pool.joinPool(specifiedAmount, maxAmountsIn);
            address[] memory tokens = pool.getCurrentTokens();
            address newBuyToken;
            // get the other token in the pool and sell the amount for buyToken eg;
            if (tokens[0] != sellToken) {
              newBuyToken = tokens[1];
            } else {
              newBuyToken = tokens[0];
            }
            uint256 extraTokenAmount = buy(sellToken, newBuyToken, specifiedAmount);
            trade.calculatedAmount = specifiedAmount + extraTokenAmount; // Assuming 1:1 LP withdrawal
            trade.price = Fraction(0, 1);
          }  else {
            // pool.joinPool(specifiedAmount, [ ,specifiedAmount]); //amountOut
            trade.calculatedAmount = specifiedAmount; // Assuming 1:1 LP deposit
            trade.price = Fraction(0, 1); // No direct price change
          }
        }
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
        tokens = pool.getCurrentTokens();
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
        uint256 tokenInBalanceFinal = tokenInBalance.badd(amount);
        uint256 tokenOutBalanceFinal = tokenOutBalance.bsub(tokenAmountOut);
        // maxPrice is just the spotPriceAfterSwap 
        uint256 spotPriceAfterSwapWithoutFee = (tokenInBalanceFinal.bdiv(tokenInWeight)).bdiv(tokenOutBalanceFinal.bdiv(tokenOutWeight));
        //calculates the constraints for the spotPrice 
        calcSpotPriceConstraints(
          amount,
          tokenAmountOut,
          tokenInBalance,
          tokenInWeight,
          tokenOutBalance, 
          tokenOutWeight,
          spotPriceAfterSwapWithoutFee
        );

        calculatedAmount = tokenAmountOut;
    }
    /// @notice Executes a buy order on the contract.
    /// @param sellToken The token being sold.
    /// @param buyToken The token being bought.
    /// @param amountOut The amount of buyToken to receive.
    /// @return calculatedAmount The amount of tokens received.
    function buy(address sellToken, address buyToken, uint256 amountOut)
        internal
        returns (uint256 calculatedAmount)
    {
      
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
    
        uint256 tokenInBalanceFinal = tokenInBalance.badd(tokenAmountIn);
        uint256 tokenOutBalanceFinal = tokenOutBalance.bsub(amountOut);
        // maxPrice is just the spotPriceAfterSwap 
        uint256 spotPriceAfterSwapWithoutFee = (tokenInBalanceFinal.bdiv(tokenInWeight)).bdiv(tokenOutBalanceFinal.bdiv(tokenOutWeight));

        // calcSpotPriceConstraints( 
        //   tokenAmountIn,
        //   amountOut,
        //   tokenInBalance,
        //   tokenInWeight,
        //   tokenOutBalance,
        //   tokenOutWeight,
        //   spotPriceAfterSwapWithoutFee
        // );
        calculatedAmount = tokenAmountIn;
    }
}

