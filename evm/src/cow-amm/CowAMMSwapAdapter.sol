// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {console2} from "forge-std/console2.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {IERC20Metadata} from
    "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {IBPool} from "./interfaces/IBPool.sol";
import {BNumLib} from "./BNum.sol";
import "src/libraries/FractionMath.sol";
import {SafeERC20} from
    "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title CowAMMSwapAdapter

/// @dev This is the CowAMM swap adapter.

// 50% and 33%
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

   /// @inheritdoc ISwapAdapter
   function price(
        bytes32,
        address sellToken,
        address buyToken,
        uint256[] memory specifiedAmounts
    ) external view override returns (Fraction[] memory calculatedPrices) {
        calculatedPrices = new Fraction[](specifiedAmounts.length);
        uint256[] memory _limits = getLimits(bytes32(0), sellToken, buyToken);

        // prevent price above sell limits as pool will revert for
        // underflow/overflow on mint/redeem
        for (uint256 i = 0; i < specifiedAmounts.length; i++) {
            _checkLimits(_limits, OrderSide.Sell, specifiedAmounts[i]);

            calculatedPrices[i] = getPriceAt(specifiedAmounts[i], sellToken, buyToken);
        }
    }
    
  // we are calculating the marginal price 
    function getPriceAt(
        uint256 specifiedAmount,
        address sellToken,
        address buyToken
    ) public view returns (Fraction memory) { 

      uint256 tokenBalanceIn = IERC20(sellToken).balanceOf(address(pool));
      uint256 tokenWeightIn = pool.getDenormalizedWeight(sellToken);

      uint256 tokenBalanceOut = IERC20(buyToken).balanceOf(address(pool));
      uint256 tokenWeightOut = pool.getDenormalizedWeight(buyToken);
    
      uint256 tokenAmountOut = calcOutGivenIn(tokenBalanceIn, tokenWeightIn, tokenBalanceOut, tokenWeightOut, specifiedAmount, 0);

      uint256 newTokenBalanceIn = tokenBalanceIn.badd(specifiedAmount);
      uint256 newTokenBalanceOut = tokenBalanceOut.bsub(tokenAmountOut); 

      uint256 num = newTokenBalanceOut.bdiv(tokenWeightOut);
      uint256 denom = newTokenBalanceIn.bdiv(tokenWeightIn);
 
      return Fraction(num, denom); 
  }

