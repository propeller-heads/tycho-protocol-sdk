// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {ISwapAdapter} from "../interfaces/ISwapAdapter.sol";
import {SafeERC20, IERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

library MathEx {
    error Overflow();

    struct Uint512 {
        uint256 hi; // 256 most significant bits
        uint256 lo; // 256 least significant bits
    }

    /**
     * @dev returns the largest integer smaller than or equal to `x * y / z`
     */
    function mulDivF(
        uint256 x,
        uint256 y,
        uint256 z
    ) internal pure returns (uint256) {
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
    function mul512(
        uint256 x,
        uint256 y
    ) internal pure returns (Uint512 memory) {
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
    function _sub512(
        Uint512 memory x,
        uint256 y
    ) private pure returns (Uint512 memory) {
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
    function _mulMod(
        uint256 x,
        uint256 y,
        uint256 z
    ) private pure returns (uint256) {
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
    function _div512(
        Uint512 memory x,
        uint256 pow2n
    ) private pure returns (uint256) {
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

    uint32 constant PPM_RESOLUTION = 1_000_000;

    IBancorV3BancorNetwork public immutable bancorNetwork;
    IBancorV3BancorNetworkInfo public immutable bancorNetworkInfo;
    IERC20 immutable bnt;

    /// @dev check if sellToken and buyToken are tradeable
    modifier onlySupportedTokens(address _sellToken, address _buyToken) {
        bool sellTokenPoolEnabled = bancorNetworkInfo.tradingEnabled(
            Token(_sellToken)
        );
        bool buyTokenPoolEnabled = bancorNetworkInfo.tradingEnabled(
            Token(_buyToken)
        );

        if (!sellTokenPoolEnabled || !buyTokenPoolEnabled) {
            revert Unavailable("This swap is not enabled");
        }

        _;
    }

    constructor(address bancorNetworkInfo_) {
        bancorNetworkInfo = IBancorV3BancorNetworkInfo(bancorNetworkInfo_);
        bancorNetwork = IBancorV3BancorNetwork(bancorNetworkInfo.network());
        bnt = bancorNetworkInfo.bnt();
    }

    /// @dev enable receive to fill the contract with ether for payable swaps
    receive() external payable {}

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32,
        IERC20 _sellToken,
        IERC20 _buyToken,
        uint256[] memory specifiedAmounts
    )
        external
        view
        override
        onlySupportedTokens(address(_sellToken), address(_buyToken))
        returns (Fraction[] memory prices)
    {
        IBancorV3PoolCollection bancorPoolCollection = checkSameCollection(
            address(_sellToken),
            address(_buyToken)
        );
        prices = new Fraction[](specifiedAmounts.length);

        for (uint256 i = 0; i < specifiedAmounts.length; i++) {
            prices[i] = getPriceAt(
                specifiedAmounts[i],
                _sellToken,
                _buyToken,
                bancorPoolCollection
            );
        }
    }

    /// @inheritdoc ISwapAdapter
    /// @notice Executes a swap on the contract.
    /// @param _sellToken The token being sold.
    /// @param _buyToken The token being bought.
    /// @param side Either buy or sell.
    /// @param specifiedAmount The amount to be traded.
    /// @return trade The amount of tokens being sold or bought.
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
        returns (Trade memory trade)
    {
        checkSameCollection(address(_sellToken), address(_buyToken));
        if (specifiedAmount == 0) {
            return trade;
        }

        uint256 gasBefore = gasleft();
        if (side == OrderSide.Sell) {
            trade.calculatedAmount = sell(
                _sellToken,
                _buyToken,
                specifiedAmount
            );
        } else {
            trade.calculatedAmount = buy(
                _sellToken,
                _buyToken,
                specifiedAmount
            );
        }
        trade.gasUsed = gasBefore - gasleft();
        trade.price = getPriceSwapAt(_sellToken, _buyToken);
    }

    /// @inheritdoc ISwapAdapter
    /// @dev Limits are underestimated at 90% of total liquidity inside pools
    function getLimits(
        bytes32,
        IERC20 _sellToken,
        IERC20 _buyToken
    )
        external
        view
        override
        onlySupportedTokens(address(_sellToken), address(_buyToken))
        returns (uint256[] memory limits)
    {
        limits = new uint256[](2);
        Token sellToken = Token(address(_sellToken));
        Token buyToken = Token(address(_buyToken));
        Token BNT = Token(address(bnt));

        if (_sellToken == bnt || _buyToken == bnt) {
            Token token = (_sellToken == bnt) ? buyToken : sellToken;
            uint256 tradingLiquidityBuyToken = (_sellToken == bnt)
                ? bancorNetworkInfo
                    .tradingLiquidity(token)
                    .baseTokenTradingLiquidity
                : bancorNetworkInfo.tradingLiquidity(token).bntTradingLiquidity;

            limits[1] = (tradingLiquidityBuyToken * 90) / 100;
            limits[0] = bancorNetworkInfo.tradeInputByTargetAmount(
                sellToken,
                buyToken,
                limits[1]
            );
        } else {
            uint256 maxBntTradeable = ((
                bancorNetworkInfo
                    .tradingLiquidity(buyToken)
                    .bntTradingLiquidity <
                    bancorNetworkInfo
                        .tradingLiquidity(sellToken)
                        .bntTradingLiquidity
                    ? bancorNetworkInfo
                        .tradingLiquidity(buyToken)
                        .bntTradingLiquidity
                    : bancorNetworkInfo
                        .tradingLiquidity(sellToken)
                        .bntTradingLiquidity
            ) * 90) / 100;

            limits[0] = bancorNetworkInfo.tradeInputByTargetAmount(
                sellToken,
                BNT,
                maxBntTradeable
            );
            limits[1] = bancorNetworkInfo.tradeOutputBySourceAmount(
                sellToken,
                buyToken,
                limits[0]
            );
        }
    }

    /// @inheritdoc ISwapAdapter
    function getCapabilities(
        bytes32,
        IERC20,
        IERC20
    ) external pure override returns (Capability[] memory capabilities) {
        capabilities = new Capability[](3);
        capabilities[0] = Capability.SellOrder;
        capabilities[1] = Capability.BuyOrder;
        capabilities[2] = Capability.PriceFunction;
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(
        bytes32 poolId
    ) external pure override returns (IERC20[] memory tokens) {
        tokens = new IERC20[](1);
        address tokenAddress = address(bytes20(poolId));
        tokens[0] = IERC20(tokenAddress);
    }

    /// @inheritdoc ISwapAdapter
    /// @dev poolIds in BanvorV3 corresponds to addresses of single tokens,
    /// since each pool
    /// is paired with bnt
    function getPoolIds(
        uint256 offset,
        uint256 limit
    ) external view override returns (bytes32[] memory ids) {
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

        _sellToken.safeTransferFrom(msg.sender, address(this), amount);
        _sellToken.safeIncreaseAllowance(address(bancorNetwork), amount);

        return
            bancorNetwork.tradeBySourceAmount(
                sellToken,
                buyToken,
                amount,
                1,
                block.timestamp + 300,
                msg.sender
            );
    }

    /// @notice Executes a buy order on a given pool.
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

        uint256 amountIn = bancorNetworkInfo.tradeInputByTargetAmount(
            sellToken,
            buyToken,
            amount
        );
        if (amountIn == 0) {
            revert Unavailable("AmountIn is zero!");
        }

        _sellToken.safeTransferFrom(msg.sender, address(this), amountIn);
        _sellToken.safeIncreaseAllowance(address(bancorNetwork), amountIn);

        return
            bancorNetwork.tradeByTargetAmount(
                sellToken,
                buyToken,
                amount,
                amountIn,
                block.timestamp + 300,
                msg.sender
            );
    }

    /// @dev check if buyToken and sellToken are in the same collection and
    /// return collection
    function checkSameCollection(
        address _sellToken,
        address _buyToken
    ) internal view returns (IBancorV3PoolCollection bancorPoolCollection) {
        address poolCollectionSellToken = address(
            bancorNetwork.collectionByPool(Token(_sellToken))
        );
        address poolCollectionBuyToken = address(
            bancorNetwork.collectionByPool(Token(_buyToken))
        );

        if (
            (_sellToken == address(bnt)) &&
            (poolCollectionBuyToken == address(0))
        ) {
            revert Unavailable(
                "No collection is associeted with the Buy Token"
            );
        }

        if (
            (_buyToken == address(bnt)) &&
            (poolCollectionSellToken == address(0))
        ) {
            revert Unavailable(
                "No collection is associeted with the Sell Token"
            );
        }

        if (
            poolCollectionBuyToken == address(0) &&
            poolCollectionBuyToken == poolCollectionSellToken
        ) {
            revert Unavailable("The tokens are not in the same collection");
        }

        if (
            _sellToken != address(bnt) &&
            _buyToken != address(bnt) &&
            poolCollectionSellToken != poolCollectionBuyToken
        ) {
            revert Unavailable("The tokens are not in the same collection");
        }

        bancorPoolCollection = (poolCollectionSellToken == address(0))
            ? IBancorV3PoolCollection(poolCollectionBuyToken)
            : IBancorV3PoolCollection(poolCollectionSellToken);
    }

    /// @notice Calculates pair prices for specified amounts
    /// @param _amountIn The amount of the token being sold
    /// @param _sellToken the token to sell
    /// @param _buyToken The token ro buy
    /// @return (fraction) price as a fraction corresponding to the provided
    /// amount
    function getPriceAt(
        uint256 _amountIn,
        IERC20 _sellToken,
        IERC20 _buyToken,
        IBancorV3PoolCollection _bancorPoolCollection
    ) internal view returns (Fraction memory) {
        GeneralPriceCache memory priceCache;

        priceCache.amountIn = _amountIn;

        if (
            Token(address(_sellToken)) == Token(address(bnt)) ||
            Token(address(_buyToken)) == Token(address(bnt))
        ) {
            (
                uint256 tradingLiquiditySellTokenBefore,
                uint256 tradingLiquidityBuyTokenBefore
            ) = getTradingLiquidityBntPool(
                    Token(address(_sellToken)),
                    Token(address(_buyToken))
                );

            TradeAmountAndFee memory tf = _bancorPoolCollection
                .tradeOutputAndFeeBySourceAmount(
                    Token(address(_sellToken)),
                    Token(address(_buyToken)),
                    priceCache.amountIn
                );

            if (Token(address(_sellToken)) == Token(address(bnt))) {
                priceCache.calculatedLiquiditySellTokenAfter =
                    tradingLiquiditySellTokenBefore +
                    (priceCache.amountIn - tf.networkFeeAmount);
                priceCache.calculatedLiquidityBuyTokenAfter =
                    tradingLiquidityBuyTokenBefore -
                    tf.amount;

                priceCache.feePPM = bancorNetworkInfo.tradingFeePPM(
                    Token(address(_buyToken))
                );
            } else {
                priceCache.calculatedLiquiditySellTokenAfter =
                    tradingLiquiditySellTokenBefore +
                    priceCache.amountIn;
                priceCache.calculatedLiquidityBuyTokenAfter =
                    tradingLiquidityBuyTokenBefore -
                    (tf.amount + tf.networkFeeAmount);

                priceCache.feePPM = bancorNetworkInfo.tradingFeePPM(
                    Token(address(_sellToken))
                );
            }

            priceCache.punctualAmountIn =
                priceCache.calculatedLiquiditySellTokenAfter /
                100000;
            priceCache.targetAmount = MathEx.mulDivF(
                priceCache.calculatedLiquidityBuyTokenAfter,
                priceCache.punctualAmountIn,
                priceCache.calculatedLiquiditySellTokenAfter +
                    priceCache.punctualAmountIn
            );
            priceCache.tradingFeeAmount = MathEx.mulDivF(
                priceCache.targetAmount,
                priceCache.feePPM,
                PPM_RESOLUTION
            );

            return
                Fraction(
                    priceCache.targetAmount - priceCache.tradingFeeAmount,
                    priceCache.punctualAmountIn
                );
        } else {
            DoublePriceCache memory doublePriceCache;

            // Get tradingLiquidity of sellTokenPool Before Swap
            (
                doublePriceCache.tradingLiquiditySellTokenSellTokenPoolBefore,
                doublePriceCache.tradingLiquidityBntSellTokenPoolBefore
            ) = getTradingLiquidityBntPool(
                Token(address(_sellToken)),
                Token(address(bnt))
            );

            // Get tradingLiquidity of buyTokenPool before the Swap
            (
                doublePriceCache.tradingLiquidityBntBuyTokenPoolBefore,
                doublePriceCache.tradingLiquidityBuyTokenBuyTokenPoolBefore
            ) = getTradingLiquidityBntPool(
                Token(address(bnt)),
                Token(address(_buyToken))
            );

            // Simulate swap SellToken Bnt in SellTokenPool
            TradeAmountAndFee memory tf1 = _bancorPoolCollection
                .tradeOutputAndFeeBySourceAmount(
                    Token(address(_sellToken)),
                    Token(address(bnt)),
                    priceCache.amountIn
                );
            TradeAmountAndFee memory tf2 = _bancorPoolCollection
                .tradeOutputAndFeeBySourceAmount(
                    Token(address(bnt)),
                    Token(address(_buyToken)),
                    tf1.amount
                );

            // CalculatedLiquidity sellTokenPool after simulated swap sellToken
            // --> BNT
            doublePriceCache.calculatedLiquiditySellTokenSellTokenPoolAfter =
                doublePriceCache.tradingLiquiditySellTokenSellTokenPoolBefore +
                priceCache.amountIn;
            doublePriceCache.calculatedLiquidityBntSellTokenPoolAfter =
                doublePriceCache.tradingLiquidityBntSellTokenPoolBefore -
                (tf1.amount + tf1.networkFeeAmount);

            // CalculatedLiquidity buyTokenPool after simulated swap BNT -->
            // buyToken
            doublePriceCache.calculatedLiquidityBntBuyTokenPoolAfter =
                doublePriceCache.tradingLiquidityBntBuyTokenPoolBefore +
                (tf1.amount - tf2.networkFeeAmount);
            doublePriceCache.calculatedLiquidityBuyTokenBuyTokenPoolAfter =
                doublePriceCache.tradingLiquidityBuyTokenBuyTokenPoolBefore -
                tf2.amount;

            doublePriceCache.feePPMSellTokenPool = bancorNetworkInfo
                .tradingFeePPM(Token(address(_sellToken)));

            priceCache.punctualAmountIn =
                doublePriceCache
                    .calculatedLiquiditySellTokenSellTokenPoolAfter /
                100000;
            doublePriceCache.intermediateTargetAmount = MathEx.mulDivF(
                doublePriceCache.calculatedLiquidityBntSellTokenPoolAfter,
                priceCache.punctualAmountIn,
                doublePriceCache
                    .calculatedLiquiditySellTokenSellTokenPoolAfter +
                    priceCache.punctualAmountIn
            );
            doublePriceCache.tradingFeeAmountSellTokenPool = MathEx.mulDivF(
                doublePriceCache.intermediateTargetAmount,
                doublePriceCache.feePPMSellTokenPool,
                PPM_RESOLUTION
            );
            doublePriceCache.amountOutBnt =
                doublePriceCache.intermediateTargetAmount -
                doublePriceCache.tradingFeeAmountSellTokenPool;

            doublePriceCache.feePPMBuyTokenPool = bancorNetworkInfo
                .tradingFeePPM(Token(address(_buyToken)));

            priceCache.targetAmount = MathEx.mulDivF(
                doublePriceCache.calculatedLiquidityBuyTokenBuyTokenPoolAfter,
                doublePriceCache.amountOutBnt,
                doublePriceCache.calculatedLiquidityBntBuyTokenPoolAfter +
                    doublePriceCache.amountOutBnt
            );
            priceCache.tradingFeeAmount = MathEx.mulDivF(
                priceCache.targetAmount,
                doublePriceCache.feePPMBuyTokenPool,
                PPM_RESOLUTION
            );
            doublePriceCache.amountOutBuyToken =
                priceCache.targetAmount -
                priceCache.tradingFeeAmount;

            return
                Fraction(
                    doublePriceCache.amountOutBuyToken,
                    priceCache.punctualAmountIn
                );
        }
    }

    /// @notice Calculates pair prices for specified amounts. We use as amount
    /// in to calculate the price post swap
    /// a fractional value (1/100000) of the trading liquidity, to impact as
    /// little as possible on the slippage
    /// @param _sellToken the token to sell
    /// @param _buyToken The token ro buy
    /// @return (fraction) price as a fraction corresponding to the provided
    /// amount after a
    /// a swap has been executed
    function getPriceSwapAt(
        IERC20 _sellToken,
        IERC20 _buyToken
    ) internal view returns (Fraction memory) {
        Token sellToken = Token(address(_sellToken));
        Token buyToken = Token(address(_buyToken));
        Token BNT = Token(address(bnt));

        uint256 tradingLiquiditySellTokenAfter = (sellToken == BNT)
            ? uint256(getTradingLiquidity(buyToken).bntTradingLiquidity)
            : uint256(getTradingLiquidity(sellToken).baseTokenTradingLiquidity);

        uint256 punctualAmountIn = tradingLiquiditySellTokenAfter / 100000;

        uint256 amountOut = bancorNetworkInfo.tradeOutputBySourceAmount(
            sellToken,
            buyToken,
            punctualAmountIn
        );

        return Fraction(amountOut, punctualAmountIn);
    }

    /// @notice Get the liquidity of _buyToken and _sellToken in a specific pool
    /// @param _sellToken the token to sell
    /// @param _buyToken The token ro buy
    function getTradingLiquidityBntPool(
        Token _sellToken,
        Token _buyToken
    )
        internal
        view
        returns (
            uint256 tradingLiquiditySellToken,
            uint256 tradingLiquidityBuyToken
        )
    {
        TradingLiquidity memory tradingLiquidityPool;
        Token BNT = Token(address(bnt));

        if (_sellToken == BNT) {
            tradingLiquidityPool = getTradingLiquidity(_buyToken);
            tradingLiquiditySellToken = uint256(
                tradingLiquidityPool.bntTradingLiquidity
            );
            tradingLiquidityBuyToken = uint256(
                tradingLiquidityPool.baseTokenTradingLiquidity
            );
        } else {
            tradingLiquidityPool = getTradingLiquidity(_sellToken);
            tradingLiquiditySellToken = uint256(
                tradingLiquidityPool.baseTokenTradingLiquidity
            );
            tradingLiquidityBuyToken = uint256(
                tradingLiquidityPool.bntTradingLiquidity
            );
        }
    }

    function getTradingLiquidity(
        Token token
    ) public view returns (TradingLiquidity memory) {
        return bancorNetworkInfo.tradingLiquidity(token);
    }
}

interface IBancorV3BancorNetworkInfo {
    /// @dev returns the network contract
    function network() external view returns (IBancorV3BancorNetwork);

    /// @dev returns the BNT contract
    function bnt() external view returns (IERC20);

    /// @dev returns the trading liquidity in a given pool
    function tradingLiquidity(
        Token pool
    ) external view returns (TradingLiquidity memory);

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
    function collectionByPool(
        Token pool
    ) external view returns (IBancorV3PoolCollection);

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
