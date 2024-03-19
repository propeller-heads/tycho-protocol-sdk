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
        uint256 forkBlock = 19332669;
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

    function testHowPriceWorks () public {

        /// calculate swap price by dividing amountOut/amountIn of a swap
        /// Check if it corresponds to 
        /// divide the tradinqLiquidity afterSwap and see if it corresponds
        uint256 amountIn = 10 ether;

        /// Liquidity of BNT and LINK in LINK Pool BEFORE Swap
        (uint256 tradingLiquidityBntBefore, uint256 tradingLiquidityLinkBefore) = getTradingLiquidity(bnt, link);
        uint256 liquidityRatioBefore = 10**18 * tradingLiquidityLinkBefore/tradingLiquidityBntBefore;

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
        uint256 liquidityRatioAfter = 10**18 * tradingLiquidityLinkAfter/tradingLiquidityBntAfter;

        console.log("tradingLiquidity LINK after swap: ",tradingLiquidityLinkAfter);
        console.log("tradingLiquidity BNT after swap: ",tradingLiquidityBntAfter);
        console.log("liquidity ratio after swap: ", liquidityRatioAfter);

        console.log("/////////////////////////////////////////////// CALCULATED AFTER SWAP ///////////////////////////////////////////////");
        
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

}