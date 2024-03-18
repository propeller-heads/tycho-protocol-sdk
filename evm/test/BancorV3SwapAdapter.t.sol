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
            amounts[i] = 1000 * (i + 1) * 10 ** 6;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(address(BNT), address(this), amounts[i]);
            BNT.approve(address(adapter), amounts[i]);

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


    function testGetPoolIdsBancor() public {
        bytes32[] memory ids = adapter.getPoolIds(1, 20);
        console.log(ids.length);
        console.logBytes32(ids[1]);
    }

    function testGetLimitsBancorV3BntLink() public {
        uint256[] memory limits = adapter.getLimits(bytes32(0), BNT, LINK);
        TradingLiquidity memory tradingLiquidityLinkPool = adapter.getTradingLiquidity(link);
        uint256 tradingLiquidityLink = uint256(tradingLiquidityLinkPool.baseTokenTradingLiquidity);
        uint256 tradingLiquidityBnt = uint256(tradingLiquidityLinkPool.bntTradingLiquidity);
        console.log("BNT Limit: ", limits[0]);
        console.log("BNT tradingLiquidity: ", tradingLiquidityBnt);
        console.log("LINK Limit", limits[1]);
        console.log("LINK tradingLiquidity: ", tradingLiquidityLink);
        assertEq(limits.length, 2);
    }

    function testGetLimitsEthBntBancor() public {
        uint256[] memory limits = adapter.getLimits(bytes32(0), ETH, BNT);
        TradingLiquidity memory tradingLiquidityEthPool = adapter.getTradingLiquidity(eth);
        uint256 tradingLiquidityEth = uint256(tradingLiquidityEthPool.baseTokenTradingLiquidity);
        uint256 tradingLiquidityBnt = uint256(tradingLiquidityEthPool.bntTradingLiquidity);
        console.log("ETH Limit", limits[0]);
        console.log("ETH tradingLiquidity: ", tradingLiquidityEth);
        console.log("BNT Limit: ", limits[1]);
        console.log("BNT tradingLiquidity: ", tradingLiquidityBnt);
        assertEq(limits.length, 2);
    }

    function testGetLimitsLinkEthBancor() public {
        uint256[] memory limits = adapter.getLimits(bytes32(0), LINK, ETH);
        console.log("LINK Limit", limits[0]);
        console.log("ETH Limit", limits[1]);
        assertEq(limits.length, 2);
    }

    function testGetLimitsEthLinkBancor() public {
        uint256[] memory limits = adapter.getLimits(bytes32(0), ETH, LINK);
        TradingLiquidity memory tradingLiquidityEthPool = adapter.getTradingLiquidity(eth);
        TradingLiquidity memory tradingLiquidityLinkPool = adapter.getTradingLiquidity(link);
        uint256 tradingLiquidityEth = uint256(tradingLiquidityEthPool.baseTokenTradingLiquidity);
        uint256 tradingLiquidityLink = uint256(tradingLiquidityLinkPool.baseTokenTradingLiquidity);
        
        console.log("ETH Limit", limits[0]);
        console.log("ETH tradingLiquidity: ", tradingLiquidityEth, tradingLiquidityEthPool.bntTradingLiquidity);
        console.log("LINK Limit", limits[1]);
        console.log("LINK tradingLiquidity: ", tradingLiquidityLink);
        assertEq(limits.length, 2);
    }

    function testPricePippo() public {
        IBancorV3BancorNetwork network = adapter.bancorNetwork();
        IBancorV3BancorNetworkInfo networkInfo = adapter.bancorNetworkInfo();

        deal(address(BNT), address(this), 10000 ether);
        BNT.approve(address(network), 10000 ether);

        uint256 amountOutInfo = networkInfo.tradeOutputBySourceAmount(bnt, eth, 100 ether);

        uint256 amountOut = 
        network.tradeBySourceAmount(
            bnt,
            link,
            100 ether,
            1,
            block.timestamp + 300,
            address(this)
        );

        console.log("AmountOut Info :", amountOutInfo);
        console.log("AmountOut :", amountOut);

    }


    function testSwapSellBntForLink() public {

        uint256 amountIn = 10000 ether;

        deal(address(BNT), address(this), amountIn);
        BNT.approve(address(adapter), amountIn);

        uint256 bnt_balance_before_swap = BNT.balanceOf(address(this));

        console.log("Address initializing swap: ", address(this));
        console.log("BNT in address(this) : ", bnt_balance_before_swap);

        uint256 amountOut = adapter.swap(PAIR, BNT, LINK, OrderSide.Sell, amountIn).calculatedAmount;
    }

    function testSwapSellBntForEth() public {

        uint256 amountIn = 10000 ether;

        deal(address(BNT), address(this), amountIn);
        BNT.approve(address(adapter), amountIn);

        uint256 bnt_balance_before_swap = BNT.balanceOf(address(this));

        console.log("Address initializing swap: ", address(this));
        console.log("BNT in address(this) : ", bnt_balance_before_swap);

        uint256 amountOut = adapter.swap(PAIR, BNT, ETH, OrderSide.Sell, amountIn).calculatedAmount;
    }

    function testSwapSellLinkForEth() public {

        uint256 amountIn = 10000 ether;

        deal(address(LINK), address(this), amountIn);
        LINK.approve(address(adapter), amountIn);

        uint256 link_balance_before_swap = LINK.balanceOf(address(this));
        uint256 eth_balance_before_swap = address(this).balance;
        
        console.log("LINK in address(this) : ", link_balance_before_swap);
        console.log("ETH in address(this) : ", eth_balance_before_swap);

        uint256 amountOut = adapter.swap(PAIR, LINK, ETH, OrderSide.Sell, amountIn).calculatedAmount;

        uint256 link_balance_after_swap = LINK.balanceOf(address(this));
        uint256 eth_balance_after_swap = address(this).balance;
        console.log("LINK in address(this) : ", link_balance_after_swap);
        console.log("ETH in address(this) : ", eth_balance_after_swap);

    }









}