// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.26;

import "./AdapterTest.sol";
import {BalancerV3Errors} from "src/balancer-v3/lib/BalancerV3Errors.sol";
import {
    BalancerV3SwapAdapter,
    IERC20,
    IVault,
    IBatchRouter,
    IERC4626,
    IPermit2
} from "src/balancer-v3/BalancerV3SwapAdapter.sol";
import {ERC20} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";

import {FractionMath} from "src/libraries/FractionMath.sol";

contract BalancerV3SwapAdapterTest is AdapterTest, ERC20, BalancerV3Errors {
    /// @notice Thrown when too many nonces are invalidated.
    error ExcessiveInvalidation();
    error WeightedPoolBptRateUnsupported();

    using FractionMath for Fraction;

    // because this value also applies to out tokens and is used in checks
    uint256 _MINIMUM_WRAP_AMOUNT = 10001;

    IVault constant balancerV3Vault =
        IVault(payable(0xbA1333333333a1BA1108E8412f11850A5C319bA9));
    BalancerV3SwapAdapter adapter;
    IBatchRouter router =
        IBatchRouter(0x136f1EFcC3f8f88516B9E94110D56FDBfB1778d1); // Batch router
    address constant permit2 = 0x000000000022D473030F116dDEE9F6B43aC78BA3;

    // ERC20
    address constant ERC20_ETHx_waWETH_POOL_ADDRESS =
        0x4AB7aB316D43345009B2140e0580B072eEc7DF16;
    address constant ERC20_waWETH = 0x0bfc9d54Fc184518A81162F8fB99c2eACa081202;
    address constant ERC20_ETHx = 0xA35b1B31Ce002FBF2058D22F30f95D405200A15b;

    // ERC20
    address constant ERC20_WEIGHTED_GOETH_USDC_POOL_ADDRESS =
        0xf91c11BA4220b7a72E1dc5E92f2b48D3fdF62726;
    address constant ERC20_GOETH = 0x440017A1b021006d556d7fc06A54c32E42Eb745B;
    address constant ERC20_USDC = 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48;

    uint256 constant TEST_ITERATIONS = 100;

    constructor() ERC20("", "") {}

    function setUp() public {
        uint256 forkBlock = 21421638;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);

        adapter = new BalancerV3SwapAdapter(
            payable(address(balancerV3Vault)), address(router), permit2
        );

        vm.label(address(balancerV3Vault), "BalancerV3Vault");
        vm.label(address(router), "BalancerV3BatchRouter");
        vm.label(address(adapter), "BalancerV3SwapAdapter");
        vm.label(ERC20_waWETH, "ERC20_waWETH");
        vm.label(ERC20_ETHx, "ERC20_ETHx");
        vm.label(
            ERC20_ETHx_waWETH_POOL_ADDRESS, "ERC20_ETHx_waWETH_POOL_ADDRESS"
        );
        vm.label(
            ERC20_WEIGHTED_GOETH_USDC_POOL_ADDRESS,
            "ERC20_WEIGHTED_GOETH_USDC_POOL_ADDRESS"
        );
        vm.label(ERC20_GOETH, "ERC20_GOETH");
        vm.label(ERC20_USDC, "ERC20_USDC");
        vm.label(permit2, "Permit2");
    }

    function testPriceFuzzErc20BalancerV3(uint256 amount0, uint256 amount1)
        public
    {
        address token0 = ERC20_waWETH;
        address token1 = ERC20_ETHx;

        bytes32 pool = bytes32(bytes20(ERC20_ETHx_waWETH_POOL_ADDRESS));
        uint256[] memory limits = adapter.getLimits(pool, token0, token1);

        vm.assume(amount0 < limits[0] && amount0 > getMinTradeAmount(token0));
        vm.assume(amount1 < limits[1] && amount1 > getMinTradeAmount(token0));

        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;

        __prankStaticCall();
        Fraction[] memory prices = adapter.price(pool, token0, token1, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testSwapFuzzBalancerV3_ERC20_ERC20(
        uint256 specifiedAmount,
        bool isBuy
    ) public {
        address token0 = ERC20_waWETH;
        address token1 = ERC20_ETHx;

        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        bytes32 pool = bytes32(bytes20(ERC20_ETHx_waWETH_POOL_ADDRESS));
        uint256[] memory limits = adapter.getLimits(pool, token0, token1);

        if (side == OrderSide.Buy) {
            vm.assume(
                specifiedAmount < limits[1]
                    && specifiedAmount > getMinTradeAmount(token0)
            );
        } else {
            vm.assume(
                specifiedAmount < limits[0]
                    && specifiedAmount > getMinTradeAmount(token0)
            );
        }

        deal(token0, address(this), type(uint256).max);
        IERC20(token0).approve(address(adapter), type(uint256).max);

        uint256 bal0 = IERC20(token0).balanceOf(address(this));
        uint256 bal1 = IERC20(token1).balanceOf(address(this));

        uint256[] memory amounts = new uint256[](1);
        amounts[0] = specifiedAmount;
        Trade memory trade =
            adapter.swap(pool, token0, token1, side, specifiedAmount);

        if (side == OrderSide.Buy) {
            assertEq(
                specifiedAmount, IERC20(token1).balanceOf(address(this)) - bal1
            );
            assertEq(
                trade.calculatedAmount,
                bal0 - IERC20(token0).balanceOf(address(this))
            );
        } else {
            assertEq(
                specifiedAmount, bal0 - IERC20(token0).balanceOf(address(this))
            );
            assertEq(
                trade.calculatedAmount,
                IERC20(token1).balanceOf(address(this)) - bal1
            );
        }
    }

    function testPriceFuzzErc20WeightedBalancerV3(uint256 amount0) public {
        address token0 = ERC20_GOETH;
        address token1 = ERC20_USDC;

        bytes32 pool = bytes32(bytes20(ERC20_WEIGHTED_GOETH_USDC_POOL_ADDRESS));
        uint256[] memory limits = adapter.getLimits(pool, token0, token1);

        vm.assume(amount0 < limits[0] && amount0 > getMinTradeAmount(token0));

        uint256[] memory amounts = new uint256[](1);
        amounts[0] = amount0;

        __prankStaticCall();
        Fraction[] memory prices = adapter.price(pool, token0, token1, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            // assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testSwapFuzzWeightedBalancerV3_ERC20_ERC20(
        uint256 specifiedAmount,
        bool isBuy
    ) public {
        address token0 = ERC20_GOETH;
        address token1 = ERC20_USDC;

        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        bytes32 pool = bytes32(bytes20(ERC20_WEIGHTED_GOETH_USDC_POOL_ADDRESS));
        uint256[] memory limits = adapter.getLimits(pool, token0, token1);

        if (side == OrderSide.Buy) {
            vm.assume(
                specifiedAmount < limits[1]
                    && specifiedAmount > getMinTradeAmount(token1)
            );
        } else {
            vm.assume(
                specifiedAmount < limits[0]
                    && specifiedAmount > getMinTradeAmount(token0)
            );
        }

        deal(token0, address(this), type(uint256).max);
        IERC20(token0).approve(address(adapter), type(uint256).max);

        uint256 bal0 = IERC20(token0).balanceOf(address(this));
        uint256 bal1 = IERC20(token1).balanceOf(address(this));

        uint256[] memory amounts = new uint256[](1);
        amounts[0] = specifiedAmount;
        Trade memory trade =
            adapter.swap(pool, token0, token1, side, specifiedAmount);

        if (side == OrderSide.Buy) {
            assertEq(
                specifiedAmount, IERC20(token1).balanceOf(address(this)) - bal1
            );
            assertEq(
                trade.calculatedAmount,
                bal0 - IERC20(token0).balanceOf(address(this))
            );
        } else {
            assertEq(
                specifiedAmount, bal0 - IERC20(token0).balanceOf(address(this))
            );
            assertEq(
                trade.calculatedAmount,
                IERC20(token1).balanceOf(address(this)) - bal1
            );
        }
    }

    function testLimitsErc20WeightedBalancerV3() public {
        address token0 = ERC20_USDC;
        address token1 = ERC20_GOETH;
        bytes32 pool = bytes32(bytes20(ERC20_WEIGHTED_GOETH_USDC_POOL_ADDRESS));
        uint256[] memory limits = adapter.getLimits(pool, token0, token1);
        console.log("limits", limits[0], limits[1]);

        deal(token0, address(this), type(uint256).max);
        IERC20(token0).approve(address(adapter), type(uint256).max);
    }

    function testSwapErc20WeightedBalancerV3() public {
        address token0 = ERC20_GOETH;
        address token1 = ERC20_USDC;

        bytes32 pool = bytes32(bytes20(ERC20_WEIGHTED_GOETH_USDC_POOL_ADDRESS));

        uint256[] memory limits = adapter.getLimits(pool, token0, token1);
        console.log("limits", limits[0], limits[1]);

        deal(token0, address(this), type(uint256).max);
        IERC20(token0).approve(address(adapter), type(uint256).max);

        Trade memory trade =
            adapter.swap(pool, token0, token1, OrderSide.Sell, 1e17);

        console.log("trade", trade.calculatedAmount);
    }

    function __prankStaticCall() internal {
        // Prank address 0x0 for both msg.sender and tx.origin (to identify as a
        // staticcall).
        vm.prank(address(0), address(0));
    }

    function getMinTradeAmount(address token) internal view returns (uint256) {
        uint256 decimals = ERC20(token).decimals();
        uint256 decimalFactor = decimals;
        if (decimals > 6) {
            decimalFactor = decimals - 1;
        }
        uint256 minTradeAmount = 10 ** decimalFactor;

        console.log("Min trade amount", minTradeAmount);
        return minTradeAmount;
    }
}
