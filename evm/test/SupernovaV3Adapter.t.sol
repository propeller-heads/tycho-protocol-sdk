// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {SupernovaV3Adapter} from "src/supernova-v3/SupernovaV3Adapter.sol";
import {IAlgebraFactory, IAlgebraPool} from "src/supernova-v3/IAlgebra.sol";
import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {ISwapAdapterTypes} from "src/interfaces/ISwapAdapterTypes.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";

contract SupernovaV3AdapterTest is Test {
    SupernovaV3Adapter public adapter;
    address public constant FACTORY = address(bytes20(hex"44B7fBd4D87149eFa5347c451E74B9FD18E89c55"));
    address public constant POOL = address(bytes20(hex"2beb35e78C9427899353c41C96bCc96C5647ec63"));
    bytes32 public poolId;

    function setUp() public {
        string memory rpcUrl = vm.envOr("ETH_RPC_URL", string("https://eth.llamarpc.com"));
        // Fork at the block provided by the user
        vm.createSelectFork(rpcUrl, 24768374);
        
        adapter = new SupernovaV3Adapter(FACTORY);
        poolId = bytes32(uint256(uint160(POOL)) << 96);
    }

    function test_GetTokens() public {
        address[] memory tokens = adapter.getTokens(poolId);
        assertEq(tokens.length, 2);
        address t0 = IAlgebraPool(POOL).token0();
        address t1 = IAlgebraPool(POOL).token1();
        assertEq(tokens[0], t0);
        assertEq(tokens[1], t1);
    }

    function test_PriceAtZero() public {
        uint256[] memory amounts = new uint256[](1);
        amounts[0] = 0;
        address token0 = IAlgebraPool(POOL).token0();
        address token1 = IAlgebraPool(POOL).token1();
        ISwapAdapterTypes.Fraction[] memory prices = adapter.price(poolId, token0, token1, amounts);
        assertGt(prices[0].numerator, 0);
        assertGt(prices[0].denominator, 0);
    }

    function test_SwapSell_At_24747824() public {
        address token0 = IAlgebraPool(POOL).token0();
        address token1 = IAlgebraPool(POOL).token1();
        
        // Let's simulate a swap of 100 USDC (token0)
        uint256 amountIn = 100 * 1e6; 
        
        deal(token0, address(this), amountIn);
        IERC20(token0).approve(address(adapter), amountIn);
        
        ISwapAdapterTypes.Trade memory trade = adapter.swap(
            poolId,
            token0,
            token1,
            ISwapAdapterTypes.OrderSide.Sell,
            amountIn
        );
        
        emit log_named_uint("Simulated Amount Out", trade.calculatedAmount);
        emit log_named_uint("Gas Used", trade.gasUsed);
        
        assertGt(trade.calculatedAmount, 0);
        assertGt(trade.gasUsed, 0);
    }

    function test_SwapBuy_At_24747824() public {
        address token0 = IAlgebraPool(POOL).token0();
        address token1 = IAlgebraPool(POOL).token1();
        
        // Exact output of 100 USDT (token1)
        uint256 amountOut = 100 * 1e6; 
        
        deal(token0, address(this), 200 * 1e6); 
        IERC20(token0).approve(address(adapter), 200 * 1e6);
        
        ISwapAdapterTypes.Trade memory trade = adapter.swap(
            poolId,
            token0,
            token1,
            ISwapAdapterTypes.OrderSide.Buy,
            amountOut
        );
        
        emit log_named_uint("Simulated Amount In", trade.calculatedAmount);
        emit log_named_uint("Gas Used", trade.gasUsed);
        
        assertGt(trade.calculatedAmount, 0);
        assertGt(trade.gasUsed, 0);
    }

    function test_Simulate_100_USDT_to_Usdc() public {
        address usdt = IAlgebraPool(POOL).token1(); 
        address usdc = IAlgebraPool(POOL).token0();
        
        uint256 amountIn = 100 * 1e6; // 100 USDT (6 decimals)
        
        deal(usdt, address(this), amountIn);
        (bool s,) = usdt.call(abi.encodeWithSignature("approve(address,uint256)", address(adapter), amountIn));
        require(s, "USDT approve failed");
        
        ISwapAdapterTypes.Trade memory trade = adapter.swap(
            poolId,
            usdt,
            usdc,
            ISwapAdapterTypes.OrderSide.Sell,
            amountIn
        );
        
        emit log_named_uint("USDT Amount In", amountIn);
        emit log_named_uint("Simulated USDC Amount Out", trade.calculatedAmount);
        emit log_named_uint("Gas Used", trade.gasUsed);
        
        assertGt(trade.calculatedAmount, 0);
    }
}
