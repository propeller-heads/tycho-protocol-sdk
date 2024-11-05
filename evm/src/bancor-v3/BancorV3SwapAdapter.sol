// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {ISwapAdapter} from "../interfaces/ISwapAdapter.sol";
import {
    SafeERC20,
    IERC20
} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

library MathEx {
    error Overflow();

    struct Uint512 {
        uint256 hi; // 256 most significant bits
        uint256 lo; // 256 least significant bits
    }

    /**
     * @dev returns the largest integer smaller than or equal to `x * y / z`
     */
    function mulDivF(uint256 x, uint256 y, uint256 z)
        internal
        pure
        returns (uint256)
    {
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
        Uint512 memory n = _sub512(xy, m); // `n = x * y - m` hence `n / z =
        // floor(x * y / z)`

        // if `n < 2 ^ 256`
        if (n.hi == 0) {
            return n.lo / z;
        }

        uint256 p = _unsafeSub(0, z) & z; // `p` is the largest power of 2 which
        // `z` is divisible by
        uint256 q = _div512(n, p); // `n` is divisible by `p` because `n` is
        // divisible by `z` and `z` is divisible by `p`
        uint256 r = _inv256(z / p); // `z / p = 1 mod 2` hence `inverse(z / p) =
        // 1 mod 2 ^ 256`
        return _unsafeMul(q, r); // `q * r = (n / p) * inverse(z / p) = n / z`
    }

    /**
     * @dev returns the value of `x * y`
     */
    function mul512(uint256 x, uint256 y)
        internal
        pure
        returns (Uint512 memory)
    {
        uint256 p = _mulModMax(x, y);
        uint256 q = _unsafeMul(x, y);
        if (p >= q) {
            return Uint512({hi: p - q, lo: q});
        }

        return Uint512({hi: _unsafeSub(p, q) - 1, lo: q});
    }

    /**
     * @dev returns the value of `x - y`, given that `x >= y`
     */
    function _sub512(Uint512 memory x, uint256 y)
        private
        pure
        returns (Uint512 memory)
    {
        if (x.lo >= y) {
            return Uint512({hi: x.hi, lo: x.lo - y});
        }
        return Uint512({hi: x.hi - 1, lo: _unsafeSub(x.lo, y)});
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
    function _mulMod(uint256 x, uint256 y, uint256 z)
        private
        pure
        returns (uint256)
    {
        return mulmod(x, y, z);
    }

    /**
     * @dev returns `x * y % (2 ^ 256 - 1)`
     */
    function _mulModMax(uint256 x, uint256 y) private pure returns (uint256) {
        return mulmod(x, y, type(uint256).max);
    }

    /**
     * @dev returns the value of `x / pow2n`, given that `x` is divisible by
     * `pow2n`
     */
    function _div512(Uint512 memory x, uint256 pow2n)
        private
        pure
        returns (uint256)
    {
        uint256 pow2nInv = _unsafeAdd(_unsafeSub(0, pow2n) / pow2n, 1); // `1 <<
        // (256 - n)`
        return _unsafeMul(x.hi, pow2nInv) | (x.lo / pow2n); // `(x.hi << (256 -
            // n)) | (x.lo >> n)`
    }

    /**
     * @dev returns the inverse of `d` modulo `2 ^ 256`, given that `d` is
     * congruent to `1` modulo `2`
     */
    function _inv256(uint256 d) private pure returns (uint256) {
        // approximate the root of `f(x) = 1 / x - d` using the newton–raphson
        // convergence method
        uint256 x = 1;
        for (uint256 i = 0; i < 8; i++) {
            x = _unsafeMul(x, _unsafeSub(2, _unsafeMul(x, d))); // `x = x * (2 -
                // x * d) mod 2 ^ 256`
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

/// @title BancorV3SwapAdapter
contract BancorV3SwapAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    struct GeneralPriceCache {
        uint256 punctualAmountIn;
        uint256 targetAmount;
        uint256 tradingFeeAmount;
        uint32 feePPM;
        uint256 calculatedLiquiditySellTokenAfter;
        uint256 calculatedLiquidityBuyTokenAfter;
        uint256 amountIn;
    }

    struct DoublePriceCache {
        uint256 tradingLiquiditySellTokenSellTokenPoolBefore;
        uint256 tradingLiquidityBntSellTokenPoolBefore;
        uint256 tradingLiquidityBntBuyTokenPoolBefore;
        uint256 tradingLiquidityBuyTokenBuyTokenPoolBefore;
        uint256 calculatedLiquiditySellTokenSellTokenPoolAfter;
        uint256 calculatedLiquidityBntSellTokenPoolAfter;
        uint256 calculatedLiquidityBntBuyTokenPoolAfter;
        uint256 calculatedLiquidityBuyTokenBuyTokenPoolAfter;
        uint256 feePPMSellTokenPool;
        uint256 feePPMBuyTokenPool;
        uint256 intermediateTargetAmount;
        uint256 tradingFeeAmountSellTokenPool;
        uint256 amountOutBnt;
        uint256 amountOutBuyToken;
    }

    uint32 private constant PPM_RESOLUTION = 1_000_000;
    address public constant BNT = 0x1F573D6Fb3F13d689FF844B4cE37794d79a7FF1C;

    IBancorV3BancorNetwork private immutable i_bancorNetwork;
    IBancorV3BancorNetworkInfo private immutable i_bancorNetworkInfo;

    /// @dev check if sellToken and buyToken are tradeable
    modifier onlySupportedTokens(address sellToken, address buyToken) {
        bool sellTokenPoolEnabled =
            i_bancorNetworkInfo.tradingEnabled(Token(sellToken));
        bool buyTokenPoolEnabled =
            i_bancorNetworkInfo.tradingEnabled(Token(buyToken));

        if (!sellTokenPoolEnabled || !buyTokenPoolEnabled) {
            revert Unavailable("This swap is not enabled");
        }

        _;
    }

    constructor(address bancorNetworkInfo, address bancorNetwork) {
        i_bancorNetworkInfo = IBancorV3BancorNetworkInfo(bancorNetworkInfo);
        i_bancorNetwork = IBancorV3BancorNetwork(bancorNetwork);
    }

    /// @dev enable receive to fill the contract with ether for payable swaps
    receive() external payable {}

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32,
        address sellToken,
        address buyToken,
        uint256[] memory specifiedAmounts
    )
        external
        view
        override
        onlySupportedTokens(sellToken, buyToken)
        returns (Fraction[] memory prices)
    {
        IBancorV3PoolCollection bancorPoolCollection =
            checkSameCollection(sellToken, buyToken);
        prices = new Fraction[](specifiedAmounts.length);

        for (uint256 i = 0; i < specifiedAmounts.length; i++) {
            prices[i] = getPriceAt(
                specifiedAmounts[i], sellToken, buyToken, bancorPoolCollection
            );
        }
    }

    /// @inheritdoc ISwapAdapter
    /// @notice Executes a swap on the contract.
    /// @param sellToken The token being sold.
    /// @param buyToken The token being bought.
    /// @param side Either buy or sell.
    /// @param specifiedAmount The amount to be traded.
    /// @return trade The amount of tokens being sold or bought.
    function swap(
        bytes32,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    )
        external
        override
        onlySupportedTokens(sellToken, buyToken)
        returns (Trade memory trade)
    {
        checkSameCollection(sellToken, buyToken);
        if (specifiedAmount == 0) {
            return trade;
        }

        uint256 gasBefore = gasleft();
        if (side == OrderSide.Sell) {
            trade.calculatedAmount = sell(sellToken, buyToken, specifiedAmount);
        } else {
            trade.calculatedAmount = buy(sellToken, buyToken, specifiedAmount);
        }
        trade.gasUsed = gasBefore - gasleft();
        trade.price = getPriceSwapAt(sellToken, buyToken);
    }

    /// @inheritdoc ISwapAdapter
    /// @dev Limits are underestimated at 90% of total liquidity inside pools
    function getLimits(bytes32, address _sellToken, address _buyToken)
        external
        view
        override
        onlySupportedTokens(_sellToken, _buyToken)
        returns (uint256[] memory limits)
    {
        limits = new uint256[](2);
        Token sellToken = Token(_sellToken);
        Token buyToken = Token(_buyToken);
        Token bnt = Token(BNT);

        if (_sellToken == BNT || _buyToken == BNT) {
            Token token = (_sellToken == BNT) ? buyToken : sellToken;
            uint256 tradingLiquidityBuyToken = (_sellToken == BNT)
                ? i_bancorNetworkInfo.tradingLiquidity(token)
                    .baseTokenTradingLiquidity
                : i_bancorNetworkInfo.tradingLiquidity(token).bntTradingLiquidity;

            limits[1] = (tradingLiquidityBuyToken * 90) / 100;
            limits[0] = i_bancorNetworkInfo.tradeInputByTargetAmount(
                sellToken, buyToken, limits[1]
            );
        } else {
            uint256 maxBntTradeable = (
                (
                    i_bancorNetworkInfo.tradingLiquidity(buyToken)
                        .bntTradingLiquidity
                        < i_bancorNetworkInfo.tradingLiquidity(sellToken)
                            .bntTradingLiquidity
                        ? i_bancorNetworkInfo.tradingLiquidity(buyToken)
                            .bntTradingLiquidity
                        : i_bancorNetworkInfo.tradingLiquidity(sellToken)
                            .bntTradingLiquidity
                ) * 90
            ) / 100;

            limits[0] = i_bancorNetworkInfo.tradeInputByTargetAmount(
                sellToken, bnt, maxBntTradeable
            );
            limits[1] = i_bancorNetworkInfo.tradeOutputBySourceAmount(
                sellToken, buyToken, limits[0]
            );
        }
    }

    /// @inheritdoc ISwapAdapter
    function getCapabilities(bytes32, address, address)
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
        returns (address[] memory tokens)
    {
        tokens = new address[](2);
        tokens[0] = address(bytes20(poolId));
        tokens[1] = BNT;
    }

    /// @inheritdoc ISwapAdapter
    /// @dev poolIds in BanvorV3 corresponds to addresses of single tokens,
    /// since each pool is paired with bnt
    function getPoolIds(uint256 offset, uint256 limit)
        external
        view
        override
        returns (bytes32[] memory ids)
    {
        Token[] memory tokenPools = i_bancorNetwork.liquidityPools();

        uint256 numEnabledPools = 0;

        for (uint256 i = 0; i < tokenPools.length; i++) {
            if (i_bancorNetworkInfo.tradingEnabled(tokenPools[i])) {
                numEnabledPools++;
            }
        }

        bytes32[] memory poolIds = new bytes32[](numEnabledPools);

        uint256 newPoolIdIndex = 0;

        for (uint256 i = 0; i < tokenPools.length; i++) {
            if (i_bancorNetworkInfo.tradingEnabled(tokenPools[i])) {
                poolIds[newPoolIdIndex] = bytes20(address(tokenPools[i]));
                newPoolIdIndex++;
            }
        }
        uint256 endIdx = offset + limit;

        if (endIdx > poolIds.length) {
            endIdx = poolIds.length;
        }
        ids = new bytes32[](endIdx - offset);
        for (uint256 i = 0; i < ids.length; i++) {
            ids[i] = poolIds[i + offset];
        }
    }

    /// @notice Executes a sell order on a given pool.
    /// @param sellToken The token being sold.
    /// @param buyToken The token being bought.
    /// @param amount The amount to be traded.
    /// @return calculatedAmount The amount of tokens received.
    function sell(address sellToken, address buyToken, uint256 amount)
        internal
        returns (uint256 calculatedAmount)
    {
        IERC20(sellToken).safeTransferFrom(msg.sender, address(this), amount);
        IERC20(sellToken).safeIncreaseAllowance(
            address(i_bancorNetwork), amount
        );

        return i_bancorNetwork.tradeBySourceAmount(
            Token(sellToken),
            Token(buyToken),
            amount,
            1,
            block.timestamp + 300,
            msg.sender
        );
    }

    /// @notice Executes a buy order on a given pool.
    /// @param sellToken The token being sold.
    /// @param buyToken The token being bought.
    /// @param amount The amount of buyToken to buy and receive.
    /// @return calculatedAmount The amount of tokens sold.
    function buy(address sellToken, address buyToken, uint256 amount)
        internal
        returns (uint256 calculatedAmount)
    {
        uint256 amountIn = i_bancorNetworkInfo.tradeInputByTargetAmount(
            Token(sellToken), Token(buyToken), amount
        );
        if (amountIn == 0) {
            revert Unavailable("AmountIn is zero!");
        }

        IERC20(sellToken).safeTransferFrom(msg.sender, address(this), amountIn);
        IERC20(sellToken).safeIncreaseAllowance(
            address(i_bancorNetwork), amountIn
        );

        return i_bancorNetwork.tradeByTargetAmount(
            Token(sellToken),
            Token(buyToken),
            amount,
            amountIn,
            block.timestamp + 300,
            msg.sender
        );
    }

    /// @dev check if buyToken and sellToken are in the same collection and
    /// return collection
    function checkSameCollection(address sellToken, address buyToken)
        internal
        view
        returns (IBancorV3PoolCollection bancorPoolCollection)
    {
        address poolCollectionSellToken =
            address(i_bancorNetwork.collectionByPool(Token(sellToken)));
        address poolCollectionBuyToken =
            address(i_bancorNetwork.collectionByPool(Token(buyToken)));

        if ((sellToken == BNT) && (poolCollectionBuyToken == address(0))) {
            revert Unavailable("No collection is associeted with the Buy Token");
        }

        if ((buyToken == BNT) && (poolCollectionSellToken == address(0))) {
            revert Unavailable(
                "No collection is associeted with the Sell Token"
            );
        }

        if (
            poolCollectionBuyToken == address(0)
                && poolCollectionBuyToken == poolCollectionSellToken
        ) {
            revert Unavailable("The tokens are not in the same collection");
        }

        if (
            sellToken != BNT && buyToken != BNT
                && poolCollectionSellToken != poolCollectionBuyToken
        ) {
            revert Unavailable("The tokens are not in the same collection");
        }

        bancorPoolCollection = (poolCollectionSellToken == address(0))
            ? IBancorV3PoolCollection(poolCollectionBuyToken)
            : IBancorV3PoolCollection(poolCollectionSellToken);
    }

    /// @notice Calculates pair prices for specified amounts
    /// @param amountIn The amount of the token being sold
    /// @param sellToken the token to sell
    /// @param buyToken The token ro buy
    /// @return (fraction) price as a fraction corresponding to the provided
    /// amount
    function getPriceAt(
        uint256 amountIn,
        address sellToken,
        address buyToken,
        IBancorV3PoolCollection bancorPoolCollection
    ) internal view returns (Fraction memory) {
        GeneralPriceCache memory priceCache;

        priceCache.amountIn = amountIn;

        if (sellToken == BNT || buyToken == BNT) {
            (
                uint256 tradingLiquiditySellTokenBefore,
                uint256 tradingLiquidityBuyTokenBefore
            ) = getTradingLiquidityBntPool(sellToken, buyToken);

            TradeAmountAndFee memory tf = bancorPoolCollection
                .tradeOutputAndFeeBySourceAmount(
                Token(sellToken), Token(buyToken), priceCache.amountIn
            );

            if (sellToken == BNT) {
                priceCache.calculatedLiquiditySellTokenAfter =
                tradingLiquiditySellTokenBefore
                    + (priceCache.amountIn - tf.networkFeeAmount);
                priceCache.calculatedLiquidityBuyTokenAfter =
                    tradingLiquidityBuyTokenBefore - tf.amount;

                priceCache.feePPM =
                    i_bancorNetworkInfo.tradingFeePPM(Token(buyToken));
            } else {
                priceCache.calculatedLiquiditySellTokenAfter =
                    tradingLiquiditySellTokenBefore + priceCache.amountIn;
                priceCache.calculatedLiquidityBuyTokenAfter =
                tradingLiquidityBuyTokenBefore
                    - (tf.amount + tf.networkFeeAmount);

                priceCache.feePPM =
                    i_bancorNetworkInfo.tradingFeePPM(Token(sellToken));
            }

            priceCache.punctualAmountIn =
                priceCache.calculatedLiquiditySellTokenAfter / 100000;
            priceCache.targetAmount = MathEx.mulDivF(
                priceCache.calculatedLiquidityBuyTokenAfter,
                priceCache.punctualAmountIn,
                priceCache.calculatedLiquiditySellTokenAfter
                    + priceCache.punctualAmountIn
            );
            priceCache.tradingFeeAmount = MathEx.mulDivF(
                priceCache.targetAmount, priceCache.feePPM, PPM_RESOLUTION
            );

            return Fraction(
                priceCache.targetAmount - priceCache.tradingFeeAmount,
                priceCache.punctualAmountIn
            );
        } else {
            DoublePriceCache memory doublePriceCache;

            // Get tradingLiquidity of sellTokenPool Before Swap
            (
                doublePriceCache.tradingLiquiditySellTokenSellTokenPoolBefore,
                doublePriceCache.tradingLiquidityBntSellTokenPoolBefore
            ) = getTradingLiquidityBntPool(sellToken, BNT);

            // Get tradingLiquidity of buyTokenPool before the Swap
            (
                doublePriceCache.tradingLiquidityBntBuyTokenPoolBefore,
                doublePriceCache.tradingLiquidityBuyTokenBuyTokenPoolBefore
            ) = getTradingLiquidityBntPool(BNT, buyToken);

            // Simulate swap SellToken Bnt in SellTokenPool
            TradeAmountAndFee memory tf1 = bancorPoolCollection
                .tradeOutputAndFeeBySourceAmount(
                Token(sellToken), Token(BNT), priceCache.amountIn
            );
            TradeAmountAndFee memory tf2 = bancorPoolCollection
                .tradeOutputAndFeeBySourceAmount(
                Token(BNT), Token(buyToken), tf1.amount
            );

            // CalculatedLiquidity sellTokenPool after simulated swap sellToken
            // --> BNT
            doublePriceCache.calculatedLiquiditySellTokenSellTokenPoolAfter =
            doublePriceCache.tradingLiquiditySellTokenSellTokenPoolBefore
                + priceCache.amountIn;
            doublePriceCache.calculatedLiquidityBntSellTokenPoolAfter =
            doublePriceCache.tradingLiquidityBntSellTokenPoolBefore
                - (tf1.amount + tf1.networkFeeAmount);

            // CalculatedLiquidity buyTokenPool after simulated swap BNT -->
            // buyToken
            doublePriceCache.calculatedLiquidityBntBuyTokenPoolAfter =
            doublePriceCache.tradingLiquidityBntBuyTokenPoolBefore
                + (tf1.amount - tf2.networkFeeAmount);
            doublePriceCache.calculatedLiquidityBuyTokenBuyTokenPoolAfter =
            doublePriceCache.tradingLiquidityBuyTokenBuyTokenPoolBefore
                - tf2.amount;

            doublePriceCache.feePPMSellTokenPool =
                i_bancorNetworkInfo.tradingFeePPM(Token(sellToken));

            priceCache.punctualAmountIn = doublePriceCache
                .calculatedLiquiditySellTokenSellTokenPoolAfter / 100000;
            doublePriceCache.intermediateTargetAmount = MathEx.mulDivF(
                doublePriceCache.calculatedLiquidityBntSellTokenPoolAfter,
                priceCache.punctualAmountIn,
                doublePriceCache.calculatedLiquiditySellTokenSellTokenPoolAfter
                    + priceCache.punctualAmountIn
            );
            doublePriceCache.tradingFeeAmountSellTokenPool = MathEx.mulDivF(
                doublePriceCache.intermediateTargetAmount,
                doublePriceCache.feePPMSellTokenPool,
                PPM_RESOLUTION
            );
            doublePriceCache.amountOutBnt = doublePriceCache
                .intermediateTargetAmount
                - doublePriceCache.tradingFeeAmountSellTokenPool;

            doublePriceCache.feePPMBuyTokenPool =
                i_bancorNetworkInfo.tradingFeePPM(Token(buyToken));

            priceCache.targetAmount = MathEx.mulDivF(
                doublePriceCache.calculatedLiquidityBuyTokenBuyTokenPoolAfter,
                doublePriceCache.amountOutBnt,
                doublePriceCache.calculatedLiquidityBntBuyTokenPoolAfter
                    + doublePriceCache.amountOutBnt
            );
            priceCache.tradingFeeAmount = MathEx.mulDivF(
                priceCache.targetAmount,
                doublePriceCache.feePPMBuyTokenPool,
                PPM_RESOLUTION
            );
            doublePriceCache.amountOutBuyToken =
                priceCache.targetAmount - priceCache.tradingFeeAmount;

            return Fraction(
                doublePriceCache.amountOutBuyToken, priceCache.punctualAmountIn
            );
        }
    }

    /// @notice Calculates pair prices for specified amounts. We use as amount
    /// in to calculate the price post swap
    /// a fractional value (1/100000) of the trading liquidity, to impact as
    /// little as possible on the slippage
    /// @param sellToken the token to sell
    /// @param buyToken The token ro buy
    /// @return (fraction) price as a fraction corresponding to the provided
    /// amount after a
    /// a swap has been executed
    function getPriceSwapAt(address sellToken, address buyToken)
        internal
        view
        returns (Fraction memory)
    {
        uint256 tradingLiquiditySellTokenAfter = (sellToken == BNT)
            ? uint256(getTradingLiquidity(buyToken).bntTradingLiquidity)
            : uint256(getTradingLiquidity(sellToken).baseTokenTradingLiquidity);

        uint256 punctualAmountIn = tradingLiquiditySellTokenAfter / 100000;

        uint256 amountOut = i_bancorNetworkInfo.tradeOutputBySourceAmount(
            Token(sellToken), Token(buyToken), punctualAmountIn
        );

        return Fraction(amountOut, punctualAmountIn);
    }

    /// @notice Get the liquidity of buyToken and sellToken in a specific pool
    /// @param sellToken the token to sell
    /// @param buyToken The token ro buy
    function getTradingLiquidityBntPool(address sellToken, address buyToken)
        internal
        view
        returns (
            uint256 tradingLiquiditySellToken,
            uint256 tradingLiquidityBuyToken
        )
    {
        TradingLiquidity memory tradingLiquidityPool;

        if (sellToken == BNT) {
            tradingLiquidityPool = getTradingLiquidity(buyToken);
            tradingLiquiditySellToken =
                uint256(tradingLiquidityPool.bntTradingLiquidity);
            tradingLiquidityBuyToken =
                uint256(tradingLiquidityPool.baseTokenTradingLiquidity);
        } else {
            tradingLiquidityPool = getTradingLiquidity(sellToken);
            tradingLiquiditySellToken =
                uint256(tradingLiquidityPool.baseTokenTradingLiquidity);
            tradingLiquidityBuyToken =
                uint256(tradingLiquidityPool.bntTradingLiquidity);
        }
    }

    function getTradingLiquidity(address token)
        public
        view
        returns (TradingLiquidity memory)
    {
        return i_bancorNetworkInfo.tradingLiquidity(Token(token));
    }
}

interface IBancorV3BancorNetworkInfo {
    /// @dev returns the network contract
    function network() external view returns (IBancorV3BancorNetwork);

    /// @dev returns the BNT contract
    function bnt() external view returns (IERC20);

    /// @dev returns the trading liquidity in a given pool
    function tradingLiquidity(Token pool)
        external
        view
        returns (TradingLiquidity memory);

    /// @dev returns the trading fee (in units of PPM)
    function tradingFeePPM(Token pool) external view returns (uint32);

    /// @dev returns whether trading is enabled
    function tradingEnabled(Token pool) external view returns (bool);

    /// @dev returns the output amount when trading by providing the source
    /// amount
    function tradeOutputBySourceAmount(
        Token sourceToken,
        Token targetToken,
        uint256 sourceAmount
    ) external view returns (uint256);

    /// @dev returns the input amount when trading by providing the target
    /// amount
    function tradeInputByTargetAmount(
        Token sourceToken,
        Token targetToken,
        uint256 targetAmount
    ) external view returns (uint256);
}

interface IBancorV3PoolCollection {
    /// @dev returns the output amount and fee when trading by providing the
    /// source amount
    function tradeOutputAndFeeBySourceAmount(
        Token sourceToken,
        Token targetToken,
        uint256 sourceAmount
    ) external view returns (TradeAmountAndFee memory);

    /// @dev returns the input amount and fee when trading by providing the
    /// target amount
    function tradeInputAndFeeByTargetAmount(
        Token sourceToken,
        Token targetToken,
        uint256 targetAmount
    ) external view returns (TradeAmountAndFee memory);
}

interface IBancorV3BancorNetwork {
    /// @dev returns the set of all valid pool collections
    function poolCollections()
        external
        view
        returns (IBancorV3PoolCollection[] memory);

    // @dev returns the respective pool collection for the provided pool
    function collectionByPool(Token pool)
        external
        view
        returns (IBancorV3PoolCollection);

    /// @dev returns the set of all liquidity pools
    function liquidityPools() external view returns (Token[] memory);

    /// @dev performs a trade by providing the input source amount, sends the
    /// proceeds to the optional beneficiary (or
    /// to the address of the caller, in case it's not supplied), and returns
    /// the trade target amount
    /// @notice the caller must have approved the network to transfer the source
    /// tokens on its behalf (except for in the native token case)
    function tradeBySourceAmount(
        Token sourceToken,
        Token targetToken,
        uint256 sourceAmount,
        uint256 minReturnAmount,
        uint256 deadline,
        address beneficiary
    ) external payable returns (uint256);

    /// @dev performs a trade by providing the output target amount, sends the
    /// proceeds to the optional beneficiary (or
    /// to the address of the caller, in case it's not supplied), and returns
    /// the trade source amount
    /// @notice the caller must have approved the network to transfer the source
    /// tokens on its behalf (except for in the native token case)
    function tradeByTargetAmount(
        Token sourceToken,
        Token targetToken,
        uint256 targetAmount,
        uint256 maxSourceAmount,
        uint256 deadline,
        address beneficiary
    ) external payable returns (uint256);
}

interface Token {}

interface IERC20Detailed is IERC20 {
    function decimals() external view returns (uint8);
}

struct TradingLiquidity {
    uint128 bntTradingLiquidity;
    uint128 baseTokenTradingLiquidity;
}

struct TradeAmountAndFee {
    uint256 amount; // the source/target amount (depending on the context)
    // resulting from the trade
    uint256 tradingFeeAmount; // the trading fee amount
    uint256 networkFeeAmount; // the network fee amount (always in units of BNT)
}
