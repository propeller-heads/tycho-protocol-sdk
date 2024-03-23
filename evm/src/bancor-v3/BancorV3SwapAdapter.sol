// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {IERC20, ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

library MathEx {

    struct Uint512 {
        uint256 hi; // 256 most significant bits
        uint256 lo; // 256 least significant bits
    }

    /**
     * @dev returns the largest integer smaller than or equal to `x * y / z`
     */
    function mulDivF(uint256 x, uint256 y, uint256 z) internal pure returns (uint256) {
        Uint512 memory xy = mul512(x, y);

        // if `x * y < 2 ^ 256`
        if (xy.hi == 0) {
            return xy.lo / z;
        }

        // assert `x * y / z < 2 ^ 256`
        if (xy.hi >= z) {
            revert Overflow();
        }

        uint256 m = _mulMod(x, y, z); // `m = x * y % z`
        Uint512 memory n = _sub512(xy, m); // `n = x * y - m` hence `n / z = floor(x * y / z)`

        // if `n < 2 ^ 256`
        if (n.hi == 0) {
            return n.lo / z;
        }

        uint256 p = _unsafeSub(0, z) & z; // `p` is the largest power of 2 which `z` is divisible by
        uint256 q = _div512(n, p); // `n` is divisible by `p` because `n` is divisible by `z` and `z` is divisible by `p`
        uint256 r = _inv256(z / p); // `z / p = 1 mod 2` hence `inverse(z / p) = 1 mod 2 ^ 256`
        return _unsafeMul(q, r); // `q * r = (n / p) * inverse(z / p) = n / z`
    }

    /**
     * @dev returns the value of `x * y`
     */
    function mul512(uint256 x, uint256 y) internal pure returns (Uint512 memory) {
        uint256 p = _mulModMax(x, y);
        uint256 q = _unsafeMul(x, y);
        if (p >= q) {
            return Uint512({ hi: p - q, lo: q });
        }
        
        return Uint512({ hi: _unsafeSub(p, q) - 1, lo: q });
    }

    /**
     * @dev returns the value of `x - y`, given that `x >= y`
     */
    function _sub512(Uint512 memory x, uint256 y) private pure returns (Uint512 memory) {
        if (x.lo >= y) {
            return Uint512({ hi: x.hi, lo: x.lo - y });
        }
        return Uint512({ hi: x.hi - 1, lo: _unsafeSub(x.lo, y) });
    }

    /**
     * @dev returns `(x - y) % 2 ^ 256`
     */
    function _unsafeSub(uint256 x, uint256 y) private pure returns (uint256) {
        unchecked {
            return x - y;
        }
    }

    /**
     * @dev returns `(x * y) % 2 ^ 256`
     */
    function _unsafeMul(uint256 x, uint256 y) private pure returns (uint256) {
        unchecked {
            return x * y;
        }
    }

    /**
     * @dev returns `x * y % z`
     */
    function _mulMod(uint256 x, uint256 y, uint256 z) private pure returns (uint256) {
        return mulmod(x, y, z);
    }

    /**
     * @dev returns `x * y % (2 ^ 256 - 1)`
     */
    function _mulModMax(uint256 x, uint256 y) private pure returns (uint256) {
        return mulmod(x, y, type(uint256).max);
    }


    /**
     * @dev returns the value of `x / pow2n`, given that `x` is divisible by `pow2n`
     */
    function _div512(Uint512 memory x, uint256 pow2n) private pure returns (uint256) {
        uint256 pow2nInv = _unsafeAdd(_unsafeSub(0, pow2n) / pow2n, 1); // `1 << (256 - n)`
        return _unsafeMul(x.hi, pow2nInv) | (x.lo / pow2n); // `(x.hi << (256 - n)) | (x.lo >> n)`
    }

    /**
     * @dev returns the inverse of `d` modulo `2 ^ 256`, given that `d` is congruent to `1` modulo `2`
     */
    function _inv256(uint256 d) private pure returns (uint256) {
        // approximate the root of `f(x) = 1 / x - d` using the newton–raphson convergence method
        uint256 x = 1;
        for (uint256 i = 0; i < 8; i++) {
            x = _unsafeMul(x, _unsafeSub(2, _unsafeMul(x, d))); // `x = x * (2 - x * d) mod 2 ^ 256`
        }
        return x;
    }

    /**
     * @dev returns `(x + y) % 2 ^ 256`
     */
    function _unsafeAdd(uint256 x, uint256 y) private pure returns (uint256) {
        unchecked {
            return x + y;
        }
    }

}

