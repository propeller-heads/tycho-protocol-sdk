// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import "src/curve/CurveAdapter.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";
import "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import "forge-std/console.sol";

contract CurveAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    CurveAdapter adapter;

    address constant DAI = 0x6B175474E89094C44Da98b954EedeAC495271d0F;
    address constant USDC = 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48;
    address constant USDT = 0xdAC17F958D2ee523a2206206994597C13D831ec7;
    address constant GUSD = 0x056Fd409E1d7A124BD7017459dFEa2F387b6d5Cd;
    address constant TRECRV = 0x6c3F90f043a72FA612cbac8115EE7e52BDe6E490;
    address constant WBTC = 0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599;
    address constant WETH = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
    address constant ETH = address(0);
    address constant wBETH = 0xa2E3356610840701BDf5611a53974510Ae27E2e1;
    address constant EURS = 0xdB25f211AB05b1c97D595516F45794528a807ad8;

    address constant STABLE_POOL_3POOL =
        0xbEbc44782C7dB0a1A60Cb6fe97d0b483032FF1C7;
    address constant STABLE_POOL_META_GUSD3CRV =
        0x4f062658EaAF2C1ccf8C8e36D6824CDf41167956;
    address constant CRYPTO_POOL_EURSUSDC =
        0x98a7F18d4E56Cfe84E3D081B40001B3d5bD3eB8B;
    address constant STABLE_POOL_ETHwBETHCRV =
        0xBfAb6FA95E0091ed66058ad493189D2cB29385E6;
    address constant CRYPTO_POOL_TRICRYPTO2 =
        0xD51a44d3FaE010294C616388b506AcdA1bfAAE46;

    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 19910426;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new CurveAdapter(0xF98B45FA17DE75FB1aD0e7aFD971b0ca00e379fC);

        vm.label(address(adapter), "CurveAdapter");

        vm.label(DAI, "DAI");
        vm.label(USDC, "USDC");
        vm.label(USDT, "USDT");
        vm.label(GUSD, "GUSD");
        vm.label(TRECRV, "TRECRV");
        vm.label(WBTC, "WBTC");
        vm.label(WETH, "WETH");
        vm.label(ETH, "ETH");
        vm.label(wBETH, "wBETH");
        vm.label(EURS, "EURS");

        vm.label(STABLE_POOL_3POOL, "STABLE_POOL_3POOL");
        vm.label(STABLE_POOL_META_GUSD3CRV, "STABLE_POOL_META_GUSD3CRV");
        vm.label(CRYPTO_POOL_EURSUSDC, "CRYPTO_POOL_EURSUSDC");
        vm.label(STABLE_POOL_ETHwBETHCRV, "STABLE_POOL_ETHwBETHCRV");
        vm.label(CRYPTO_POOL_TRICRYPTO2, "CRYPTO_POOL_TRICRYPTO2");
    }

    receive() external payable {}

    /// Pool = STABLE_POOL_3POOL; sellToken = USDC; buyToken = USDT;
    function testSwapFuzzCurveStablePool3Pool(uint256 specifiedAmount) public {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(STABLE_POOL_3POOL));
        uint256[] memory limits = adapter.getLimits(pair, USDC, USDT);

        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 6);

        deal(USDC, address(this), specifiedAmount);
        IERC20(USDC).approve(address(adapter), specifiedAmount);

        uint256 usdc_balance = IERC20(USDC).balanceOf(address(this));
        uint256 usdt_balance = IERC20(USDT).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, USDC, USDT, side, specifiedAmount);

        assertEq(
            specifiedAmount,
            usdc_balance - IERC20(USDC).balanceOf(address(this))
        );
        assertEq(
            trade.calculatedAmount,
            IERC20(USDT).balanceOf(address(this)) - usdt_balance
        );
    }

    /// Pool = STABLE_POOL_META_GUSD3CRV; sellToken = USDT; buyToken = DAI;
    /// @dev swapping underlying tokens
    /// @dev [FAIL. Reason: The `vm.assume` cheatcode rejected too many inputs
    /// (65536 allowed)]
    function testSwapFuzzCurveStablePoolMetaGusd3CrvUnderlying(
        uint256 specifiedAmount
    ) public {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(STABLE_POOL_META_GUSD3CRV));
        uint256[] memory limits = adapter.getLimits(pair, USDT, DAI);

        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 6);

        deal(USDT, address(this), specifiedAmount);
        IERC20(USDT).approve(address(adapter), specifiedAmount);

        uint256 usdt_balance = IERC20(USDT).balanceOf(address(this));
        uint256 dai_balance = IERC20(DAI).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, USDT, DAI, side, specifiedAmount);

        assertEq(
            specifiedAmount,
            usdt_balance - IERC20(USDT).balanceOf(address(this))
        );
        assertEq(
            trade.calculatedAmount,
            IERC20(DAI).balanceOf(address(this)) - dai_balance
        );
    }

    /// Pool = CRYPTO_POOL_EURSUSDC; sellToken = USDC; buyToken = EURS
    function testSwapFuzzCurveCryptoPoolEursUsdc(uint256 specifiedAmount)
        public
    {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(CRYPTO_POOL_EURSUSDC));
        uint256[] memory limits = adapter.getLimits(pair, USDC, EURS);

        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 4);

        deal(USDC, address(this), specifiedAmount);
        IERC20(USDC).approve(address(adapter), specifiedAmount);

        uint256 usdc_balance = IERC20(USDC).balanceOf(address(this));
        uint256 eurs_balance = IERC20(EURS).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, USDC, EURS, side, specifiedAmount);

        assertEq(
            specifiedAmount,
            usdc_balance - IERC20(USDC).balanceOf(address(this))
        );
        assertEq(
            trade.calculatedAmount,
            IERC20(EURS).balanceOf(address(this)) - eurs_balance
        );
    }

    /// Pool = STABLE_POOL_3POOL; sellToken = DAI; buyToken = USDC;
    /// @dev [FAIL. Reason: assertion failed: 1 != 0]
    function testSwapSellIncreasingSwapsCurveStablePool3Pool() public {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(STABLE_POOL_3POOL));

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * i * 10 ** 6;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(DAI, address(this), amounts[i]);
            IERC20(DAI).approve(address(adapter), amounts[i]);

            trades[i] = adapter.swap(pair, DAI, USDC, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 0);
        }
    }

    /// Pool = STABLE_POOL_META_GUSD3CRV; sellToken = GUSD; buyToken = TRECRV
    /// @dev FAIL. Reason: revert: stdStorage find(StdStorage): Slot(s) not
    /// found.]
    /// @dev GUSD has only 2 decimals
    /// @dev GUSD = 0x056Fd409E1d7A124BD7017459dFEa2F387b6d5Cd; is a Proxy
    /// contract
    function testSwapSellIncreasingSwapsCurveStablePoolMetaGusd3Crv() public {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(STABLE_POOL_META_GUSD3CRV));

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = i * 10;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(GUSD, address(this), amounts[i]);
            IERC20(GUSD).approve(address(adapter), amounts[i]);

            trades[i] = adapter.swap(pair, GUSD, TRECRV, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    /// Pool = STABLE_POOL_META_GUSD3CRV; sellToken = DAI; buyToken = USDT
    /// Swapping underlying tokens
    function testSwapSellIncreasingSwapsCurveStablePoolMetaGusd3CrvUnderlying()
        public
    {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(STABLE_POOL_META_GUSD3CRV));

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * i * 10 ** 6;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(DAI, address(this), amounts[i]);
            IERC20(DAI).approve(address(adapter), amounts[i]);

            trades[i] = adapter.swap(pair, DAI, USDT, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 0);
        }
    }

    /// Pool = CRYPTO_POOL_EURSUSDC; sellToken = EURS; buyToken = USDC
    function testSwapSellIncreasingSwapsCurveCryptoPoolEursUsdc() public {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(CRYPTO_POOL_EURSUSDC));

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * i * 10 ** 4;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(EURS, address(this), amounts[i]);
            IERC20(EURS).approve(address(adapter), amounts[i]);

            trades[i] = adapter.swap(pair, EURS, USDC, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    /// Pool = STABLE_POOL_ETHwBETHCRV; sellToken = wBETH; buyToken = ETH
    /// @dev fails
    function testSwapSellIncreasingSwapsCurveStablePoolEthwBethCrvwBethForEth()
        public
    {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(STABLE_POOL_ETHwBETHCRV));

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * i * 10 ** 16;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(wBETH, address(this), amounts[i]);
            IERC20(wBETH).approve(address(adapter), amounts[i]);

            trades[i] = adapter.swap(pair, wBETH, ETH, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    /// Pool = STABLE_POOL_ETHwBETHCRV; sellToken = ETH; buyToken = wBETH
    /// @dev [FAIL. Reason: revert: Swap failed]
    function testSwapSellIncreasingSwapsCurveStablePoolEthwBethCrvEthForwBeth()
        public
    {
        OrderSide side = OrderSide.Sell;

        bytes32 pair = bytes32(bytes20(STABLE_POOL_ETHwBETHCRV));

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * i * 10 ** 10; // Adjust the amount
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            // Deal ETH to this address
            deal(address(adapter), amounts[i]);

            // // Construct the call data for the swap function
            // bytes memory callData = abi.encodeWithSelector(
            //     adapter.swap.selector, pair, ETH, wBETH, side, amounts[i]
            // );

            trades[i] = adapter.swap(pair, ETH, wBETH, side, amounts[i]);

            // // Call the swap function with ETH
            // (bool success, bytes memory returnData) =
            //     address(adapter).call{value: amounts[i]}(callData);

            // Assert that the call was successful
            // require(success, "Swap failed");

            // // Decode the returned data
            // trades[i] = abi.decode(returnData, (Trade));

            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertEq(trades[i].price.compareFractions(trades[i + 1].price), 1);
        }
    }

    function testGetTokensCurveStablePool3Pool() public {
        bytes32 pair = bytes32(bytes20(STABLE_POOL_3POOL));
        address[] memory tokens = adapter.getTokens(pair);

        assertEq(tokens.length, 3);
        assertEq(tokens[0], DAI);
        assertEq(tokens[1], USDC);
        assertEq(tokens[2], USDT);
    }

    function testGetTokensCurveStablePoolMetaGusd3Crv() public {
        bytes32 pair = bytes32(bytes20(STABLE_POOL_META_GUSD3CRV));
        address[] memory tokens = adapter.getTokens(pair);

        assertEq(tokens.length, 2);
        assertEq(tokens[0], GUSD);
        assertEq(tokens[1], TRECRV);
    }

    function testGetTokensCurveCryptoPoolEursUsdc() public {
        bytes32 pair = bytes32(bytes20(CRYPTO_POOL_EURSUSDC));
        address[] memory tokens = adapter.getTokens(pair);

        assertEq(tokens.length, 2);
        assertEq(tokens[0], USDC);
        assertEq(tokens[1], EURS);
    }

    function testGetTokensCurveStablePoolEthwBethCrv() public {
        bytes32 pair = bytes32(bytes20(STABLE_POOL_ETHwBETHCRV));
        address[] memory tokens = adapter.getTokens(pair);

        assertEq(tokens.length, 2);
        assertEq(tokens[0], ETH);
        assertEq(tokens[1], wBETH);
    }

    function testGetLimitsCurveStablePool3Pool() public {
        bytes32 pair = bytes32(bytes20(STABLE_POOL_3POOL));
        uint256[] memory limits = adapter.getLimits(pair, DAI, USDC);

        assertEq(limits.length, 2);
    }

    function testGetLimitsCurveStablePoolMetaGusd3Crv() public {
        bytes32 pair = bytes32(bytes20(STABLE_POOL_META_GUSD3CRV));
        uint256[] memory limits = adapter.getLimits(pair, GUSD, TRECRV);

        assertEq(limits.length, 2);
    }

    function testGetLimitsCurveCryptoPoolEursUsdc() public {
        bytes32 pair = bytes32(bytes20(CRYPTO_POOL_EURSUSDC));
        uint256[] memory limits = adapter.getLimits(pair, EURS, USDC);

        assertEq(limits.length, 2);
    }

    function testGetLimitsCurveStablePoolEthwBethCrv() public {
        bytes32 pair = bytes32(bytes20(STABLE_POOL_ETHwBETHCRV));
        uint256[] memory limits = adapter.getLimits(pair, ETH, wBETH);

        assertEq(limits.length, 2);
    }
}
