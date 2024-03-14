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
    IERC20 constant ENJ = IERC20(0xF629cBd94d3791C9250152BD8dfBDF380E2a3B9c);
    

    function setUp() public {
        uint256 forkBlock = 19432669;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);

        adapter = new BancorV3SwapAdapter(BANCOR_NETWORK_PROXY_ADDRESS, BANCOR_NETWORK_INFO_PROXY_ADDRESS, POOL_COLLECTION_ADDRESS);
    }

    function testGetPoolIdsBancor() public {
        bytes32[] memory ids = adapter.getPoolIds(1, 20);

        console.log(ids.length);
        console.logBytes32(ids[1]);

    }

    function testGetLimitsBntEthBancor() public {
        uint256[] memory limits = adapter.getLimits(bytes32(0), BNT, ETH);
        Token eth = Token(address(ETH));
        TradingLiquidity memory tradingLiquidityEthPool = adapter.getTradingLiquidity(eth);
        uint256 tradingLiquidityEth = uint256(tradingLiquidityEthPool.baseTokenTradingLiquidity);
        uint256 tradingLiquidityBnt = uint256(tradingLiquidityEthPool.bntTradingLiquidity);
        console.log("BNT Limit: ", limits[0]);
        console.log("BNT tradingLiquidity: ", tradingLiquidityBnt);
        console.log("ETH Limit", limits[1]);
        console.log("ETH tradingLiquidity: ", tradingLiquidityEth);
        assertEq(limits.length, 2);
    }

    function testGetLimitsEthBntBancor() public {
        uint256[] memory limits = adapter.getLimits(bytes32(0), ETH, BNT);
        Token eth = Token(address(ETH));
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
        Token eth = Token(address(ETH));
        Token link = Token(address(LINK));
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


}