/// @title BancorV3Swap Adapter
contract BancorV3SwapAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;
    
    uint32 constant PPM_RESOLUTION = 1_000_000;

    IBancorV3BancorNetwork public immutable bancorNetwork;
    IBancorV3BancorNetworkInfo public immutable bancorNetworkInfo;
    IBancorV3PoolCollection public immutable bancorPoolCollection;
    IERC20 immutable bnt;

    constructor (address bancorNetwork_, address bancorNetworkInfo_, address bancorPoolCollection_) {
        bancorNetwork = IBancorV3BancorNetwork(bancorNetwork_);
        bancorNetworkInfo = IBancorV3BancorNetworkInfo(bancorNetworkInfo_);
        bancorPoolCollection = IBancorV3PoolCollection(bancorPoolCollection_);
        bnt = bancorNetworkInfo.bnt();
    }

    /// @dev check if sellToken and buyToken are tradeable
    modifier onlySupportedTokens(address _sellToken, address _buyToken) {
        Token sellToken = Token(_sellToken);
        Token buyToken = Token(_buyToken);
        bool sellTokenPoolEnabled = bancorNetworkInfo.tradingEnabled(sellToken);
        bool buyTokenPoolEnabled = bancorNetworkInfo.tradingEnabled(buyToken);

        if(!sellTokenPoolEnabled || !buyTokenPoolEnabled) {
            revert Unavailable("This swap is not enabled");
        }

        _;
    }

    function price(
        bytes32,
        IERC20 _sellToken,
        IERC20 _buyToken,
        uint256[] memory _specifiedAmounts
    ) external view override returns (Fraction[] memory _prices) {
        
    }

    // 4. Price Function
    // Chiamare tradeOutputAndFeeBySourceAmount e usare output per aggiornare le riserve 
    // Aggiorno le riserve con le formule che ho già trovato
    // Chiamo la funzione per Fee PPM (PoolCollection)
    // Riproduco in locale la formula di tradeOutputAndFeeBySourceAmount con input amount millesimale rispetto alla riserva
    // Numeratore: uint256 targetAmount = MathEx.mulDivF(targetBalance, sourceAmount, sourceBalance + sourceAmount) - uint256 tradingFeeAmount = MathEx.mulDivF(targetAmount, feePPM, PPM_RESOLUTION);
    // Denominatore: Input amount (millesimale)

    function getPriceAt(uint256 _amountIn, IERC20 _sellToken, IERC20 _buyToken) 
    external
    view
    onlySupportedTokens(address(_sellToken), address(_buyToken))
    returns (Fraction memory)
    {   
        uint256 numerator;
        uint256 denominator;
        
        Token sellToken = Token(address(_sellToken));
        Token buyToken = Token(address(_buyToken));
        Token BNT = Token(address(bnt));

        uint256 amountIn = _amountIn;

        if (sellToken == BNT || buyToken == BNT) {
        
        (uint256 tradingLiquiditySellTokenBefore, uint256 tradingLiquidityBuyTokenBefore) = getTradingLiquidityBntPool(sellToken, buyToken);

        bancorPoolCollection.TradeAmountAndFee memory tf = bancorPoolCollection.tradeOutputAndFeeBySourceAmount(sellToken, buyToken, sourceAmount);

            if (sellToken == BNT) {

                uint256 calculatedLiquiditySellTokenAfter = tradingLiquiditySellTokenBefore + (amountIn - tf.networkFeeAmount);
                uint256 calculatedLiquidityBuyTokenAfter = tradingLiquidityBuyTokenBefore - tf.amount;

                uint32 feePPM = bancorNetworkInfo.tradingFeePPM(buyToken);

            } else {

                calculatedLiquiditySellTokenAfter = tradingLiquiditySellTokenBefore + amountIn;
                calculatedLiquidityBuyTokenAfter = tradingLiquidityBuyTokenBefore - (tf.amount + tf.networkFeeAmount);
                
                feePPM = bancorNetworkInfo.tradingFeePPM(sellToken);

            }

            uint256 punctualAmountIn = calculatedLiquiditySellTokenAfter/1000;
            uint256 targetAmount = MathEx.mulDivF(calculatedLiquidityBuyTokenAfter, punctualAmountIn, calculatedLiquiditySellTokenAfter + punctualAmountIn);
            uint256 tradingFeeAmount = MathEx.mulDivF(targetAmount, feePPM, PPM_RESOLUTION); /// What is PPM_RESOLUTION?
            
            uint256 missingDecimalsSellToken = 18 - uint256(IERC20Detailed(address(_sellToken)).decimals());
            uint256 missingDecimalsBuyToken = 18 - uint256(IERC20Detailed(address(_buyToken)).decimals());

            numerator = (targetAmount - tradingFeeAmount) * 10 ** missingDecimalsBuyToken;
            denominator = punctualAmountIn * 10 ** missingDecimalsSellToken;

            return Fraction(numerator, denominator);

        } else {

            return Fraction(0,0);
        }

    }

    /// @inheritdoc ISwapAdapter
    function swap(
        bytes32,
        IERC20 _sellToken,
        IERC20 _buyToken,
        OrderSide side,
        uint256 specifiedAmount
        ) 
        external
        override
        onlySupportedTokens(address(_sellToken), address(_buyToken))
        returns (Trade memory trade) {
        if (specifiedAmount == 0) {
            return trade;
        }

        uint256 gasBefore = gasleft();
        if (side == OrderSide.Sell) {
            trade.calculatedAmount =
                sell(_sellToken, _buyToken, specifiedAmount);
        } else {
            trade.calculatedAmount =
                buy(_sellToken, _buyToken, specifiedAmount);
        }
        trade.gasUsed = gasBefore - gasleft();
        trade.price = getPriceSwapAt( _sellToken, _buyToken);
    }

    /// @notice Executes a sell order on a given pool.
    /// @param _sellToken The token being sold.
    /// @param _buyToken The token being bought.
    /// @param amount The amount to be traded.
    /// @return calculatedAmount The amount of tokens received.
    function sell(
        IERC20 _sellToken,
        IERC20 _buyToken,
        uint256 amount
    ) internal returns (uint256 calculatedAmount) {
        Token sellToken = Token(address(_sellToken));
        Token buyToken = Token(address(_buyToken));
        
        uint256 amountOut = bancorNetworkInfo.tradeOutputBySourceAmount(sellToken, buyToken, amount);
        if (amountOut == 0) {
            revert Unavailable("AmountOut is zero!");
        }

        // First, approve the network contract to spend tokens
        _sellToken.safeTransferFrom(msg.sender, address(this), amount);
        _sellToken.approve(address(bancorNetwork), amount);

        return bancorNetwork.tradeBySourceAmount(
            sellToken,
            buyToken,
            amount,
            amountOut,
            block.timestamp + 300,
            msg.sender
        );

    }

    /// @notice Executes a sell order on a given pool.
    /// @param _sellToken The token being sold.
    /// @param _buyToken The token being bought.
    /// @param amount The amount of _buyToken to buy and receive.
    /// @return calculatedAmount The amount of tokens sold.
    function buy(
        IERC20 _sellToken,
        IERC20 _buyToken,
        uint256 amount
    ) internal returns (uint256 calculatedAmount) {
        Token sellToken = Token(address(_sellToken));
        Token buyToken = Token(address(_buyToken));
        
        uint256 amountIn = bancorNetworkInfo.tradeInputByTargetAmount(sellToken, buyToken, amount);
        if (amountIn == 0) {
            revert Unavailable("AmountIn is zero!");
        }

        // First, approve the network contract to spend tokens
        _sellToken.safeTransferFrom(msg.sender, address(this), amountIn);
        _sellToken.approve(address(bancorNetwork), amountIn);

        return bancorNetwork.tradeByTargetAmount(
            sellToken,
            buyToken,
            amount,
            amountIn,
            block.timestamp + 300,
            msg.sender
        );

    }


    function getPriceSwapAt (IERC20 _sellToken, IERC20 _buyToken) 
    internal
    returns (Fraction memory) {
        Token sellToken = Token(address(_sellToken));
        Token buyToken = Token(address(_buyToken));
        Token BNT = Token(address(bnt));

        uint256 tradingLiquiditySellTokenAfter = (sellToken == BNT) ? 
        uint256(getTradingLiquidity(BNT).bntTradingLiquidity) :
        uint256(getTradingLiquidity(sellToken).baseTokenTradingLiquidity);

        uint256 missingDecimalsSellToken = 18 - uint256(IERC20Detailed(address(_sellToken)).decimals());
        uint256 missingDecimalsBuyToken = 18 - uint256(IERC20Detailed(address(_buyToken)).decimals());

        uint256 punctualAmountIn = tradingLiquiditySellTokenAfter/1000;

        uint256 amountOut = bancorNetworkInfo.tradeOutputBySourceAmount(sellToken, buyToken, punctualAmountIn);

        return Fraction(amountOut * 10 ** missingDecimalsBuyToken, punctualAmountIn * 10 ** missingDecimalsSellToken);

    }

    /// @inheritdoc ISwapAdapter
    /// @dev Limits are underestimated at 90% of total liquidity inside pools
    function getLimits(bytes32, IERC20 _sellToken, IERC20 _buyToken)
    external
    view
    override
    onlySupportedTokens(address(_sellToken), address(_buyToken))
    returns (uint256[] memory limits)
    {
        limits = new uint256[](2) ;
        Token sellToken = Token(address(_sellToken));
        Token buyToken = Token(address(_buyToken));
        Token BNT = Token(address(bnt));

        if (_sellToken == bnt || _buyToken == bnt) {
            Token token = (_sellToken == bnt) ? buyToken : sellToken;
        uint256 tradingLiquidityBuyToken = (_sellToken == bnt) ? bancorNetworkInfo.tradingLiquidity(token).baseTokenTradingLiquidity 
        : bancorNetworkInfo.tradingLiquidity(token).bntTradingLiquidity;

        limits[1] = tradingLiquidityBuyToken * 90 / 100;
        limits[0] = bancorNetworkInfo.tradeInputByTargetAmount(sellToken, buyToken, limits[1]);

        } else {

            uint256 maxBntTradeable = 
            (bancorNetworkInfo.tradingLiquidity(buyToken).bntTradingLiquidity < bancorNetworkInfo.tradingLiquidity(sellToken).bntTradingLiquidity ?
            bancorNetworkInfo.tradingLiquidity(buyToken).bntTradingLiquidity : bancorNetworkInfo.tradingLiquidity(sellToken).bntTradingLiquidity)
            * 90 / 100;

            limits[0] = bancorNetworkInfo.tradeInputByTargetAmount(sellToken, BNT, maxBntTradeable);
            limits[1] = bancorNetworkInfo.tradeOutputBySourceAmount(sellToken, buyToken, limits[0]);
        }
    }

    /// @inheritdoc ISwapAdapter
    function getCapabilities(bytes32, IERC20, IERC20)
        external
        pure
        override
        returns (Capability[] memory capabilities)
    {
        capabilities = new Capability[](3);
        capabilities[0] = Capability.SellOrder;
        capabilities[1] = Capability.BuyOrder;
        capabilities[2] = Capability.PriceFunction;
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(bytes32 poolId)
        external
        pure
        override
        returns (IERC20[] memory tokens)
    {
        tokens = new IERC20[](1);
        address tokenAddress = address(bytes20(poolId));
        tokens[0] = IERC20(tokenAddress);
    }

    /// @inheritdoc ISwapAdapter
    function getPoolIds(uint256 offset, uint256 limit)
        external
        view
        override
        returns (bytes32[] memory ids)
    {
        uint256 endIdx = offset + limit;
        Token[] memory tokenPools = bancorNetwork.liquidityPools(); 
        if (endIdx > tokenPools.length) {
            endIdx = tokenPools.length;
        }
        ids = new bytes32[](endIdx - offset);
        for (uint256 i = 0; i < ids.length; i++) {
            ids[i] = bytes20(address(tokenPools[offset + i]));
        }
    }

    function getTradingLiquidityBntPool(Token _sellToken, Token _buyToken) 
    internal 
    returns (uint256 tradingLiquiditySellToken, uint256 tradingLiquidityBuyToken) 
    {
        TradingLiquidity memory tradingLiquidityPool;
        if (sellToken == bnt) {
            tradingLiquidityPool = getTradingLiquidity(_buyToken);
            tradingLiquiditySellToken = uint256(tradingLiquidityPool.bntTradingLiquidity);
            tradingLiquidityBuyToken = uint256(tradingLiquidityPool.baseTokenTradingLiquidity);
        } else {
            tradingLiquidityPool = getTradingLiquidity(_sellToken);
            tradingLiquiditySellToken = uint256(tradingLiquidityPool.baseTokenTradingLiquidity);
            tradingLiquidityBuyToken = uint256(tradingLiquidityPool.bntTradingLiquidity);
        }
    }

    function getTradingLiquidity(Token token) public view returns (TradingLiquidity memory) {
        return bancorNetworkInfo.tradingLiquidity(token);
    }

}

