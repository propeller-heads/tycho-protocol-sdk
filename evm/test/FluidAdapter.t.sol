// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {AdapterTest} from "./AdapterTest.sol";
import "forge-std/console.sol";
import {IERC20} from "lib/forge-std/src/interfaces/IERC20.sol";
import {FluidAdapter} from "src/fluid/FluidAdapter.sol";
import {FluidSwapExecutor} from "src/fluid/FluidSwapExecutor.sol";
import {ISwapAdapterTypes} from "src/interfaces/ISwapAdapterTypes.sol";
import {FractionMath} from "src/libraries/FractionMath.sol";
import {FluidDexReservesResolver} from "src/fluid/Interfaces/FluidInterfaces.sol";
import {Structs} from "src/fluid/Interfaces/structs.sol";
import {IFluidDexT1} from "src/fluid/Interfaces/iDexT1.sol";

contract FluidAdapterTest is AdapterTest {
    using FractionMath for Fraction;

    FluidAdapter adapter;
    FluidSwapExecutor executor;
    FluidDexReservesResolver resolver;
    bytes32 poolId1;
    bytes32 poolId2;
    address pool2Token0;
    address pool2Token1;
    address pool1Token0;
    address pool1Token1;
    address pool1Address;
    address pool2Address;


    function setUp() public {

        uint256 mainnetFork = vm.createFork("https://eth.llamarpc.com", 21422012);
        vm.selectFork(mainnetFork);

        // Create a mock resolver
        resolver = FluidDexReservesResolver(0x45f4Ad57e300DA55C33DEa579A40FCeE000d7B94);
        
        // Deploy FluidAdapter with mock resolver
        adapter = new FluidAdapter(address(resolver));
        executor = new FluidSwapExecutor(address(resolver));

        poolId1 = bytes32(abi.encode(1)); // wstETH/Eth
        poolId2 = bytes32(abi.encode(2)); // USDC/USDT

        pool1Address = resolver.getPoolAddress(uint256(poolId1));
        pool2Address = resolver.getPoolAddress(uint256(poolId2));
        (pool1Token0, pool1Token1) = resolver.getPoolTokens(pool1Address);
        (pool2Token0, pool2Token1) = resolver.getPoolTokens(pool2Address);
        console.log();
    }

    function test_price() public {
        uint256[] memory amounts = new uint256[](1);
        amounts[0] = 1e6;
        Fraction[] memory prices = adapter.price(bytes32(abi.encode(2)), 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48, 0xdAC17F958D2ee523a2206206994597C13D831ec7, amounts);
        assert(prices[0].numerator == 999916);
        assert(prices[0].denominator == 1);

        prices = adapter.price(bytes32(abi.encode(2)), 0xdAC17F958D2ee523a2206206994597C13D831ec7, 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48, amounts);
        assert(prices[0].numerator == 999883);
        assert(prices[0].denominator == 1);

        amounts[0] = 2e6;
        prices = adapter.price(bytes32(abi.encode(2)), 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48, 0xdAC17F958D2ee523a2206206994597C13D831ec7, amounts);
        assert(prices[0].numerator == 1999833);
        assert(prices[0].denominator == 1);

        amounts[0] = 2e15;
        prices = adapter.price(bytes32(abi.encode(1)), 0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0, 0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE, amounts);
        assert(prices[0].numerator == 2374661202000000);
        assert(prices[0].denominator == 1);

        amounts[0] = 2e15;
        prices = adapter.price(bytes32(abi.encode(1)), 0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE, 0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0, amounts);
        assert(prices[0].numerator == 1684113940000000);
        assert(prices[0].denominator == 1);


    }

    function test_swap() public {
        address testUser = makeAddr("testUser");

        vm.prank(0x37305B1cD40574E4C5Ce33f8e8306Be057fD7341);   // usdc whale
        IERC20(pool2Token0).transfer(testUser, 1e10);

        vm.startPrank(testUser);
        IERC20(pool2Token0).approve(address(adapter), 1e6);

        Trade memory trade = adapter.swap(poolId2, pool2Token0, pool2Token1, OrderSide.Sell, 1e6);
        assert(trade.calculatedAmount == 999916);

        vm.stopPrank();

        vm.prank(0x5313b39bf226ced2332C81eB97BB28c6fD50d1a3);   // wsteth whale
        IERC20(pool1Token0).transfer(testUser, 2e15);

        vm.deal(testUser, 1e18);
        vm.startPrank(testUser);
        bytes memory data = abi.encode(poolId1, pool1Token0, pool1Token1, true);

        IERC20(pool1Token0).approve(address(executor), 2e15);

        uint256 calculatedAmount = executor.swap(2e15, data);
        assert(calculatedAmount == 2374661202000000);

        data = abi.encode(poolId1, pool1Token1, pool1Token0, true);
        calculatedAmount = executor.swap{value : 1e18}(1e18, data);
        assert(calculatedAmount == 842056960368000000);
        vm.stopPrank();
    }

    function test_getLimits() public {
        uint256[] memory limits = adapter.getLimits(
            bytes32(abi.encode(2)), 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48, 0xdAC17F958D2ee523a2206206994597C13D831ec7
        );

        uint256[] memory amounts = new uint256[](1);
        amounts[0] = 5000000000000;
        console.log(adapter.price(bytes32(abi.encode(2)), 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48, 0xdAC17F958D2ee523a2206206994597C13D831ec7, amounts)[0].numerator);

        amounts[0] = limits[0] + 1;
        assert(adapter.price(bytes32(abi.encode(2)), 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48, 0xdAC17F958D2ee523a2206206994597C13D831ec7, amounts)[0].numerator == 0);

        console.log(limits.length);
        for(uint i; i < limits.length; i++){
            console.log(limits[i]);
        }
    }

    function test_getCapabilities(bytes32 a, address b, address c) public {
        Capability[] memory capabilities = adapter.getCapabilities(a, b, c);
        assert(capabilities.length == 4);
        assert(capabilities[0] == Capability.SellOrder);
        assert(capabilities[1] == Capability.BuyOrder);
        assert(capabilities[2] == Capability.PriceFunction);
        assert(capabilities[3] == Capability.HardLimits);
    }

    function test_getTokens() public {
        address[] memory pool1Tokens = adapter.getTokens(poolId1);
        assert(pool1Tokens[0] == pool1Token0);
        assert(pool1Tokens[1] == pool1Token1);

        address[] memory pool2Tokens = adapter.getTokens(poolId2);
        assert(pool1Tokens[0] == pool2Token0);
        assert(pool1Tokens[1] == pool2Token1);
    }

    function test_poolIds() public {
        bytes32[] memory poolIds = adapter.getPoolIds(1, 5);
        assert(poolIds.length == 5);
        assert(poolIds[2] == bytes32(abi.encode(3)));
    }

}
