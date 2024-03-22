// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "forge-std/console.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";
import "src/bancor-v3/BancorV3SwapAdapter.sol";

/// @title TemplateSwapAdapterTest
/// @dev This is a template for a swap adapter test.
/// Test all functions that are implemented in your swap adapter, the two test included here are just an example.
/// Feel free to use UniswapV2SwapAdapterTest and BalancerV2SwapAdapterTest as a reference.
contract BancorV3SwapAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    BancorV3SwapAdapter adapter;

    address constant BANCOR_NETWORK_PROXY_ADDRESS = 0xeEF417e1D5CC832e619ae18D2F140De2999dD4fB;
    address constant BANCOR_NETWORK_INFO_PROXY_ADDRESS = 0x8E303D296851B320e6a697bAcB979d13c9D6E760;
    address constant POOL_COLLECTION_ADDRESS = 0xde1B3CcfC45e3F5bff7f43516F2Cd43364D883E4;

    IERC20 constant ETH = IERC20(0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE);
    IERC20 constant LINK = IERC20(0x514910771AF9Ca656af840dff83E8264EcF986CA);
    IERC20 constant BNT = IERC20(0x1F573D6Fb3F13d689FF844B4cE37794d79a7FF1C);
    IERC20 constant WBTC = IERC20(0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599);

    bytes32 constant PAIR = bytes32(0);
    uint256 constant TEST_ITERATIONS = 100;

    Token immutable eth = Token(address(ETH));
    Token immutable bnt = Token(address(BNT));
    Token immutable link = Token(address(LINK));
    Token immutable wbtc = Token(address(WBTC));
    
    receive() external payable {}

    function setUp() public {
        uint256 forkBlock = 19489171;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);

        adapter = new BancorV3SwapAdapter(BANCOR_NETWORK_PROXY_ADDRESS, BANCOR_NETWORK_INFO_PROXY_ADDRESS, POOL_COLLECTION_ADDRESS);
    }

    function getTradingLiquidity(Token sellToken, Token buyToken) 
    internal 
    returns (uint256 tradingLiquiditySellToken, uint256 tradingLiquidityBuyToken) 
    {
        TradingLiquidity memory tradingLiquidityPool;
        if (sellToken == bnt) {
            tradingLiquidityPool = adapter.getTradingLiquidity(buyToken);
            tradingLiquiditySellToken = uint256(tradingLiquidityPool.bntTradingLiquidity);
            tradingLiquidityBuyToken = uint256(tradingLiquidityPool.baseTokenTradingLiquidity);
        } else {
            tradingLiquidityPool = adapter.getTradingLiquidity(sellToken);
            tradingLiquiditySellToken = uint256(tradingLiquidityPool.baseTokenTradingLiquidity);
            tradingLiquidityBuyToken = uint256(tradingLiquidityPool.bntTradingLiquidity);
        }
    }

    function calculateLiquidityRatio(uint256 multiplier, uint256 tradingLiquidityBuyToken, uint256 tradingLiquiditySellToken)
    internal
    returns (uint256 tradingLiquidityRatio) 
    {
        tradingLiquidityRatio = multiplier * tradingLiquidityBuyToken/tradingLiquiditySellToken;
    }

    /*
    CALCULATING THE TRADING LIQUIDITY AFTER A SWAP

    It doesn't matter if the Orderside is Buy or Sell

    When selling BNT for a TOKEN:
    uint256 calculatedLiquidityLinkAfter = tradingLiquidityLinkBefore - amountOut;
    uint256 calculatedLiquidityBntAfter = tradingLiquidityBntBefore + (amountIn - tf.networkFeeAmount);

    When selling a TOKEN for BNT:
    uint256 calculatedLiquidityLinkAfter = tradingLiquidityLinkBefore + amountIn;
    uint256 calculatedLiquidityBntAfter = tradingLiquidityBntBefore - (amountOut + tf.networkFeeAmount);
    
     */


    /**
    1. chiamre poolCollection per vedere la collection dei 2 token sa tradare
    Se sono in Pool collection differenti -> Revert
    2. Castare l'address della Pool Collection in IPoolCollection

    3. Per calcolare price dopo swap 
    Fare lo swap 
    Fare il prodotto delle 2 chiamate a tradeOutputAndFeeBySourceAmount, ma passare un valore millesimale
    della riserva per evitare lo slippage

    4. Price Function
    Chiamare tradeOutputAndFeeBySourceAmount e usare output per aggiornare le riserve 
    Aggiorno le riserve con le formule che ho già trovato
    Chiamo la funzione per Fee PPM (PoolCollection)
    Riproduco in locale la formula di tradeOutputAndFeeBySourceAmount con input amount millesimale rispetto alla riserva
    Numeratore: uint256 targetAmount = MathEx.mulDivF(targetBalance, sourceAmount, sourceBalance + sourceAmount) - uint256 tradingFeeAmount = MathEx.mulDivF(targetAmount, feePPM, PPM_RESOLUTION);
    Denominatore: Input amount (millesimale)

    Test chiamare price con amountIn X e fare swap con lo stesso amountIn (swap di tipo sell)
    I due price devono coincidere
    
     */


    /// Test punctual price after swap | sellToken: LINK | buyToken: WBTC

    function testPunctualPriceAfterSwapLinkForWbtc () public {
        uint256 amountIn = 1000 ether;

        deal(address(LINK), address(this), amountIn);
        LINK.approve(address(adapter), amountIn);

        // Swap Link for WBTC
        uint256 amountOut = adapter.swap(PAIR, LINK, WBTC, OrderSide.Sell, amountIn).calculatedAmount;

        console.log(amountOut);

        (uint256 tradingLiquidityLinkAfter, uint256 tradingLiquidityWbtcAfter) = getTradingLiquidity(link, wbtc);

        uint256 sellTokenDecimals = uint256(IERC20Detailed(address(LINK)).decimals());

        // Calculate a very small amount of LINK to be used for simulating the swap
        // The goal is to convert the liquidity back to a 'whole' unit, take a small fraction, 
        // and then convert it back to the smallest unit.
        uint256 punctualAmountIn = (tradingLiquidityLinkAfter/10**sellTokenDecimals)/1000 * 10**sellTokenDecimals;

        console.log("sellTokenDecimals: ", sellTokenDecimals );
        console.log("tradingLiquidityLinkAfter", tradingLiquidityLinkAfter);
        console.log("punctualAmountIn: ", punctualAmountIn );

        IBancorV3PoolCollection poolColl = adapter.bancorPoolCollection();
        IBancorV3PoolCollection.TradeAmountAndFee memory tf = poolColl.tradeOutputAndFeeBySourceAmount(link, bnt, punctualAmountIn);
        IBancorV3PoolCollection.TradeAmountAndFee memory tf2 = poolColl.tradeOutputAndFeeBySourceAmount(bnt, wbtc, tf.amount);


        uint256 punctualPriceNumerator = tf2.amount;
        uint256 punctualPrice = tf2.amount*10**6/amountIn;
        uint256 punctualPrice = tf2.amount*10**6/amountIn;
        console.log("amountOut: ", tf2.amount*10**6);
        console.log("punctual price: ", punctualPrice);

    }

    /// Sell | SellToken: LINK | BuyToken: BNT
    /// All Tests Passed
    function testHowPriceWorksLinkForWbtc () public {

        uint256 amountIn = 1000 ether;

        /// Liquidity of BNT and LINK in LINK Pool BEFORE Swap
        (uint256 tradingLiquidityLinkBefore, uint256 tradingLiquidityBntBefore) = getTradingLiquidity(link, bnt);
        uint256 liquidityRatioBefore = calculateLiquidityRatio(10**18, tradingLiquidityBntBefore, tradingLiquidityLinkBefore);

        console.log("tradingLiquidity LINK before swap: ",tradingLiquidityLinkBefore);
        console.log("tradingLiquidity BNT before swap: ",tradingLiquidityBntBefore);
        console.log("liquidity ratio before swap: ", liquidityRatioBefore);


        console.log("######################################### Simulation of tradeOutputAndFeeBySourceAmount ###############");


        IBancorV3PoolCollection poolColl = adapter.bancorPoolCollection();
        IBancorV3PoolCollection.TradeAmountAndFee memory tf = poolColl.tradeOutputAndFeeBySourceAmount(link, bnt, amountIn);
        IBancorV3PoolCollection.TradeAmountAndFee memory tf2 = poolColl.tradeOutputAndFeeBySourceAmount(bnt, wbtc, tf.amount);


        console.log("SIMULATED amountOut", tf.amount);
        console.log("SIMULATED tradingFee", tf.tradingFeeAmount);
        console.log("SIMULATED networkFee", tf.networkFeeAmount);

        console.log("SIMULATED amountOut2", tf2.amount);
        console.log("SIMULATED tradingFee", tf2.tradingFeeAmount);
        console.log("SIMULATED networkFee", tf2.networkFeeAmount);
        

        deal(address(LINK), address(this), amountIn);
        LINK.approve(address(adapter), amountIn);

        /// LINK for BNT
        uint256 amountOut = adapter.swap(PAIR, LINK, WBTC, OrderSide.Sell, amountIn).calculatedAmount;

        console.log("/////////////////////////////////////////////// AFTER SWAP ///////////////////////////////////////////////");

        /// Liquidity of BNT and LINK in LINK Pool AFTER Swap
        (uint256 tradingLiquidityLinkAfter, uint256 tradingLiquidityBntAfter) = getTradingLiquidity(link, bnt);
        uint256 liquidityRatioAfter = calculateLiquidityRatio(10**18, tradingLiquidityBntAfter, tradingLiquidityLinkAfter);

        console.log("tradingLiquidity LINK after swap: ",tradingLiquidityLinkAfter);
        console.log("tradingLiquidity BNT after swap: ",tradingLiquidityBntAfter);
        console.log("liquidity ratio after swap: ", liquidityRatioAfter);

        console.log("/////////////////////////////////////////////// CALCULATED AFTER SWAP ///////////////////////////////////////////////");
        
        /// Calculate LINK liquidity after swap --> tradingLiquidityLinkBefore - amountOut
        uint256 calculatedLiquidityLinkAfter = tradingLiquidityLinkBefore + amountIn;
        /// Calculate BNT liquidity after swap --> tradingLiquidityBntBefore + amountIn
        uint256 calculatedLiquidityBntAfter = tradingLiquidityBntBefore - (amountOut + tf.networkFeeAmount);

        uint256 calculatedLiquidityRatioAfter = 10**18 * calculatedLiquidityBntAfter/calculatedLiquidityLinkAfter;

        console.log("CALCULATED tradingLiquidity LINK after swap: ",calculatedLiquidityLinkAfter);
        console.log("CALCULATED tradingLiquidity BNT after swap: ",calculatedLiquidityBntAfter);
        console.log("CALCULATED liquidity ratio after swap: ", calculatedLiquidityRatioAfter);

        console.log("amountOut: ", amountOut);

        /// Check if tradingLiquidityLinkAfter == calculatedLiquidityLinkAfter
        assertEq(tradingLiquidityLinkAfter, calculatedLiquidityLinkAfter);

        /// Check if tradingLiquidityBntAfter == calculatedLiquidityBntAfter
        assertEq(tradingLiquidityBntAfter, calculatedLiquidityBntAfter);

        /// Check if amountOut == amountOutSimulated
        assertEq(amountOut, tf.amount);
        
    }


    /// Buy BNT with LINK | SellToken: LINK | BuyToken: BNT
    /// All Test Passed
    function testHowPriceWorksBuyBntWithLink () public {

        uint256 amountOut = 10 ether;

        /// Liquidity of BNT and LINK in LINK Pool BEFORE Swap
        (uint256 tradingLiquidityLinkBefore, uint256 tradingLiquidityBntBefore) = getTradingLiquidity(link, bnt);
        uint256 liquidityRatioBefore = calculateLiquidityRatio(10**18, tradingLiquidityBntBefore, tradingLiquidityLinkBefore);

        console.log("tradingLiquidity LINK before swap: ",tradingLiquidityLinkBefore);
        console.log("tradingLiquidity BNT before swap: ",tradingLiquidityBntBefore);
        console.log("liquidity ratio before swap: ", liquidityRatioBefore);


        console.log("##################### Simulation of tradeInputAndFeeByTargetAmount ###############");

        IBancorV3PoolCollection poolColl = adapter.bancorPoolCollection();
        IBancorV3PoolCollection.TradeAmountAndFee memory tf = poolColl.tradeInputAndFeeByTargetAmount(link, bnt, amountOut);

        console.log("SIMULATED amountIn", tf.amount);
        console.log("SIMULATED tradingFee", tf.tradingFeeAmount);
        console.log("SIMULATED networkFee", tf.networkFeeAmount);

        deal(address(LINK), address(this), tf.amount);
        LINK.approve(address(adapter), tf.amount);

        /// Sell BNT for LINK
        uint256 amountIn = adapter.swap(PAIR, LINK, BNT, OrderSide.Buy, amountOut).calculatedAmount;

        console.log("/////////////////////////////////////////////// AFTER SWAP ///////////////////////////////////////////////");

        /// Liquidity of BNT and LINK in LINK Pool AFTER Swap
        (uint256 tradingLiquidityLinkAfter, uint256 tradingLiquidityBntAfter) = getTradingLiquidity(link, bnt);
        uint256 liquidityRatioAfter = calculateLiquidityRatio(10**18, tradingLiquidityBntAfter, tradingLiquidityLinkAfter);

        console.log("tradingLiquidity LINK after swap: ",tradingLiquidityLinkAfter);
        console.log("tradingLiquidity BNT after swap: ",tradingLiquidityBntAfter);
        console.log("liquidity ratio after swap: ", liquidityRatioAfter);

        console.log("/////////////////////////////////////////////// CALCULATED AFTER SWAP ///////////////////////////////////////////////");
        
        /// Calculate LINK liquidity after swap
        uint256 calculatedLiquidityLinkAfter = tradingLiquidityLinkBefore + amountIn;
        /// Calculate BNT liquidity after swap
        uint256 calculatedLiquidityBntAfter = tradingLiquidityBntBefore - (amountOut + tf.networkFeeAmount);

        uint256 calculatedLiquidityRatioAfter = 10**18 * calculatedLiquidityBntAfter/calculatedLiquidityLinkAfter;

        console.log("CALCULATED tradingLiquidity LINK after swap: ",calculatedLiquidityLinkAfter);
        console.log("CALCULATED tradingLiquidity BNT after swap: ",calculatedLiquidityBntAfter);
        console.log("CALCULATED liquidity ratio after swap: ", calculatedLiquidityRatioAfter);

        console.log("amountIn: ", amountIn);

        /// Check if tradingLiquidityLinkAfter == calculatedLiquidityLinkAfter
        assertEq(tradingLiquidityLinkAfter, calculatedLiquidityLinkAfter);

        /// Check if tradingLiquidityBntAfter == calculatedLiquidityBntAfter
        assertEq(tradingLiquidityBntAfter, calculatedLiquidityBntAfter);

        /// Check if amountIn == amountInSimulated
        assertEq(amountIn, tf.amount);
        
    }

    /// Buy LINK with BNT | SellToken: BNT | BuyToken: LINK
    /// All Tests Passed
    function testHowPriceWorksBuyLinkWithBnt () public {

        uint256 amountOut = 1 ether;

        /// Liquidity of BNT and LINK in LINK Pool BEFORE Swap
        (uint256 tradingLiquidityBntBefore, uint256 tradingLiquidityLinkBefore) = getTradingLiquidity(bnt, link);
        uint256 liquidityRatioBefore = calculateLiquidityRatio(10**18, tradingLiquidityLinkBefore, tradingLiquidityBntBefore);

        console.log("tradingLiquidity LINK before swap: ",tradingLiquidityLinkBefore);
        console.log("tradingLiquidity BNT before swap: ",tradingLiquidityBntBefore);
        console.log("liquidity ratio before swap: ", liquidityRatioBefore);


        console.log("//////////////////////////////// Simulation of tradeInputAndFeeByTargetAmount ///////////////////////////////");


        IBancorV3PoolCollection poolColl = adapter.bancorPoolCollection();
        IBancorV3PoolCollection.TradeAmountAndFee memory tf = poolColl.tradeInputAndFeeByTargetAmount(bnt, link, amountOut);

        console.log("SIMULATED amountIn", tf.amount);
        console.log("SIMULATED tradingFee", tf.tradingFeeAmount);
        console.log("SIMULATED networkFee", tf.tradingFeeAmount);

        deal(address(BNT), address(this), tf.amount);
        BNT.approve(address(adapter), tf.amount);

        /// Sell BNT for LINK
        uint256 amountIn = adapter.swap(PAIR, BNT, LINK, OrderSide.Buy, amountOut).calculatedAmount;

        console.log("/////////////////////////////////////////////// AFTER SWAP ///////////////////////////////////////////////");

        /// Liquidity of BNT and LINK in LINK Pool AFTER Swap
        (uint256 tradingLiquidityBntAfter, uint256 tradingLiquidityLinkAfter) = getTradingLiquidity(bnt, link);
        uint256 liquidityRatioAfter = calculateLiquidityRatio(10**18, tradingLiquidityLinkAfter, tradingLiquidityBntAfter);

        uint256 calculatedLiquidityLinkAfter = tradingLiquidityLinkBefore - amountOut;
        uint256 calculatedLiquidityBntAfter = tradingLiquidityBntBefore + (amountIn - tf.networkFeeAmount);

        uint256 calculatedLiquidityRatioAfter = 10**18 * calculatedLiquidityLinkAfter/calculatedLiquidityBntAfter;

        console.log("CALCULATED tradingLiquidity LINK after swap: ",calculatedLiquidityLinkAfter);
        console.log("CALCULATED tradingLiquidity BNT after swap: ",calculatedLiquidityBntAfter);
        console.log("CALCULATED liquidity ratio after swap: ", calculatedLiquidityRatioAfter);

        console.log("amountIn: ", amountIn);

        /// Check if tradingLiquidityLinkAfter == calculatedLiquidityLinkAfter
        assertEq(tradingLiquidityLinkAfter, calculatedLiquidityLinkAfter);

        /// Check if tradingLiquidityBntAfter == calculatedLiquidityBntAfter
        assertEq(tradingLiquidityBntAfter, calculatedLiquidityBntAfter);
        // assertLe(tradingLiquidityBntAfter, calculatedLiquidityBntAfter);

        /// Check if amountIn == amountInSimulated
        assertEq(amountIn, tf.amount);
        
    }

    /// Sell BNT for LINK
    /// All Tests Passed
    function testHowPriceWorksBntForLink () public {

        /// calculate swap price by dividing amountOut/amountIn of a swap
        /// Check if it corresponds to 
        /// divide the tradinqLiquidity afterSwap and see if it corresponds
        uint256 amountIn = 10 ether;

        /// Liquidity of BNT and LINK in LINK Pool BEFORE Swap
        (uint256 tradingLiquidityBntBefore, uint256 tradingLiquidityLinkBefore) = getTradingLiquidity(bnt, link);
        uint256 liquidityRatioBefore = calculateLiquidityRatio(10**18, tradingLiquidityLinkBefore, tradingLiquidityBntBefore);

        console.log("tradingLiquidity LINK before swap: ",tradingLiquidityLinkBefore);
        console.log("tradingLiquidity BNT before swap: ",tradingLiquidityBntBefore);
        console.log("liquidity ratio before swap: ", liquidityRatioBefore);


        console.log("######################################### Simulation of tradeOutputAndFeeBySourceAmount ###############");

        /// Use the function tradeOutputAndFeeBySourceAmount to get the amountOutSimulated and Fees
        /// Than check if amountOutSimulated is == to amountOut
        IBancorV3PoolCollection poolColl = adapter.bancorPoolCollection();
        IBancorV3PoolCollection.TradeAmountAndFee memory tf = poolColl.tradeOutputAndFeeBySourceAmount(bnt, link, amountIn);

        console.log("SIMULATED amountOut", tf.amount);
        console.log("SIMULATED tradingFee", tf.tradingFeeAmount);
        console.log("SIMULATED networkFee", tf.tradingFeeAmount);

        deal(address(BNT), address(this), amountIn);
        BNT.approve(address(adapter), amountIn);

        /// Sell BNT for LINK
        uint256 amountOut = adapter.swap(PAIR, BNT, LINK, OrderSide.Sell, amountIn).calculatedAmount;

        console.log("/////////////////////////////////////////////// AFTER SWAP ///////////////////////////////////////////////");

        /// Liquidity of BNT and LINK in LINK Pool AFTER Swap
        (uint256 tradingLiquidityBntAfter, uint256 tradingLiquidityLinkAfter) = getTradingLiquidity(bnt, link);
        uint256 liquidityRatioAfter = calculateLiquidityRatio(10**18, tradingLiquidityLinkAfter, tradingLiquidityBntAfter);
        
        /// Calculate LINK liquidity after swap --> tradingLiquidityLinkBefore - amountOut
        uint256 calculatedLiquidityLinkAfter = tradingLiquidityLinkBefore - amountOut;
        /// Calculate BNT liquidity after swap --> tradingLiquidityBntBefore + amountIn
        uint256 calculatedLiquidityBntAfter = tradingLiquidityBntBefore + (amountIn - tf.networkFeeAmount);

        uint256 calculatedLiquidityRatioAfter = 10**18 * calculatedLiquidityLinkAfter/calculatedLiquidityBntAfter;

        console.log("CALCULATED tradingLiquidity LINK after swap: ",calculatedLiquidityLinkAfter);
        console.log("CALCULATED tradingLiquidity BNT after swap: ",calculatedLiquidityBntAfter);
        console.log("CALCULATED liquidity ratio after swap: ", calculatedLiquidityRatioAfter);

        console.log("amountOut: ", amountOut);

        /// Check if tradingLiquidityLinkAfter == calculatedLiquidityLinkAfter
        assertEq(tradingLiquidityLinkAfter, calculatedLiquidityLinkAfter);

        /// Check if tradingLiquidityBntAfter == calculatedLiquidityBntAfter
        assertEq(tradingLiquidityBntAfter, calculatedLiquidityBntAfter);
        // assertLe(tradingLiquidityBntAfter, calculatedLiquidityBntAfter);

        /// Check if amountOut == amountOutSimulated
        assertEq(amountOut, tf.amount);
        
    }

    /// Sell WBTC for BNT
    /// All Tests Passed
    function testHowPriceWorksWbtcForBnt () public {

        uint256 amountIn = 10 ether;

        /// Liquidity of BNT and WBTC in WBTC Pool BEFORE Swap
        (uint256 tradingLiquidityWbtcBefore, uint256 tradingLiquidityBntBefore) = getTradingLiquidity(wbtc, bnt);
        uint256 liquidityRatioBefore = calculateLiquidityRatio(10**18, tradingLiquidityBntBefore, tradingLiquidityWbtcBefore);

        console.log("tradingLiquidity WBTC before swap: ",tradingLiquidityWbtcBefore);
        console.log("tradingLiquidity BNT before swap: ",tradingLiquidityBntBefore);
        console.log("liquidity ratio before swap: ", liquidityRatioBefore);


        console.log("######################################### Simulation of tradeOutputAndFeeBySourceAmount ###############");

        /// Use the function tradeOutputAndFeeBySourceAmount to get the amountOutSimulated and Fees
        /// Than check if amountOutSimulated is == to amountOut
        IBancorV3PoolCollection poolColl = adapter.bancorPoolCollection();
        IBancorV3PoolCollection.TradeAmountAndFee memory tf = poolColl.tradeOutputAndFeeBySourceAmount(wbtc, bnt, amountIn);

        console.log("SIMULATED amountOut", tf.amount);
        console.log("SIMULATED tradingFee", tf.tradingFeeAmount);
        console.log("SIMULATED networkFee", tf.tradingFeeAmount);

        deal(address(WBTC), address(this), amountIn);
        WBTC.approve(address(adapter), amountIn);

        /// Sell BNT for WBTC
        uint256 amountOut = adapter.swap(PAIR, WBTC, BNT, OrderSide.Sell, amountIn).calculatedAmount;

        console.log("/////////////////////////////////////////////// AFTER SWAP ///////////////////////////////////////////////");

        /// Liquidity of BNT and WBTC in WBTC Pool AFTER Swap
        (uint256 tradingLiquidityWbtcAfter, uint256 tradingLiquidityBntAfter) = getTradingLiquidity(wbtc, bnt);
        uint256 liquidityRatioAfter = calculateLiquidityRatio(10**18, tradingLiquidityBntAfter, tradingLiquidityWbtcAfter);

        console.log("tradingLiquidity WBTC after swap: ",tradingLiquidityWbtcAfter);
        console.log("tradingLiquidity BNT after swap: ",tradingLiquidityBntAfter);
        console.log("liquidity ratio after swap: ", liquidityRatioAfter);

        console.log("/////////////////////////////////////////////// CALCULATED AFTER SWAP ///////////////////////////////////////////////");
        
        /// Calculate WBTC liquidity after swap --> tradingLiquidityWbtcBefore - amountOut
        uint256 calculatedLiquidityWbtcAfter = tradingLiquidityWbtcBefore + amountIn;
        /// Calculate BNT liquidity after swap --> tradingLiquidityBntBefore + amountIn
        uint256 calculatedLiquidityBntAfter = tradingLiquidityBntBefore - (amountOut + tf.networkFeeAmount);

        uint256 calculatedLiquidityRatioAfter = 10**18 * calculatedLiquidityBntAfter/calculatedLiquidityWbtcAfter;

        console.log("CALCULATED tradingLiquidity WBTC after swap: ",calculatedLiquidityWbtcAfter);
        console.log("CALCULATED tradingLiquidity BNT after swap: ",calculatedLiquidityBntAfter);
        console.log("CALCULATED liquidity ratio after swap: ", calculatedLiquidityRatioAfter);

        console.log("amountOut: ", amountOut);

        /// Check if tradingLiquidityWbtcAfter == calculatedLiquidityWbtcAfter
        assertEq(tradingLiquidityWbtcAfter, calculatedLiquidityWbtcAfter);

        /// Check if tradingLiquidityBntAfter == calculatedLiquidityBntAfter
        assertEq(tradingLiquidityBntAfter, calculatedLiquidityBntAfter);

        /// Check if amountOut == amountOutSimulated
        assertEq(amountOut, tf.amount);
        
    }

    /// Sell BNT for WBTC
    /// All Tests Passed
    function testHowPriceWorksBntForWbtc () public {

        /// calculate swap price by dividing amountOut/amountIn of a swap
        /// Check if it corresponds to 
        /// divide the tradinqLiquidity afterSwap and see if it corresponds
        uint256 amountIn = 10 ether;

        /// Liquidity of BNT and WBTC in WBTC Pool BEFORE Swap
        (uint256 tradingLiquidityBntBefore, uint256 tradingLiquidityWbtcBefore) = getTradingLiquidity(bnt, wbtc);
        uint256 liquidityRatioBefore = calculateLiquidityRatio(10**18, tradingLiquidityWbtcBefore, tradingLiquidityBntBefore);

        console.log("tradingLiquidity WBTC before swap: ",tradingLiquidityWbtcBefore);
        console.log("tradingLiquidity BNT before swap: ",tradingLiquidityBntBefore);
        console.log("liquidity ratio before swap: ", liquidityRatioBefore);


        console.log("######################################### Simulation of tradeOutputAndFeeBySourceAmount ###############");

        /// Use the function tradeOutputAndFeeBySourceAmount to get the amountOutSimulated and Fees
        /// Than check if amountOutSimulated is == to amountOut
        IBancorV3PoolCollection poolColl = adapter.bancorPoolCollection();
        IBancorV3PoolCollection.TradeAmountAndFee memory tf = poolColl.tradeOutputAndFeeBySourceAmount(bnt, wbtc, amountIn);

        console.log("SIMULATED amountOut", tf.amount);
        console.log("SIMULATED tradingFee", tf.tradingFeeAmount);
        console.log("SIMULATED networkFee", tf.tradingFeeAmount);

        deal(address(BNT), address(this), amountIn);
        BNT.approve(address(adapter), amountIn);

        /// Sell BNT for WBTC
        uint256 amountOut = adapter.swap(PAIR, BNT, WBTC, OrderSide.Sell, amountIn).calculatedAmount;

        console.log("/////////////////////////////////////////////// AFTER SWAP ///////////////////////////////////////////////");

        /// Liquidity of BNT and WBTC in WBTC Pool AFTER Swap
        (uint256 tradingLiquidityBntAfter, uint256 tradingLiquidityWbtcAfter) = getTradingLiquidity(bnt, wbtc);
        uint256 liquidityRatioAfter = calculateLiquidityRatio(10**18, tradingLiquidityWbtcAfter, tradingLiquidityBntAfter);
        
        /// Calculate WBTC liquidity after swap --> tradingLiquidityWbtcBefore - amountOut
        uint256 calculatedLiquidityWbtcAfter = tradingLiquidityWbtcBefore - amountOut;
        /// Calculate BNT liquidity after swap --> tradingLiquidityBntBefore + amountIn
        uint256 calculatedLiquidityBntAfter = tradingLiquidityBntBefore + (amountIn - tf.networkFeeAmount);

        uint256 calculatedLiquidityRatioAfter = 10**18 * calculatedLiquidityWbtcAfter/calculatedLiquidityBntAfter;

        console.log("CALCULATED tradingLiquidity WBTC after swap: ",calculatedLiquidityWbtcAfter);
        console.log("CALCULATED tradingLiquidity BNT after swap: ",calculatedLiquidityBntAfter);
        console.log("CALCULATED liquidity ratio after swap: ", calculatedLiquidityRatioAfter);

        console.log("amountOut: ", amountOut);

        /// Check if tradingLiquidityWbtcAfter == calculatedLiquidityWbtcAfter
        assertEq(tradingLiquidityWbtcAfter, calculatedLiquidityWbtcAfter);

        /// Check if tradingLiquidityBntAfter == calculatedLiquidityBntAfter
        assertEq(tradingLiquidityBntAfter, calculatedLiquidityBntAfter);
        // assertLe(tradingLiquidityBntAfter, calculatedLiquidityBntAfter);

        /// Check if amountOut == amountOutSimulated
        assertEq(amountOut, tf.amount);
        
    }

    function testSwapFuzzBancorV3BntLink(uint256 specifiedAmount, bool isBuy) public {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;

        uint256[] memory limits = adapter.getLimits(PAIR, BNT, LINK);

        vm.assume(specifiedAmount > 1000);

        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1]);

            deal(address(BNT), address(this), type(uint256).max);
            BNT.approve(address(adapter), type(uint256).max);
        } else {
            vm.assume(specifiedAmount < limits[0]);

            deal(address(BNT), address(this), specifiedAmount);
            BNT.approve(address(adapter), specifiedAmount);
        }

        uint256 bnt_balance_before_swap = BNT.balanceOf(address(this));
        uint256 link_balance_before_swap = LINK.balanceOf(address(this));

        Trade memory trade =
            adapter.swap(PAIR, BNT, LINK, side, specifiedAmount);

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    LINK.balanceOf(address(this)) - link_balance_before_swap
                );
                assertEq(
                    trade.calculatedAmount,
                    bnt_balance_before_swap - BNT.balanceOf(address(this))
                );
            } else {
                assertEq(
                    specifiedAmount,
                    bnt_balance_before_swap - BNT.balanceOf(address(this))
                );
                assertEq(
                    trade.calculatedAmount,
                    LINK.balanceOf(address(this)) - link_balance_before_swap
                );
            }
        }
    }

    function testSwapSellIncreasingBancorV3() public {
        executeIncreasingSwaps(OrderSide.Sell);
    }

    function executeIncreasingSwaps(OrderSide side) internal {

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * (i) * 10 ** 18;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(address(BNT), address(this), type(uint256).max);
            BNT.approve(address(adapter), type(uint256).max);

            trades[i] = adapter.swap(PAIR, BNT, LINK, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
            // assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    function testSwapBuyIncreasingBancorV3() public {
        executeIncreasingSwaps(OrderSide.Buy);
    }

    function testGetPoolIdsBancorV3() public view {
        bytes32[] memory ids = adapter.getPoolIds(1, 20);
        console.log(ids.length);
        console.logBytes32(ids[1]);
    }

    function testGetLimitsBancorV3() public {
        uint256[] memory limits = adapter.getLimits(bytes32(0), BNT, LINK);
        assertEq(limits.length, 2);

        limits = adapter.getLimits(bytes32(0), ETH, BNT);
        assertEq(limits.length, 2);

        limits = adapter.getLimits(bytes32(0), LINK, ETH);
        assertEq(limits.length, 2);

        limits = adapter.getLimits(bytes32(0), ETH, LINK);
        assertEq(limits.length, 2);
    }

    // function calculateTradingLiquidityAfterSwapSell(IERC20 sellToken_, IERC20 buyToken_, uint256 _amountIn)
    // internal
    // returns (uint256 tradingLiquiditySellTokenAfter, uint256 tradingLiquidityBuyTokenAfter, uint256 calculatedTradingLiquiditySellTokenAfter, uint256 calculatedTradingLiquidityBuyTokenAfter ) 
    // {   
    //     Token sellToken = Token(address(sellToken_));
    //     Token buyToken = Token(address(buyToken_));

    //     IBancorV3PoolCollection poolColl = adapter.bancorPoolCollection();
    //     IBancorV3PoolCollection.TradeAmountAndFee memory tf = poolColl.tradeOutputAndFeeBySourceAmount(sellToken, buyToken, _amountIn);

    //     (uint256 tradingLiquiditySellTokenBefore, uint256 tradingLiquidityBuyTokenBefore) = getTradingLiquidity(sellToken, buyToken);

    //     deal(address(sellToken_), address(this), _amountIn);
    //     sellToken_.approve(address(adapter), _amountIn);

    //     uint256 amountOut = adapter.swap(PAIR, sellToken_, buyToken_, OrderSide.Sell, _amountIn).calculatedAmount;

    //     (tradingLiquiditySellTokenAfter, tradingLiquidityBuyTokenAfter) = getTradingLiquidity(sellToken, buyToken);

    //     if (sellToken == bnt) {
            
    //     calculatedTradingLiquiditySellTokenAfter = tradingLiquiditySellTokenBefore + (_amountIn - tf.networkFeeAmount);
    //     calculatedTradingLiquidityBuyTokenAfter = tradingLiquidityBuyTokenBefore - amountOut;

    //     } else if (buyToken == bnt) {

    //     calculatedTradingLiquiditySellTokenAfter = tradingLiquiditySellTokenBefore + _amountIn;
    //     calculatedTradingLiquidityBuyTokenAfter = tradingLiquidityBuyTokenBefore - (amountOut - tf.networkFeeAmount);

    //     } else {

    //         revert NotImplemented("TemplateSwapAdapter.price");

    //     }

    // }

    // function testTradingLiquidityAfterSwapSellBntForLink () public {

    //     uint256 amountIn = 10 ether;

    //     (uint256 tradingLiquidityBntAfter, uint256 tradingLiquidityLinkAfter, 
    //     uint256 calculatedTradingLiquidityBntAfter, uint256 calculatedTradingLiquidityLinkAfter) 
    //     = calculateTradingLiquidityAfterSwapSell(BNT, LINK, amountIn);

    //     assertEq(tradingLiquidityLinkAfter, calculatedTradingLiquidityLinkAfter);

    //     assertEq(tradingLiquidityBntAfter, calculatedTradingLiquidityBntAfter);

    // }
}