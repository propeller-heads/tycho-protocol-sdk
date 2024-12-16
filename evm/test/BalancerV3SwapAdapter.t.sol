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
import "forge-std/Test.sol";

contract BalancerV3SwapAdapterTest is AdapterTest, ERC20, BalancerV3Errors {
    /// @notice Thrown when too many nonces are invalidated.
    error ExcessiveInvalidation();

    using FractionMath for Fraction;

    // minimum amounts from balancer
    uint256 _MINIMUM_TRADE_AMOUNT = (1000000) * 2; // balancer's minimum * 2
    // because this value also applies to out tokens and is used in checks
    uint256 _MINIMUM_WRAP_AMOUNT = 10001;

    IVault constant balancerV3Vault =
        IVault(payable(0xbA1333333333a1BA1108E8412f11850A5C319bA9));
    BalancerV3SwapAdapter adapter;
    IBatchRouter router =
        IBatchRouter(0x136f1EFcC3f8f88516B9E94110D56FDBfB1778d1); // Batch router
    address constant permit2 = 0x000000000022D473030F116dDEE9F6B43aC78BA3;

    // ERC20
    address constant USDC_USDT_POOL_ADDRESS =
        0x89BB794097234E5E930446C0CeC0ea66b35D7570;
    address constant USDC = 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48;
    address constant USDT = 0xdAC17F958D2ee523a2206206994597C13D831ec7;

    address constant waEthUSDT = 0x7Bc3485026Ac48b6cf9BaF0A377477Fff5703Af8;
    address constant waEthUSDC = 0xD4fa2D31b7968E448877f69A96DE69f5de8cD23E;

    // ERC20
    address constant GOETH_USDC_POOL_ADDRESS =
        0xf91c11BA4220b7a72E1dc5E92f2b48D3fdF62726;
    address constant GOETH = 0x440017A1b021006d556d7fc06A54c32E42Eb745B;
    

    uint256 constant TEST_ITERATIONS = 100;

    constructor() ERC20("", "") {}

    function setUp() public {
        uint256 forkBlock = 21409971;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);

        adapter = new BalancerV3SwapAdapter(
            payable(address(balancerV3Vault)), address(router), permit2
        );

        vm.label(address(balancerV3Vault), "BalancerV3Vault");
        vm.label(address(router), "BalancerV3BatchRouter");
        vm.label(USDC, "USDC");
        vm.label(USDT, "USDT");
        vm.label(address(adapter), "BalancerV3SwapAdapter");
    }

    function testGetLimitsBalancerV3() public view {
        bytes32 pool = bytes32(bytes20(USDC_USDT_POOL_ADDRESS));
        uint256[] memory limits = adapter.getLimits(pool, USDT, USDC);
        console.log("limit USDT: ", limits[0]);
        console.log("limit USDC: ", limits[1]);
    }

    function testSwapSellUsdcForGoeth() public {
        bytes32 pool = bytes32(bytes20(GOETH_USDC_POOL_ADDRESS));
        // uint256[] memory limits = adapter.getLimits(pool, USDT, USDC);

        OrderSide side = OrderSide.Sell;
        uint256 specifiedAmount = 10000000;

        deal(USDC, address(this), specifiedAmount);
        IERC20(USDC).approve(address(adapter), type(uint256).max);

        uint256 balBeforeUSDC = IERC20(USDC).balanceOf(address(this));
        uint256 balBeforeGOETH = IERC20(GOETH).balanceOf(address(this));

        Trade memory trade = adapter.swap(pool, USDC, GOETH, side, specifiedAmount);

        uint256 balAfterUSDC = IERC20(USDC).balanceOf(address(this));
        uint256 balAfterGOETH = IERC20(USDT).balanceOf(address(this));

        // assertEq(balBeforeUSDC - balAfterUSDC, specifiedAmount);
        // assertEq(balBeforeGOETH + trade.calculatedAmount, balAfterGOETH);
    }

    function testSwapSellUsdcForUsdt() public {
        bytes32 pool = bytes32(bytes20(USDC_USDT_POOL_ADDRESS));
        // uint256[] memory limits = adapter.getLimits(pool, USDT, USDC);

        OrderSide side = OrderSide.Sell;
        uint256 specifiedAmount = 1000*1e6;

        deal(USDC, address(this), specifiedAmount);
        IERC20(USDC).approve(address(adapter), type(uint256).max);

        uint256 balBeforeUSDC = IERC20(USDC).balanceOf(address(this));
        uint256 balBeforeUSDT = IERC20(USDT).balanceOf(address(this));

        Trade memory trade = adapter.swap(pool, USDC, USDT, side, specifiedAmount);

        uint256 balAfterUSDC = IERC20(USDC).balanceOf(address(this));
        uint256 balAfterUSDT = IERC20(USDT).balanceOf(address(this));

        assertEq(balBeforeUSDC - balAfterUSDC, specifiedAmount);
        assertEq(balBeforeUSDT + trade.calculatedAmount, balAfterUSDT);
    }

    function testSwapSellUsdtForUsdc() public {
        bytes32 pool = bytes32(bytes20(USDC_USDT_POOL_ADDRESS));

        OrderSide side = OrderSide.Sell;
        uint256 specifiedAmount = 10000000;

        deal(USDT, address(this), type(uint256).max);
        IERC20(USDT).approve(address(adapter), specifiedAmount);

        adapter.swap(pool, USDT, USDC, side, specifiedAmount);
    }
}