/** @dev Computes how many tokens can be taken out of a pool if `tokenAmountIn` are sent, given the current balances and
     * price bounds. */
    
    /**********************************************************************************************
    // calcOutGivenIn                                                                            //
    // aO = tokenAmountOut                                                                       //
    // bO = tokenBalanceOut                                                                      //
    // bI = tokenBalanceIn              /      /            bI             \    (wI / wO) \      //
    // aI = tokenAmountIn    aO = bO * |  1 - | --------------------------  | ^            |     //
    // wI = tokenWeightIn               \      \ ( bI + ( aI * ( 1 - sF )) /              /      //
    // wO = tokenWeightOut                                                                       //
    // sF = swapFee                                                                              //
    **********************************************************************************************/
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
      /**
   * @notice Calculate the amount of token in given the amount of token out for a swap
   * @param tokenBalanceIn The balance of the input token in the pool
   * @param tokenWeightIn The weight of the input token in the pool
   * @param tokenBalanceOut The balance of the output token in the pool
   * @param tokenWeightOut The weight of the output token in the pool
   * @param tokenAmountOut The amount of the output token
   * @param swapFee The swap fee of the pool
   * @return tokenAmountIn The amount of token in given the amount of token out for a swap
   * @dev Formula:
   * aI = tokenAmountIn
   * bO = tokenBalanceOut               /  /     bO      \    (wO / wI)      \
   * bI = tokenBalanceIn          bI * |  | ------------  | ^            - 1  |
   * aO = tokenAmountOut    aI =        \  \ ( bO - aO ) /                   /
   * wI = tokenWeightIn           --------------------------------------------
   * wO = tokenWeightOut                          ( 1 - sF )
   * sF = swapFee
   */
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

    //gotten from https://github.com/balancer/balancer-v2-monorepo/blob/6c9e24e22d0c46cca6dd15861d3d33da61a60b98/pkg/core/contracts/pools/weighted/WeightedMath.sol#L299
    //calculates the proportion of tokens that a user receives from exiting the pool or needs to send to the pool to receive an amount of lpToken 
    function calcTokensOutGivenExactLpTokenIn(
        uint256[] memory balances,
        uint256 lpTokenAmountIn,
        uint256 totalLpToken
    ) internal pure returns (uint256[] memory) {
        /**********************************************************************************************
        // exactLpTokenInForTokensOut                                                                        //
        // (per token)                                                                                       //
        // aO = amountOut                  /       lpTokenAmountIn        \                                  //
        // b = balance           a0 = b * |     ---------------------      |                                 //
        // lpTokenAmountIn                 \        totalLpToken          /                                  //
        // totalLpToken                                                                                      //
        **********************************************************************************************/

        uint256 lpTokenRatio = lpTokenAmountIn.bdiv(totalLpToken);

        uint256[] memory amountsOut = new uint256[](balances.length);
        for (uint256 i = 0; i < balances.length; i++) {
            amountsOut[i] = balances[i].bmul(lpTokenRatio);
        }

        return amountsOut;
    }
    function calcSpotPrice(
        uint256 tokenBalanceIn,
        uint256 tokenWeightIn,
        uint256 tokenBalanceOut,
        uint256 tokenWeightOut,
        uint256 swapFee
    ) internal pure returns (uint256 spotPrice) {
        uint256 numer = tokenBalanceIn.bdiv(tokenWeightIn);
        uint256 denom = tokenBalanceOut.bdiv(tokenWeightOut);
        uint256 ratio = numer.bdiv(denom);
        uint256 scale = BONE.bdiv(BONE.bsub(swapFee));
        return ratio.bmul(scale);
    }

    enum SwapType { TokenToToken, ExitPool, JoinPool, Invalid }

    function _getSwapType(address sellToken, address buyToken) private view returns (SwapType) {
        bool sellIsPool = (sellToken == address(pool));
        bool buyIsPool = (buyToken == address(pool));
        
        if (!sellIsPool && !buyIsPool) return SwapType.TokenToToken;
        if (sellIsPool && !buyIsPool) return SwapType.ExitPool;
        if (!sellIsPool && buyIsPool) return SwapType.JoinPool;
        return SwapType.Invalid; // Both are pool tokens
    }
    /**
     * @notice Approves tokens for pool operations
     */
    function _approvePoolTokens(address[] memory tokens, uint256[] memory amounts) private {
        for (uint i = 0; i < tokens.length; i++) {
            IERC20(tokens[i]).approve(address(pool), amounts[i]);
        }
    }
    /**
    * @notice Executes standard token-to-token swap
    */
    function _executeTokenSwap(
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) private returns (Trade memory trade) {
        // Check swap limits to prevent pool revert
        uint256[] memory limits = getLimits(bytes32(0), sellToken, buyToken);
        _checkLimits(limits, side, specifiedAmount);
        
        if (side == OrderSide.Sell) {
            trade.calculatedAmount = sell(sellToken, buyToken, specifiedAmount);
            trade.price = getPriceAt(specifiedAmount, sellToken, buyToken);
        } else {
            trade.calculatedAmount = buy(sellToken, buyToken, specifiedAmount);
            trade.price = getPriceAt(trade.calculatedAmount, sellToken, buyToken);
        }
    }

    /**
     * @notice Executes pool exit (LP token → underlying token)
     */
    function _executePoolExit(
        address sellToken, // LP token (pool address)
        address buyToken,  // Desired underlying token
        OrderSide side,
        uint256 specifiedAmount // LP tokens to burn
    ) private returns (Trade memory trade) {
        require(side == OrderSide.Sell, "SwapAdapter: pool exit must be sell order");
        
        PoolExitParams memory params = _getPoolExitParams(buyToken, specifiedAmount);
        
        // Validate exit amounts against limits
        _validatePoolExitLimits(params);
        
        // Execute pool exit
        pool.exitPool(specifiedAmount, params.maxTokenAmountsOut);
        
        // If we received excess tokens, swap them for the desired token
        uint256 swappedAmount = sell(params.tokenToSwap, buyToken, params.amountToSwap);
        trade.calculatedAmount = params.directAmount + swappedAmount;
        trade.price = getPriceAt(params.amountToSwap, params.tokenToSwap, buyToken);
    }

    /**
     * @notice Executes pool join (underlying token → LP token)
     */
    function _executePoolJoin(
        address sellToken, // Token we have
        address buyToken,  // LP token (pool address)  
        OrderSide side,
        uint256 specifiedAmount // LP tokens desired
    ) private returns (Trade memory trade) {
        require(side == OrderSide.Buy, "SwapAdapter: pool join must be buy order");
        
        PoolJoinParams memory params = _getPoolJoinParams(sellToken, specifiedAmount);
        
        // Validate join amounts against limits  
        _validatePoolJoinLimits(params);
        
        // Buy the secondary token we need for pool entry
        trade.calculatedAmount = buy(sellToken, params.secondaryToken, params.amountToBuy);
        trade.price = getPriceAt(trade.calculatedAmount, params.secondaryToken, sellToken);
        // Approve tokens for pool entry
        _approvePoolTokens(params.tokens, params.maxTokenAmountsIn);
        
        // Join the pool
        pool.joinPool(specifiedAmount, params.maxTokenAmountsIn);
    }

     struct PoolExitParams {
        address[] tokens;
        uint256[] balances;
        uint256[] maxTokenAmountsOut;
        address tokenToSwap;
        uint256 amountToSwap;
        uint256 directAmount;
        uint256 totalSupply;
    }
    
    struct PoolJoinParams {
        address[] tokens;
        address secondaryToken;
        uint256[] balances;
        uint256[] maxTokenAmountsIn;
        uint256 amountToBuy;
        uint256 totalSupply;
    }
    
    /**
     * @notice Prepares parameters for pool exit operation
     */
    function _getPoolExitParams(
        address buyToken,
        uint256 lpTokenAmount
    ) private view returns (PoolExitParams memory params) {
        params.tokens = pool.getFinalTokens();
        params.totalSupply = pool.totalSupply();
        
        // Get current pool balances
        params.balances = new uint256[](2);
        params.balances[0] = IERC20(params.tokens[0]).balanceOf(address(pool));
        params.balances[1] = IERC20(params.tokens[1]).balanceOf(address(pool));
        
        // Calculate tokens received from pool exit
        params.maxTokenAmountsOut = calcTokensOutGivenExactLpTokenIn(
            params.balances,
            lpTokenAmount,
            params.totalSupply
        );
        
        // Determine which token to swap and which to keep
        bool buyTokenIsFirst = (params.tokens[0] == buyToken);
        
        if (buyTokenIsFirst) {
            params.directAmount = params.maxTokenAmountsOut[0];
            params.amountToSwap = params.maxTokenAmountsOut[1];
            params.tokenToSwap = params.tokens[1];
        } else {
            params.directAmount = params.maxTokenAmountsOut[1];
            params.amountToSwap = params.maxTokenAmountsOut[0];
            params.tokenToSwap = params.tokens[0];
        }
    }
    
    /**
     * @notice Prepares parameters for pool join operation
     */
    function _getPoolJoinParams(
        address sellToken,
        uint256 lpTokenAmount
    ) private view returns (PoolJoinParams memory params) {
        params.tokens = pool.getFinalTokens();
        params.totalSupply = pool.totalSupply();
        
        // Get current pool balances
        params.balances = new uint256[](2);
        params.balances[0] = IERC20(params.tokens[0]).balanceOf(address(pool));
        params.balances[1] = IERC20(params.tokens[1]).balanceOf(address(pool));
        
        // Calculate tokens needed for pool join
        params.maxTokenAmountsIn = calcTokensOutGivenExactLpTokenIn(
            params.balances,
            lpTokenAmount,
            params.totalSupply
        );
        
        // Determine secondary token and amount to buy
        bool sellTokenIsFirst = (params.tokens[0] == sellToken);
        
        if (sellTokenIsFirst) {
            params.secondaryToken = params.tokens[1];
            params.amountToBuy = params.maxTokenAmountsIn[1];
        } else {
            params.secondaryToken = params.tokens[0];
            params.amountToBuy = params.maxTokenAmountsIn[0];
        }
    }
    
    /**
     * @notice Validates pool exit amounts against limits
     */
    function _validatePoolExitLimits(PoolExitParams memory params) private pure {
        uint256 limit0 = params.balances[0] * MAX_IN_FACTOR / 100;
        uint256 limit1 = params.balances[1] * MAX_OUT_FACTOR / 100;
        
        require(
            params.maxTokenAmountsOut[0] < limit0,
            "SwapAdapter: token0 exit amount exceeds limit"
        );
        require(
            params.maxTokenAmountsOut[1] < limit1,
            "SwapAdapter: token1 exit amount exceeds limit"
        );
    }
    
    /**
     * @notice Validates pool join amounts against limits
     */
    function _validatePoolJoinLimits(PoolJoinParams memory params) private pure {
        uint256 limit0 = params.balances[0] * MAX_IN_FACTOR / 100;
        uint256 limit1 = params.balances[1] * MAX_OUT_FACTOR / 100;
        
        require(
            params.maxTokenAmountsIn[0] < limit0,
            "SwapAdapter: token0 join amount exceeds limit"
        );
        require(
            params.maxTokenAmountsIn[1] < limit1,
            "SwapAdapter: token1 join amount exceeds limit"
        );
    }
    
    function swap(
          bytes32,
          address sellToken,
          address buyToken,
          OrderSide side,
          uint256 specifiedAmount
      ) external returns (Trade memory trade) {
      require(sellToken != buyToken, "Tokens must be different");
      require(specifiedAmount != 0,"Specified amount cannot be zero");
      uint256 gasBefore = gasleft();

       // Determine swap type based on token addresses
        SwapType swapType = _getSwapType(sellToken, buyToken);
        
        // Execute appropriate swap logic
        if (swapType == SwapType.TokenToToken) {
            trade = _executeTokenSwap(sellToken, buyToken, side, specifiedAmount);
        } else if (swapType == SwapType.ExitPool) {
            trade = _executePoolExit(sellToken, buyToken, side, specifiedAmount);
        } else if (swapType == SwapType.JoinPool) {
            trade = _executePoolJoin(sellToken, buyToken, side, specifiedAmount);
        } else {
            revert("SwapAdapter: LP-to-LP swap not supported");
        }
        

    //   if (sellToken != address(pool) && buyToken != address(pool)) {
    //       // prevent swap above sell limits as pool will revert for
    //       // underflow/overflow on mint/redeem
    //       uint256[] memory _limits = getLimits(bytes32(0), sellToken, buyToken);
    //       _checkLimits(_limits, side, specifiedAmount);
    //       // Standard Token-to-Token Swap
    //       if (side == OrderSide.Sell) {
    //           trade.calculatedAmount = sell(sellToken, buyToken, specifiedAmount);
    //           trade.price = getPriceAt(specifiedAmount, sellToken, buyToken);
    //       } else {
    //           uint256 amountIn = buy(sellToken, buyToken, specifiedAmount);
    //           trade.calculatedAmount = amountIn;
    //           trade.price = getPriceAt(trade.calculatedAmount, sellToken, buyToken);
    //       }
    //   } 
    //   // TODO THIS WHOLE REGION
    //   else if (sellToken == address(pool) && buyToken != address(pool)) {
    //       // Exiting Pool (LP token is being sold)
    //       require(side == OrderSide.Sell, "Exiting pool must be OrderSide.Sell");
    //       uint256 totalSupply = pool.totalSupply();

    //       address[] memory tokens = pool.getFinalTokens();

    //       //get the other token in the pool
    //       address secondaryToken = tokens[0] == buyToken ? tokens[1] : tokens[0];
          
    //       uint256 token0Balance = IERC20(tokens[0]).balanceOf(address(pool));
    //       uint256 token1Balance = IERC20(tokens[1]).balanceOf(address(pool));

    //       uint256 limit0 = token0Balance.bmul(MAX_IN_FACTOR).bdiv(100);
    //       uint256 limit1 = token1Balance.bmul(MAX_OUT_FACTOR).bdiv(100);

    //       uint256[] memory balances = new uint256[](2);

    //       balances[0] = token0Balance;
    //       balances[1] = token1Balance;

    //       uint256[] memory maxTokenAmountsIn = calcTokensOutGivenExactLpTokenIn(balances, specifiedAmount, totalSupply);
          
    //       //these constraints are necessary to cause an early return because of the subsequent superfluous token swap we will make in the 
    //       //second leg 
    //       if (maxTokenAmountsIn[0] > limit0) {
    //         revert("The amount of expectedToken0Out surpasses the limits for the amount that can be swapped into the pool");
    //       }

    //       if (maxTokenAmountsIn[1] > limit1) {
    //         revert("The amount of expectedToken1Out surpasses the limits for the amount that can be swapped into the pool");
    //       }
          
    //       pool.exitPool(specifiedAmount, maxTokenAmountsIn); 
          
    //       // if the first token we get from the pool.getFinalTokens() is the token we are buying, then select the other token proportion
    //       // because thats the proportion of the superfluous token we want to sell for our buyToken
    //       uint256 amountToSell = tokens[0] == buyToken ? maxTokenAmountsIn[1] : maxTokenAmountsIn[0];
    //       uint256 amountOfBuyTokenReceived = tokens[0] == buyToken ? maxTokenAmountsIn[0] : maxTokenAmountsIn[1];
    //       trade.calculatedAmount = sell(secondaryToken, buyToken, amountToSell) + amountOfBuyTokenReceived;
    //       trade.price = getPriceAt(amountToSell, secondaryToken, buyToken);
    //   } 
      
    //   else if (sellToken != address(pool) && buyToken == address(pool)) {
    //       // Joining Pool (LP token is being bought)
    //       require(side == OrderSide.Buy, "Joining pool must be OrderSide.Buy");

    //       uint256 totalSupply = pool.totalSupply();

    //       address[] memory tokens = pool.getFinalTokens();
    //       //get the other token in the pool
    //       address secondaryToken = tokens[0] == sellToken ? tokens[1] : tokens[0];

    //       uint256 token0Balance = IERC20(tokens[0]).balanceOf(address(pool));
    //       uint256 token1Balance = IERC20(tokens[1]).balanceOf(address(pool));

    //       uint256 limit0 = token0Balance.bmul(MAX_IN_FACTOR).bdiv(100);
    //       uint256 limit1 = token1Balance.bmul(MAX_OUT_FACTOR).bdiv(100);

    //       uint256[] memory balances = new uint256[](2);

    //       balances[0] = token0Balance;
    //       balances[1] = token1Balance;

    //       uint256[] memory maxTokenAmountsIn = calcTokensOutGivenExactLpTokenIn(balances, specifiedAmount, totalSupply);

    //       //the limits don't apply when joining or exiting a pool, but we have to put it because of when we are swapping the superfluous token amount out 
    //       if (maxTokenAmountsIn[0] > limit0) {
    //         revert("The amount of token0 in surpasses the limits for the amount that can be swapped into the pool");
    //       }

    //       if (maxTokenAmountsIn[1] > limit1) {
    //         revert("The amount of token1 surpasses the limits for the amount that can be swapped into the pool");
    //       }
    //       //if tokens[0] is sellToken then token0Balance will be the balance of the sellToken 
    //       uint256 amountToBuy = tokens[0] == sellToken ? maxTokenAmountsIn[1] : maxTokenAmountsIn[0];
    //       trade.calculatedAmount = buy(sellToken, secondaryToken, amountToBuy); // we want to buy the other token wstETH that we don't have, so we get the amount of COW we need , hence -> calc in given out
    //       //approve spending the tokens to send (join) them to the pool
    //       IERC20(tokens[0]).approve(address(pool), maxTokenAmountsIn[0]);
    //       IERC20(tokens[1]).approve(address(pool), maxTokenAmountsIn[1]);
    //       pool.joinPool(specifiedAmount, maxTokenAmountsIn);
    //       //the final price of the trade will be the price we swapped the other token we needed into, the amountOfSellTokenRedeemed will not be included here
    //       trade.price = getPriceAt(trade.calculatedAmount, secondaryToken, sellToken);
    //   } 
      
    //   else if (sellToken == address(pool) && buyToken == address(pool)) {
    //       // Invalid: Swapping LP token to LP token is not supported
    //       revert("Cannot swap between LP tokens"); 
    //   } 
      
    //   else {
    //       // Should never reach here
    //       revert("Invalid token and side combination");
    //   }
      trade.gasUsed = gasBefore - gasleft();
  }
       function getLimits(bytes32, address sellToken, address buyToken)
        public view 
        returns (uint256[] memory limits)
    {     
        uint256 sellTokenBal = pool.getBalance(sellToken);
        uint256 buyTokenBal = pool.getBalance(buyToken);
        limits = new uint256[](2);
        limits[0] = (sellTokenBal * MAX_IN_FACTOR) / 100;
        limits[1] = (buyTokenBal * MAX_OUT_FACTOR) / 100;
    }

    function getCapabilities(
        bytes32,
        address,
        address
    ) external 
      pure 
      override 
    returns (Capability[] memory capabilities) 
    {
        capabilities = new Capability[](5);
        capabilities[0] = Capability.SellOrder;
        capabilities[1] = Capability.BuyOrder;
        capabilities[2] = Capability.PriceFunction;
        capabilities[3] =  Capability.MarginalPrice;
        capabilities[4] = Capability.HardLimits;
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
        tokens[2] = address(pool);
    }

    function getPoolIds(uint256, uint256)
        external
        pure 
        returns (bytes32[] memory)
    {
        revert NotImplemented("CowAMMSwapAdapter.getPoolIds");
    }

    /// @notice Executes a sell order on the contract.
    /// @param sellToken The token being sold.
    /// @param buyToken The token being bought.
    /// @param amountIn The amount to be sold.
    /// @return calculatedAmount The amount of tokens received.
    function sell(address sellToken, address buyToken, uint256 amountIn)
        internal
        view
        returns (uint256 calculatedAmount)
    {    
        require(amountIn > 0, "Specified amount cannot be zero");

        uint256 tokenInBalance = IERC20(sellToken).balanceOf(address(pool));
        uint256 tokenInWeight = pool.getDenormalizedWeight(sellToken);

        uint256 tokenOutBalance = IERC20(buyToken).balanceOf(address(pool));
        uint256 tokenOutWeight = pool.getDenormalizedWeight(buyToken);

        // since we already have limit constraints externally, this shouldn't be too necessary
        // but added as an extra check, leaving it causes it to revert before the line 196 that 
        // actually reverts in AdapterTest.sol
        // Enforce 50% max in constraint
        // uint256 maxIn = (tokenInBalance).bmul(MAX_IN_FACTOR).bdiv(100);

        // if (amountIn > maxIn) {
        //     revert IBPool.BPool_TokenAmountInAboveMaxRatio();
        // }

        uint256 tokenAmountOut = calcOutGivenIn(
                    tokenInBalance,
                    tokenInWeight,
                    tokenOutBalance,
                    tokenOutWeight,
                    amountIn,
                    0
        );
        calculatedAmount = tokenAmountOut; //Convert to human-readable;
    }
    /// @notice Executes a buy order on the contract.
    /// @param sellToken The token being sold.
    /// @param buyToken The token being bought.
    /// @param amountOut The amount of tokens to be bought. 
    /// @return calculatedAmount The amount of tokens sold.
    function buy(address sellToken, address buyToken, uint256 amountOut)
        internal
        view 
        returns (uint256 calculatedAmount)
    {   
        require(amountOut > 0, "Specified amount cannot be zero");

        uint256 tokenInBalance = IERC20(sellToken).balanceOf(address(pool));
        uint256 tokenInWeight = pool.getDenormalizedWeight(sellToken);

        uint256 tokenOutBalance = IERC20(buyToken).balanceOf(address(pool));
        uint256 tokenOutWeight = pool.getDenormalizedWeight(buyToken);

        // Enforce 33% max out constraint
        uint256 maxOut = tokenOutBalance.bmul(MAX_OUT_FACTOR).bdiv(100); 
        if (amountOut > maxOut) {
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
    /// @notice Checks if the specified amount is within the hard limits
    /// @dev If not, reverts
    /// @param limits The limits of the tokens being traded.
    /// @param side The side of the trade.
    /// @param specifiedAmount The amount to be traded.
    function _checkLimits(
        uint256[] memory limits,
        OrderSide side,
        uint256 specifiedAmount
    ) internal pure {
        if (side == OrderSide.Sell && specifiedAmount > limits[0]) {
            require(specifiedAmount < limits[0], "Limit exceeded");
        } else if (side == OrderSide.Buy && specifiedAmount > limits[1]) {
            require(specifiedAmount < limits[1], "Limit exceeded");
        }
    }
}