interface IBancorV3BancorNetworkInfo {

    /// @dev returns the BNT contract
    function bnt() external view returns (IERC20);

    /// @dev returns the trading liquidity in a given pool
    function tradingLiquidity(Token pool) external view returns (TradingLiquidity memory);

    /// @dev returns the trading fee (in units of PPM)
    function tradingFeePPM(Token pool) external view returns (uint32);
    
    /// @dev returns whether trading is enabled
    function tradingEnabled(Token pool) external view returns (bool);

    /// @dev returns the output amount when trading by providing the source amount
    function tradeOutputBySourceAmount(
        Token sourceToken,
        Token targetToken,
        uint256 sourceAmount
    ) external view returns (uint256);

    /// @dev returns the input amount when trading by providing the target amount
    function tradeInputByTargetAmount(
        Token sourceToken,
        Token targetToken,
        uint256 targetAmount
    ) external view returns (uint256);

}

interface IBancorV3PoolCollection {

    struct TradeAmountAndFee {
    uint256 amount; // the source/target amount (depending on the context) resulting from the trade
    uint256 tradingFeeAmount; // the trading fee amount
    uint256 networkFeeAmount; // the network fee amount (always in units of BNT)
    }

    /**
     * @dev returns the output amount and fee when trading by providing the source amount
     */
    function tradeOutputAndFeeBySourceAmount(
        Token sourceToken,
        Token targetToken,
        uint256 sourceAmount
    ) external view returns (TradeAmountAndFee memory);

