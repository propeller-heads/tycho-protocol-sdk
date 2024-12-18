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
    using FractionMath for Fraction;

    // because this value also applies to out tokens and is used in checks
    uint256 _MINIMUM_WRAP_AMOUNT = 10001;

    IVault constant balancerV3Vault =
        IVault(payable(0xbA1333333333a1BA1108E8412f11850A5C319bA9));
    BalancerV3SwapAdapter adapter;
    IBatchRouter router =
        IBatchRouter(0x136f1EFcC3f8f88516B9E94110D56FDBfB1778d1); // Batch router
    address constant permit2 = 0x000000000022D473030F116dDEE9F6B43aC78BA3;

    // ETHx waWETH - Stable Pool
    address constant ERC4626_ERC20_ETHx_waWETH_STABLE_POOL =
        0x4AB7aB316D43345009B2140e0580B072eEc7DF16;
    address constant ERC4626_waEthWETH =
        0x0bfc9d54Fc184518A81162F8fB99c2eACa081202;
    address constant ERC20_ETHx = 0xA35b1B31Ce002FBF2058D22F30f95D405200A15b;

    // 50USDC-50@G - Weighted Pool
    address constant ERC20_ERC20_GOETH_USDC_WEIGHTED_POOL =
        0xf91c11BA4220b7a72E1dc5E92f2b48D3fdF62726;
    address constant ERC20_GOETH = 0x440017A1b021006d556d7fc06A54c32E42Eb745B;
    address constant ERC20_USDC = 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48;

    // Aave Lido wETH-wstETH - Stable Pool
    address constant ERC4626_ERC4626_WETH_wstETH_STABLE_POOL =
        0xc4Ce391d82D164c166dF9c8336DDF84206b2F812;
    address constant ERC20_WETH = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
    address constant ERC20_wstETH = 0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0;
    address constant ERC4626_waEthLidoWETH =
        0x0FE906e030a44eF24CA8c7dC7B7c53A6C4F00ce9;
    address constant ERC4626_waEthLidowstETH =
        0x775F661b0bD1739349b9A2A3EF60be277c5d2D29;
    address constant ETH = address(0);

    uint256 constant TEST_ITERATIONS = 100;

    constructor() ERC20("", "") {}

    function setUp() public {
        uint256 forkBlock = 21421638;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);

        adapter = new BalancerV3SwapAdapter(
            payable(address(balancerV3Vault)),
            address(router),
            permit2,
            ERC20_WETH
        );

        vm.label(address(balancerV3Vault), "BalancerV3Vault");
        vm.label(address(router), "BalancerV3BatchRouter");
        vm.label(address(adapter), "BalancerV3SwapAdapter");
        vm.label(ERC4626_waEthWETH, "ERC4626_waEthWETH");
        vm.label(ERC20_ETHx, "ERC20_ETHx");
        vm.label(
            ERC4626_ERC20_ETHx_waWETH_STABLE_POOL, "ERC4626_ERC20_ETHx_waWETH_STABLE_POOL"
        );
        vm.label(
            ERC20_ERC20_GOETH_USDC_WEIGHTED_POOL, "ERC20_ERC20_GOETH_USDC_WEIGHTED_POOL"
        );
        vm.label(ERC20_GOETH, "ERC20_GOETH");
        vm.label(ERC20_USDC, "ERC20_USDC");
        vm.label(
            ERC4626_ERC4626_WETH_wstETH_STABLE_POOL,
            "ERC4626_ERC4626_WETH_wstETH_STABLE_POOL"
        );
        vm.label(ERC20_WETH, "ERC20_WETH");
        vm.label(ERC20_wstETH, "ERC20_wstETH");
        vm.label(ERC4626_waEthLidoWETH, "ERC4626_waEthLidoWETH");
        vm.label(ERC4626_waEthLidoWETH, "ERC4626_waEthLidoWETH");
        vm.label(permit2, "Permit2");
    }

    ///////////////////////////////////////// ERC4626_ERC20_DIRECT
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////// 

    function testPriceFuzzBalancerV3_ERC4626_ERC20_DIRECT(uint256 amount0) public {
        address token0 = ERC4626_waEthWETH;
        address token1 = ERC20_ETHx;

        bytes32 pool = bytes32(bytes20(ERC4626_ERC20_ETHx_waWETH_STABLE_POOL));
        uint256[] memory limits = adapter.getLimits(pool, token0, token1);
        uint256 minTradeAmount = getMinTradeAmount(token0);

        vm.assume(amount0 < limits[0]);
        vm.assume(amount0 > minTradeAmount);

        uint256[] memory amounts = new uint256[](1);
        amounts[0] = amount0;

        __prankStaticCall();
        Fraction[] memory prices = adapter.price(pool, token0, token1, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testSwapFuzzBalancerV3_ERC4626_ERC20_DIRECT(
        uint256 specifiedAmount,
        bool isBuy
    ) public {
        address token0 = ERC4626_waEthWETH;
        address token1 = ERC20_ETHx;

        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        bytes32 pool = bytes32(bytes20(ERC4626_ERC20_ETHx_waWETH_STABLE_POOL));
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
        IERC4626(token0).approve(address(adapter), type(uint256).max);

        uint256 bal0 = IERC4626(token0).balanceOf(address(this));
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
                bal0 - IERC4626(token0).balanceOf(address(this))
            );
        } else {
            assertEq(
                specifiedAmount,
                bal0 - IERC4626(token0).balanceOf(address(this))
            );
            assertEq(
                trade.calculatedAmount,
                IERC20(token1).balanceOf(address(this)) - bal1
            );
        }
    }

    ///////////////////////////////////////// ERC20_ERC20_DIRECT
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////// 

    function testPriceFuzzBalancerV3_ERC20_ERC20_DIRECT(uint256 amount0) public {
        address token0 = ERC20_GOETH;
        address token1 = ERC20_USDC;

        bytes32 pool = bytes32(bytes20(ERC20_ERC20_GOETH_USDC_WEIGHTED_POOL));
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

    function testSwapFuzzBalancerV3_ERC20_ERC20_DIRECT(
        uint256 specifiedAmount,
        bool isBuy
    ) public {
        address token0 = ERC20_GOETH;
        address token1 = ERC20_USDC;

        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        bytes32 pool = bytes32(bytes20(ERC20_ERC20_GOETH_USDC_WEIGHTED_POOL));
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

    ///////////////////////////////////////// ERC4626_ERC4626_DIRECT
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    function testPriceFuzzBalancerV3_ERC4626_ERC4626_DIRECT(
        uint256 amount0
    ) public {
        address token0 = ERC4626_waEthLidoWETH;
        address token1 = ERC4626_waEthLidowstETH;

        bytes32 pool = bytes32(bytes20(ERC4626_ERC4626_WETH_wstETH_STABLE_POOL));

        uint256[] memory limits = adapter.getLimits(pool, token0, token1);

        uint256 minTradeAmount = getMinTradeAmount(token0);

        vm.assume(amount0 < limits[0]);
        vm.assume(amount0 > minTradeAmount);

        uint256[] memory amounts = new uint256[](1);
        amounts[0] = amount0;

        __prankStaticCall();
        Fraction[] memory prices = adapter.price(pool, token0, token1, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testSwapFuzzBalancerV3_ERC4626_ERC4626_DIRECT(
        uint256 specifiedAmount,
        bool isBuy
    ) public {
        address token0 = ERC4626_waEthLidoWETH;
        address token1 = ERC4626_waEthLidowstETH;

        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        bytes32 pool = bytes32(bytes20(ERC4626_ERC4626_WETH_wstETH_STABLE_POOL));
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

        deal(token0, address(this), IERC4626(token0).totalSupply() * 2);
        IERC4626(token0).approve(address(adapter), type(uint256).max);

        uint256 bal0 = IERC4626(token0).balanceOf(address(this));
        uint256 bal1 = IERC4626(token1).balanceOf(address(this));

        uint256[] memory amounts = new uint256[](1);
        amounts[0] = specifiedAmount;
        Trade memory trade =
            adapter.swap(pool, token0, token1, side, specifiedAmount);

        if (side == OrderSide.Buy) {
            assertEq(
                specifiedAmount,
                IERC4626(token1).balanceOf(address(this)) - bal1
            );
            assertEq(
                trade.calculatedAmount,
                bal0 - IERC4626(token0).balanceOf(address(this))
            );
        } else {
            assertEq(
                specifiedAmount,
                bal0 - IERC4626(token0).balanceOf(address(this))
            );
            assertEq(
                trade.calculatedAmount,
                IERC4626(token1).balanceOf(address(this)) - bal1
            );
        }
    }

    ///////////////////////////////////////// ETH --> WETH --> (waEthWETH --> ETHx) WRAP_WRAP_SWAP
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    function testPriceFuzzBalancerV3_ETH_ERC20_ERC4626_ERC20_WRAP_WRAP_SWAP(uint256 amount0) public {
        address token0 = ETH;
        address token1 = ERC20_ETHx;

        bytes32 pool = bytes32(bytes20(ERC4626_ERC20_ETHx_waWETH_STABLE_POOL));
        uint256[] memory limits = adapter.getLimits(pool, token0, token1);
        uint256 minTradeAmount = getMinTradeAmount(token0);

        vm.assume(amount0 < limits[0]);
        vm.assume(amount0 > minTradeAmount);

        uint256[] memory amounts = new uint256[](1);
        amounts[0] = amount0;

        __prankStaticCall();
        Fraction[] memory prices = adapter.price(pool, token0, token1, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testSwapFuzzBalancerV3_ETH_ERC20_ERC4626_ERC20_WRAP_WRAP_SWAP(
        uint256 specifiedAmount,
        bool isBuy
    ) public {
        address token0 = ETH;
        address token1 = ERC20_ETHx;

        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        bytes32 pool = bytes32(bytes20(ERC4626_ERC20_ETHx_waWETH_STABLE_POOL));
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

        deal(address(this), type(uint256).max);

        uint256 bal0 = address(this).balance;
        uint256 bal1 = IERC20(token1).balanceOf(address(this));

        uint256[] memory amounts = new uint256[](1);
        amounts[0] = specifiedAmount;
        Trade memory trade = adapter.swap{value: isBuy ? trade.calculatedAmount : specifiedAmount}(
            pool, 
            token0,
            token1,
            side,
            specifiedAmount
        );

        if (side == OrderSide.Buy) {
            assertEq(
                specifiedAmount, IERC20(token1).balanceOf(address(this)) - bal1
            );
            assertEq(
                trade.calculatedAmount,
                bal0 - address(this).balance
            );
        } else {
            assertEq(
                specifiedAmount,
                bal0 - address(this).balance
            );
            assertEq(
                trade.calculatedAmount,
                IERC20(token1).balanceOf(address(this)) - bal1
            );
        }
    }

    ///////////////////////////////////////// ( ETHx --> waEthWETH) --> WETH --> ETH  SWAP_UNWRAP_UNWRAP
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    function testPriceFuzzBalancerV3_ERC20_ERC4626_ERC20_ETH_SWAP_UNWRAP_UNWRAP(uint256 amount0) public {
        address token0 = ERC20_ETHx;
        address token1 = ETH;

        bytes32 pool = bytes32(bytes20(ERC4626_ERC20_ETHx_waWETH_STABLE_POOL));
        uint256[] memory limits = adapter.getLimits(pool, token0, token1);
        uint256 minTradeAmount = getMinTradeAmount(token0);

        vm.assume(amount0 < limits[0]);
        vm.assume(amount0 > minTradeAmount);

        uint256[] memory amounts = new uint256[](1);
        amounts[0] = amount0;

        __prankStaticCall();
        Fraction[] memory prices = adapter.price(pool, token0, token1, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testSwapFuzzBalancerV3_ERC20_ERC4626_ERC20_ETH_SWAP_UNWRAP_UNWRAP(
        uint256 specifiedAmount,
        bool isBuy
    ) public {
        address token0 = ERC20_ETHx;
        address token1 = ETH;

        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        bytes32 pool = bytes32(bytes20(ERC4626_ERC20_ETHx_waWETH_STABLE_POOL));
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

        deal(token0, address(this), IERC20(token0).totalSupply() * 2);
        IERC20(token0).approve(address(adapter), type(uint256).max);

        uint256 bal0 = IERC20(token0).balanceOf(address(this));
        uint256 bal1 = address(this).balance;

        uint256[] memory amounts = new uint256[](1);
        amounts[0] = specifiedAmount;
        Trade memory trade =
            adapter.swap(pool, token0, token1, side, specifiedAmount);

        if (side == OrderSide.Buy) {
            assertEq(
                specifiedAmount, address(this).balance - bal1
            );
            assertEq(
                trade.calculatedAmount,
                bal0 - IERC20(token0).balanceOf(address(this))
            );
        } else {
            assertEq(
                specifiedAmount,
                bal0 - IERC20(token0).balanceOf(address(this))
            );
            assertEq(
                trade.calculatedAmount,
                address(this).balance - bal1
            );
        }
    }

    ///////////////////////////////////////// ERC20-->(ERC4626 --> ERC4626) WRAP_SWAP
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    function testPriceFuzzBalancerV3_ERC20_ERC4626_ERC4626_WRAP_SWAP(uint256 amount0)
        public
    {
        address token0 = ERC20_WETH;
        address token1 = ERC4626_waEthLidowstETH;

        bytes32 pool = bytes32(bytes20(ERC4626_ERC4626_WETH_wstETH_STABLE_POOL));

        uint256[] memory limits = adapter.getLimits(pool, token0, token1);

        uint256 minTradeAmount = getMinTradeAmount(token0);

        vm.assume(amount0 < limits[0]);
        vm.assume(amount0 > minTradeAmount);

        uint256[] memory amounts = new uint256[](1);
        amounts[0] = amount0;

        __prankStaticCall();
        Fraction[] memory prices = adapter.price(pool, token0, token1, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testSwapFuzzBalancerV3_ERC20_ERC4626_ERC4626_WRAP_SWAP(
        uint256 specifiedAmount,
        bool isBuy
    ) public {
        address token0 = ERC20_WETH;
        address token1 = ERC4626_waEthLidowstETH;

        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        bytes32 pool = bytes32(bytes20(ERC4626_ERC4626_WETH_wstETH_STABLE_POOL));
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

        deal(token0, address(this), IERC20(token0).totalSupply() * 2);
        IERC20(token0).approve(address(adapter), type(uint256).max);

        uint256 bal0 = IERC20(token0).balanceOf(address(this));
        uint256 bal1 = IERC4626(token1).balanceOf(address(this));

        uint256[] memory amounts = new uint256[](1);
        amounts[0] = specifiedAmount;
        Trade memory trade =
            adapter.swap(pool, token0, token1, side, specifiedAmount);

        if (side == OrderSide.Buy) {
            assertEq(
                specifiedAmount,
                IERC4626(token1).balanceOf(address(this)) - bal1
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
                IERC4626(token1).balanceOf(address(this)) - bal1
            );
        }
    }

    ///////////////////////////////////////// (ERC4626-->ERC4626)--> ERC20 SWAP_UNWRAP
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    function testPriceFuzzBalancerV3_ERC4626_ERC4626_ERC20_SWAP_UNWRAP(uint256 amount0)
        public
    {
        address token0 = ERC4626_waEthLidowstETH;
        address token1 = ERC20_WETH;

        bytes32 pool = bytes32(bytes20(ERC4626_ERC4626_WETH_wstETH_STABLE_POOL));

        uint256[] memory limits = adapter.getLimits(pool, token0, token1);

        uint256 minTradeAmount = getMinTradeAmount(token0);

        vm.assume(amount0 < limits[0]);
        vm.assume(amount0 > minTradeAmount);

        uint256[] memory amounts = new uint256[](1);
        amounts[0] = amount0;

        __prankStaticCall();
        Fraction[] memory prices = adapter.price(pool, token0, token1, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testSwapFuzzBalancerV3_ERC4626_ERC4626_ERC20_SWAP_UNWRAP(
        uint256 specifiedAmount,
        bool isBuy
    ) public {
        address token0 = ERC4626_waEthLidowstETH;
        address token1 = ERC20_WETH;

        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        bytes32 pool = bytes32(bytes20(ERC4626_ERC4626_WETH_wstETH_STABLE_POOL));
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
        IERC4626(token0).approve(address(adapter), type(uint256).max);

        uint256 bal0 = IERC4626(token0).balanceOf(address(this));
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
                bal0 - IERC4626(token0).balanceOf(address(this))
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

    ///////////////////////////////////////// ERC20-->ERC20 UNDERLYING DIRECT
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    function testPriceFuzzBalancerV3_ERC20_ERC20_UNDERLYING_DIRECT(uint256 amount0) public {
        address token0 = ERC20_WETH;
        address token1 = ERC20_wstETH;

        bytes32 pool = bytes32(bytes20(ERC4626_ERC4626_WETH_wstETH_STABLE_POOL));

        uint256[] memory limits = adapter.getLimits(pool, token0, token1);

        uint256 minTradeAmount = getMinTradeAmount(token0);

        vm.assume(amount0 < limits[0]);
        vm.assume(amount0 > minTradeAmount);

        uint256[] memory amounts = new uint256[](1);
        amounts[0] = amount0;

        __prankStaticCall();
        Fraction[] memory prices = adapter.price(pool, token0, token1, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testSwapFuzzBalancerV3_ERC20_ERC20_UNDERLYING_DIRECT(
        uint256 specifiedAmount,
        bool isBuy
    ) public {
        address token0 = ERC20_WETH;
        address token1 = ERC20_wstETH;

        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        bytes32 pool = bytes32(bytes20(ERC4626_ERC4626_WETH_wstETH_STABLE_POOL));
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

        deal(token0, address(this), IERC20(token0).totalSupply() * 2);
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

    function getMinTradeAmount(address token) internal view returns (uint256) {
        uint256 decimals = ERC20(token).decimals();
        uint256 decimalFactor = decimals; // n, e.g. stablecoins
        if (decimals > 6) {
            decimalFactor = decimals - 1; // 0.n
        }
        if (decimals > 12) {
            decimalFactor = decimals - 3; // e.g. ETH, BTC, ...
        }

        uint256 minTradeAmount = 10 ** decimalFactor;

        return minTradeAmount;
    }
}
