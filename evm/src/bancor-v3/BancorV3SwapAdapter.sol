// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {IERC20, ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";

contract BancorV3SwapAdapter is ISwapAdapter {
    IBancorV3BancorNetwork immutable bancorNetwork;
    IBancorV3BancorNetworkInfo immutable bancorNetworkInfo;
    IBancorV3PoolCollection immutable bancorPoolCollection;

    constructor (address bancorNetwork_, address bancorNetworkInfo_, address bancorPoolCollection_) {
        bancorNetwork = IBancorV3BancorNetwork(bancorNetwork_);
        bancorNetworkInfo = IBancorV3BancorNetworkInfo(bancorNetworkInfo_);
        bancorPoolCollection = IBancorV3PoolCollection(bancorPoolCollection_);
    }

    modifier onlySupportedTokens(address sellToken, address buyToken) {
        bool sellTokenPoolEnabled = bancorNetworkInfo.tradingEnabled(sellToken);
        bool buyTokenPoolEnabled = bancorNetworkInfo.tradingEnabled(sellToken);

        if(!sellTokenPoolEnabled || !buyTokenPoolEnabled) {
            revert Unavailable("This swap is not enabled");
        }

        _;
    }

    function price(
        bytes32 _poolId,
        IERC20 _sellToken,
        IERC20 _buyToken,
        uint256[] memory _specifiedAmounts
    ) external view override returns (Fraction[] memory _prices) {
        revert NotImplemented("TemplateSwapAdapter.price");
    }

    function getPriceAt(uint256 amountIn, IERC20 sellToken, IERC20 buyToken) 
    external
    view
    onlySupportedTokens(address(sellToken), address(buyToken))
    returns (Fraction Memory)
    {
        // 1. if sellToken or buyToken is BNT
            // call tradingLiquidity(Token that is not BNT)
            // 1.1 if sellToken = BNT
                // call tradeOutputAndFeeBySourceAmount(sellToken, buyToken, amountIn) -> amountOut, tradingFeeAmount, networkFeeAmount
                // numerator = tradingLiquidityBuyToken - amountOut
                // denominator = tradingLiquiditySellToken + amountIn
            // 1.2 else 
        // get amountOut by calling tradeInputByTargetAmount()
    }

    function swap(
        bytes32 poolId,
        IERC20 sellToken,
        IERC20 buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external returns (Trade memory trade) {
        revert NotImplemented("TemplateSwapAdapter.swap");
    }

    function getLimits(bytes32 poolId, IERC20 sellToken, IERC20 buyToken)
        external
        returns (uint256[] memory limits)
    {
        revert NotImplemented("TemplateSwapAdapter.getLimits");
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
        view
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


}

interface IBancorV3BancorNetworkInfo {

    /// @dev returns the trading liquidity in a given pool
    function tradingLiquidity(Token pool) external view returns (TradingLiquidity memory);

    /// @dev returns the trading fee (in units of PPM)
    function tradingFeePPM(Token pool) external view returns (uint32);
    
    /// @dev returns whether trading is enabled
    function tradingEnabled(Token pool) external view returns (bool);

}

interface IBancorV3PoolCollection {
        /**
     * @dev returns the output amount and fee when trading by providing the source amount
     TradeAmountAndFee({
                amount: result.targetAmount,
                tradingFeeAmount: result.tradingFeeAmount,
                networkFeeAmount: result.networkFeeAmount
            });
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

    /// @dev returns the respective pool collection for the provided pool
    // function collectionByPool(Token pool) external view returns (IPoolCollection);

    /**
     * @dev performs a trade by providing the input source amount, sends the proceeds to the optional beneficiary (or
     * to the address of the caller, in case it's not supplied), and returns the trade target amount
     *
     * requirements:
     *
     * - the caller must have approved the network to transfer the source tokens on its behalf (except for in the
     *   native token case)
     */
    function tradeBySourceAmount(
        Token sourceToken,
        Token targetToken,
        uint256 sourceAmount,
        uint256 minReturnAmount,
        uint256 deadline,
        address beneficiary
    ) external payable returns (uint256);

    /**
     * @dev performs a trade by providing the output target amount, sends the proceeds to the optional beneficiary (or
     * to the address of the caller, in case it's not supplied), and returns the trade source amount
     *
     * requirements:
     *
     * - the caller must have approved the network to transfer the source tokens on its behalf (except for in the
     *   native token case)
     */
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

struct TradeAmountAndFee {
    uint256 amount; // the source/target amount (depending on the context) resulting from the trade
    uint256 tradingFeeAmount; // the trading fee amount
    uint256 networkFeeAmount; // the network fee amount (always in units of BNT)
}