    /**
     * @dev returns the input amount and fee when trading by providing the target amount
     */
    function tradeInputAndFeeByTargetAmount(
        Token sourceToken,
        Token targetToken,
        uint256 targetAmount
    ) external view returns (TradeAmountAndFee memory);


}

interface IBancorV3BancorNetwork {

    //function poolCollections() external view returns (IPoolCollection[] memory);

    /// @dev returns the set of all liquidity pools
    function liquidityPools() external view returns (Token[] memory);

    /// @dev returns the set of all valid pool collections
    // function poolCollections() external view returns (IPoolCollection[] memory);

    /// @dev performs a trade by providing the input source amount, sends the proceeds to the optional beneficiary (or
    /// to the address of the caller, in case it's not supplied), and returns the trade target amount
    /// @notice the caller must have approved the network to transfer the source tokens on its behalf (except for in the native token case)
    function tradeBySourceAmount(
        Token sourceToken,
        Token targetToken,
        uint256 sourceAmount,
        uint256 minReturnAmount,
        uint256 deadline,
        address beneficiary
    ) external payable returns (uint256);

    /// @dev performs a trade by providing the output target amount, sends the proceeds to the optional beneficiary (or
    /// to the address of the caller, in case it's not supplied), and returns the trade source amount
    /// @notice the caller must have approved the network to transfer the source tokens on its behalf (except for in the native token case)
    function tradeByTargetAmount(
        Token sourceToken,
        Token targetToken,
        uint256 targetAmount,
        uint256 maxSourceAmount,
        uint256 deadline,
        address beneficiary
    ) external payable returns (uint256);

}

interface Token {

}

interface IERC20Detailed is IERC20 {
    function decimals() external view returns (uint8);
}


struct TradingLiquidity {
    uint128 bntTradingLiquidity;
    uint128 baseTokenTradingLiquidity;
}