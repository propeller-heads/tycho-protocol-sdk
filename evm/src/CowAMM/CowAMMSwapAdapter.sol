// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

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

uint256 constant RESERVE_LIMIT_FACTOR = 3;

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
        // tokens = getTokens(poolId); ..if its the first one that comes lexicographically, we'll just check 
        // it against the < comparison, it might not even matter, we don't know for now, until testing
        //address(bytes20(poolId))
        for (uint256 i = 0; i < specifiedAmounts.length; i++) {
            calculatedPrices[i] = getPriceAt(specifiedAmounts[i], buyToken, sellToken);
        }
    }

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
        foo = BONE.bsub(BONE);
        tokenAmountIn = BONE.bsub(swapFee);
        tokenAmountIn = (tokenBalanceIn.bmul(foo)).bdiv(tokenAmountIn);
        return tokenAmountIn; 
    }

    //we are calculating the price as a fraction of the amount we'll get out
    /// @notice Calculates pool prices for specified amounts
    /// @param specifiedAmount The amount of the token being sold.
    /// @param sellToken The address of the token being sold.
    /// @param buyToken The address of the token being bought n
    /// @return The price as a fraction corresponding to the provided amount.
   function getPriceAt(
        uint256 specifiedAmount,
        address sellToken,
        address buyToken
        // address poolId 
  ) internal view returns (Fraction memory) {
    
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
                    specifiedAmount,
                    0
    );

    return Fraction(amountOut, amountIn);
}

//This alternative implementation was inspired by IntegralSwapAdapter, will test it out too
//    function getPriceAt(
//         address sellToken,
//         address buyToken,
//         address poolId
//   ) internal pure returns (Fraction memory) {
    
//     uint256 tokenBalanceIn = IERC20(sellToken).balanceOf(poolId);
//     uint256 tokenWeightIn = pool.getDenormalizedWeight(sellToken);

//     uint256 tokenBalanceOut = IERC20(buyToken).balanceOf(poolId);
//     uint256 tokenWeightOut = pool.getDenormalizedWeight(buyToken);

//      // get token decimals
//     uint256 sellTokenDecimals = 10 ** IERC20Metadata(sellToken).decimals();
//     uint256 buyTokenDecimals = 10 ** IERC20Metadata(buyToken).decimals();

//     uint256 numer = BONE.bdiv(tokenBalanceIn, tokenWeightIn);
//     uint256 denom = BONE.bdiv(tokenBalanceOut, tokenWeightOut);
//     uint256 ratio = BONE.bdiv(numer, denom);
//     uint256 scale = BONE.bdiv(BONE, BONE.bsub(BONE, swapFee));
//     spotPrice = BONE.bmul(ratio, scale);
//      /**
//          * @dev
//          * Denominator works as a "standardizer" for the price rather than a
//          * reserve value
//          * We can calculate CowAMM prices directly 
//          * it is therefore used to maintain integrity for the Fraction division,
//          * as numerator and denominator could have different token decimals(es.
//          * ETH(18)-USDC(6)).
//          * Both numerator and denominator are also multiplied by
//          * STANDARD_TOKEN_DECIMALS
//          * to ensure that precision losses are minimized or null.
//          */
//     return Fraction(
//         spotPrice * STANDARD_TOKEN_DECIMALS,
//         STANDARD_TOKEN_DECIMALS * sellTokenDecimals * 1 // no swap fee so i just put 1
//                 / buyTokenDecimals
//     );
// }
// or using pool.getSpotPrice(tokenIn, tokenOut); directly and do the fraction above
    function swap(
        bytes32,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external returns (Trade memory trade) {
    require(sellToken != buyToken, "Tokens must be different");
    if (specifiedAmount == 0) {
          return trade;
    }

    // address poolAddress = address(bytes20(poolId));
    uint256 gasBefore = gasleft();
    if (side == OrderSide.Sell) {
            // Standard Token Swap
        if (sellToken != address(0) && buyToken != address(0)) {
            uint256 amountOut = sell(sellToken, buyToken, specifiedAmount);
            trade.calculatedAmount = amountOut;
            trade.price = getPriceAt(specifiedAmount,sellToken,buyToken); // Example price calculation
        } 
            // Single Sided LP Providing (Join Pool)
        else {
          if (sellToken != address(0) && buyToken == address(0)) {
            // tokens = getTokens(_poolId);
            //if (first token in tokens is token we want to join then pass amount to left)
            // or maybe we can just use that lexicographic sorting 
            //how can we tell which one (token in the pool) comes first ? 
            // pool.joinPool(specifiedAmount, [0, 0]); //amountOut
            trade.calculatedAmount = specifiedAmount; // Assuming 1:1 LP deposit
            trade.price = Fraction(0, 1); // No direct price change
          } 
        }
    } 
    else {
        if (sellToken == address(0) && buyToken != address(0)) {
            // Single sided LP Withdrawal (Burn) (Exit Pool)
            // pool.exitPool(specifiedAmount, [type(uint256).max,0]);
            trade.calculatedAmount = specifiedAmount; // Assuming 1:1 LP withdrawal
            trade.price = Fraction(0, 1);
        } 
        else { 
          if (sellToken != address(0) && buyToken != address(0)) {
            // Buy Side Swap
            uint256 amountOut = buy(sellToken, buyToken, specifiedAmount);
            trade.calculatedAmount = amountOut;
            trade.price = getPriceAt(specifiedAmount,sellToken,buyToken);
          } 
        }
    } 
 }

    function getLimits(bytes32, address sellToken, address buyToken)
        external
        returns (uint256[] memory limits)
    { 
    // theres no explicit limit set for CowAMM so will use the limit for balancer

        uint256 sellTokenBal = pool.getBalance(sellToken);
    
        uint256 buyTokenBal = pool.getBalance(buyToken);

        if (sellTokenBal > buyTokenBal) {
            limits[0] = sellTokenBal * RESERVE_LIMIT_FACTOR / 10;
            limits[1] = buyTokenBal *  RESERVE_LIMIT_FACTOR / 10;
        } else {
            limits[0] = buyTokenBal *  RESERVE_LIMIT_FACTOR / 10;
            limits[1] = sellTokenBal * RESERVE_LIMIT_FACTOR / 10;
        }
    }
    function getCapabilities(
        bytes32 poolId,
        address sellToken,
        address buyToken
    ) external returns (Capability[] memory capabilities) {
        capabilities = new Capability[](4);
        capabilities[0] = Capability.SellOrder;
        capabilities[1] = Capability.BuyOrder;
        capabilities[2] = Capability.PriceFunction;
        capabilities[3] = Capability.HardLimits;
    }

    function getTokens(bytes32 _poolId)
        external
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
        IERC20(sellToken).safeTransferFrom(msg.sender, address(this), amount);
        IERC20(sellToken).approve(address(pool), amount);

        uint256 tokenAmountIn;

        (tokenAmountIn,) = pool.swapExactAmountIn(
            sellToken, amount, buyToken, 0, type(uint256).max
        );
        calculatedAmount = tokenAmountIn;
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
        IERC20(sellToken).safeTransferFrom(
            msg.sender, address(this), calculatedAmount
        );
        IERC20(buyToken).approve(address(pool), amountOut);
        uint256 tokenAmountOut;

        (tokenAmountOut,) = pool.swapExactAmountOut(
            sellToken, 0, buyToken, amountOut, type(uint256).max
        );
        calculatedAmount = tokenAmountOut;
    }
}
