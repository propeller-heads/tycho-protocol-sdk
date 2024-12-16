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
    address constant ERC20_POOL_ADDRESS =
        0x4AB7aB316D43345009B2140e0580B072eEc7DF16;
    address constant ERC20_TOKEN_0 = 0x0bfc9d54Fc184518A81162F8fB99c2eACa081202;
    address constant ERC20_TOKEN_1 = 0xA35b1B31Ce002FBF2058D22F30f95D405200A15b;

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
        vm.label(address(adapter), "BalancerV3SwapAdapter");
        vm.label(ERC20_TOKEN_0, "ERC20_TOKEN_0");
        vm.label(ERC20_TOKEN_1, "ERC20_TOKEN_1");
        vm.label(ERC20_POOL_ADDRESS, "ERC20_POOL_ADDRESS");
        vm.label(waEthUSDT, "waEthUSDT");
        vm.label(waEthUSDC, "waEthUSDC");
        vm.label(GOETH_USDC_POOL_ADDRESS, "GOETH_USDC_POOL_ADDRESS");
        vm.label(GOETH, "GOETH");
        vm.label(permit2, "Permit2");
    }

    function testPriceFuzzBalancerV3(uint256 amount0, uint256 amount1) public {
        address token0 = ERC20_TOKEN_0;
        address token1 = ERC20_TOKEN_1;

        bytes32 pool = bytes32(bytes20(ERC20_POOL_ADDRESS));
        uint256[] memory limits = adapter.getLimits(pool, token0, token1);

        vm.assume(amount0 < limits[0] && amount0 > _MINIMUM_TRADE_AMOUNT);
        vm.assume(amount1 < limits[1] && amount1 > _MINIMUM_TRADE_AMOUNT);

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

    function testSwapFuzzBalancerV3_ERC20_ERC20(uint256 specifiedAmount, bool isBuy)
        public
    {
        address token0 = ERC20_TOKEN_0;
        address token1 = ERC20_TOKEN_1;

        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        bytes32 pool = bytes32(bytes20(ERC20_POOL_ADDRESS));
        uint256[] memory limits = adapter.getLimits(pool, token0, token1);

        if (side == OrderSide.Buy) {
            vm.assume(
                specifiedAmount < limits[1]
                    && specifiedAmount > _MINIMUM_TRADE_AMOUNT
            );
        } else {
            vm.assume(
                specifiedAmount < limits[0]
                    && specifiedAmount > _MINIMUM_TRADE_AMOUNT
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

    function __prankStaticCall() internal {
        // Prank address 0x0 for both msg.sender and tx.origin (to identify as a
        // staticcall).
        vm.prank(address(0), address(0));
    }
}
