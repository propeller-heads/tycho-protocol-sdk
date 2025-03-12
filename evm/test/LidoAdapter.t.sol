// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import "src/lido/LidoAdapter.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";

contract LidoAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    LidoAdapter adapter;
    address constant wstETH = 0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0;
    address constant stETH = 0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84;
    address constant kernelProxy = 0xb8FFC3Cd6e7Cf5a098A1c92F48009765B24088Dc;
    address constant kernel = 0x2b33CF282f867A7FF693A66e11B0FcC5552e4425;
    address constant lido = 0x17144556fd3424EDC8Fc8A4C940B2D04936d17eb;
    address constant lido2 = 0x20dC62D5904633cC6a5E34bEc87A048E80C92e97;
    address constant ETH = address(0);
    uint256 constant TEST_ITERATIONS = 100;
    uint256 constant BUFFER = 1000000;

    function setUp() public {
        uint256 forkBlock = 21929540;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new LidoAdapter(wstETH, stETH);

        vm.label(address(adapter), "LidoAdapter");
        vm.label(ETH, "ETH");
        vm.label(wstETH, "wstETH");
        vm.label(stETH, "stETH");
        vm.label(kernelProxy, "KernelProxy");
        vm.label(kernel, "Kernel");
        vm.label(lido, "Lido");
        vm.label(lido2, "Lido2");
    }

    /// @dev enable receive as ether will be sent to this address, and it is a
    /// contract, to prevent reverts
    receive() external payable {}

    /// @dev custom function to mint stETH tokens as normal "deal" will revert
    /// due to stETH internal functions
    /// (ref: StETH.sol:375)
    /// @dev because of internal precision losses in Lido contracts, we unwrap
    /// the final amount - 1 to prevent overflows,
    /// although the final amounts are the expected since solidity handles it
    /// correctly
    /// throughout the contract calls and operations. Indeed the asserts are met
    /// in swap functions.
    /// (ref: LidoAdapter.t.sol:196)

    function dealStEthTokens(uint256 amount) public {
        // For stETH, balances are calculated based on shares
        // We need to find the storage slot for the shares mapping in the stETH
        // contract

        // First, let's check our current balance
        uint256 initialBalance = IERC20(stETH).balanceOf(address(this));

        // The shares mapping is at slot 0 in the StETH contract
        // The specific slot for an address's shares is keccak256(address . slot)
        bytes32 sharesSlot = keccak256(abi.encode(address(this), uint256(0)));

        // Calculate shares directly using the contract's function
        // Add a small buffer (0.1%) to account for rounding issues
        uint256 sharesToAdd = IStETH(stETH).getSharesByPooledEth(amount);
        sharesToAdd = sharesToAdd + (sharesToAdd / 1000); // Add 0.1% buffer

        // Read the current shares value
        bytes32 currentSharesValue = vm.load(stETH, sharesSlot);
        uint256 currentShares = uint256(currentSharesValue);

        // Calculate the new shares value
        uint256 newShares = currentShares + sharesToAdd;
        bytes32 newSharesValue = bytes32(newShares);

        // Store the new shares value
        vm.store(stETH, sharesSlot, newSharesValue);

        // Verify the new balance
        uint256 updatedBalance = IERC20(stETH).balanceOf(address(this));

        // If the balance didn't update correctly, try a different approach
        if (updatedBalance < initialBalance + (amount * 9) / 10) {
            // We might need to also update the total shares
            bytes32 totalSharesSlot =
                0xe3b4b636e601189b5f4c6742edf2538ac12bb61ed03e6da26949d69838fa447e;
            bytes32 currentTotalSharesValue = vm.load(stETH, totalSharesSlot);
            uint256 currentTotalShares = uint256(currentTotalSharesValue);

            // Update total shares
            uint256 newTotalShares = currentTotalShares + sharesToAdd;
            bytes32 newTotalSharesValue = bytes32(newTotalShares);
            vm.store(stETH, totalSharesSlot, newTotalSharesValue);

            updatedBalance = IERC20(stETH).balanceOf(address(this));
        }
    }

    function testPriceLidoSteth() public view {
        Fraction[] memory prices = new Fraction[](2);

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = (0.001 ether) + (i * 10 ** 14);
        }

        // stETH-ETH
        prices = adapter.price(bytes32(0), ETH, stETH, amounts);
        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }

        // stETH-wstETH
        prices = adapter.price(bytes32(0), stETH, wstETH, amounts);
        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testPriceLidoETH() public view {
        Fraction[] memory prices = new Fraction[](2);

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = (0.001 ether) + (i * 10 ** 14);
        }

        // ETH-stETH
        prices = adapter.price(bytes32(0), ETH, stETH, amounts);
        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }

        // ETH-wstETH
        prices = adapter.price(bytes32(0), ETH, wstETH, amounts);
        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testPriceLidoWsteth() public view {
        Fraction[] memory prices = new Fraction[](2);

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = (0.001 ether) + (i * 10 ** 14);
        }

        // wstETH-ETH
        prices = adapter.price(bytes32(0), ETH, wstETH, amounts);
        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }

        // wstETH-stETH
        prices = adapter.price(bytes32(0), wstETH, stETH, amounts);
        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testSwapFuzzLidoStEthWstEth(uint256 specifiedAmount, bool isBuy)
        public
    {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        vm.assume(specifiedAmount > 1e10);

        bytes32 pair = bytes32(0);

        uint256[] memory limits = adapter.getLimits(pair, stETH, wstETH);
        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1]);

            dealStEthTokens(IStETH(stETH).getPooledEthByShares(specifiedAmount));

            // Calculate the amount with buffer that the adapter will try to
            // transfer
            uint256 neededStEth =
                IStETH(stETH).getPooledEthByShares(specifiedAmount);
            neededStEth = neededStEth + (neededStEth / BUFFER);

            // Approve the amount with buffer
            IERC20(stETH).approve(address(adapter), neededStEth);
        } else {
            vm.assume(specifiedAmount < limits[0]);

            dealStEthTokens(specifiedAmount);

            // For sell orders, we need to approve the exact amount
            IERC20(stETH).approve(address(adapter), specifiedAmount);
        }
        uint256 stETH_balance_before = IERC20(stETH).balanceOf(address(this));
        uint256 wstETH_balance_before = IERC20(wstETH).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, stETH, wstETH, side, specifiedAmount);

        uint256 stETH_balance_after = IERC20(stETH).balanceOf(address(this));
        uint256 wstETH_balance_after = IERC20(wstETH).balanceOf(address(this));

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    wstETH_balance_after - wstETH_balance_before
                );
                assertApproxEqAbs(
                    trade.calculatedAmount,
                    stETH_balance_before - stETH_balance_after,
                    2
                );
            } else {
                assertApproxEqAbs(
                    specifiedAmount,
                    stETH_balance_before - stETH_balance_after,
                    2
                );
                assertEq(
                    trade.calculatedAmount,
                    wstETH_balance_after - wstETH_balance_before
                );
            }
        }
    }

    function testSwapFuzzLidoWstEthStEth(uint256 specifiedAmount, bool isBuy)
        public
    {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        vm.assume(specifiedAmount > 1e10);

        bytes32 pair = bytes32(0);

        uint256[] memory limits = adapter.getLimits(pair, wstETH, stETH);
        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1]);

            uint256 neededWstEth =
                IStETH(stETH).getSharesByPooledEth(specifiedAmount);

            neededWstEth = neededWstEth + (neededWstEth / BUFFER);

            deal(wstETH, address(this), neededWstEth);
            IERC20(wstETH).approve(address(adapter), neededWstEth);
        } else {
            vm.assume(specifiedAmount < limits[0]);

            deal(wstETH, address(this), specifiedAmount);
            IERC20(wstETH).approve(address(adapter), specifiedAmount);
        }
        uint256 stETH_balance_before = IERC20(stETH).balanceOf(address(this));
        uint256 wstETH_balance_before = IERC20(wstETH).balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, wstETH, stETH, side, specifiedAmount);

        uint256 stETH_balance_after = IERC20(stETH).balanceOf(address(this));
        uint256 wstETH_balance_after = IERC20(wstETH).balanceOf(address(this));

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertApproxEqAbs(
                    specifiedAmount,
                    stETH_balance_after - stETH_balance_before,
                    2
                );
                assertEq(
                    trade.calculatedAmount,
                    wstETH_balance_before - wstETH_balance_after
                );
            } else {
                assertEq(
                    specifiedAmount,
                    wstETH_balance_before - wstETH_balance_after
                );
                assertApproxEqAbs(
                    trade.calculatedAmount,
                    stETH_balance_after - stETH_balance_before,
                    2
                );
            }
        }
    }

    function testSwapFuzzLidoEthStETH(uint256 specifiedAmount, bool isBuy)
        public
    {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        vm.assume(specifiedAmount > 1e10);

        uint256 stETH_balance_before;
        uint256 ETH_balance_before;

        bytes32 pair = bytes32(0);

        uint256[] memory limits = adapter.getLimits(pair, ETH, stETH);
        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1]);

            uint256 neededEth = specifiedAmount + (specifiedAmount / BUFFER);

            deal(address(this), neededEth);

            stETH_balance_before = IERC20(stETH).balanceOf(address(this));
            ETH_balance_before = address(this).balance;

            (bool sent_,) = address(adapter).call{value: neededEth}("");
            if (!sent_) revert();
        } else {
            vm.assume(specifiedAmount < limits[0]);

            deal(address(this), specifiedAmount);
            stETH_balance_before = IERC20(stETH).balanceOf(address(this));

            ETH_balance_before = address(this).balance;

            (bool sent_,) = address(adapter).call{value: specifiedAmount}("");
            if (!sent_) revert();
        }

        Trade memory trade =
            adapter.swap(pair, ETH, stETH, side, specifiedAmount);

        uint256 stETH_balance_after = IERC20(stETH).balanceOf(address(this));
        uint256 ETH_balance_after = address(this).balance;

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertApproxEqAbs(
                    specifiedAmount,
                    stETH_balance_after - stETH_balance_before,
                    2
                );
                assertEq(
                    trade.calculatedAmount,
                    ETH_balance_before - ETH_balance_after
                );
            } else {
                assertEq(
                    specifiedAmount, ETH_balance_before - ETH_balance_after
                );
                assertApproxEqAbs(
                    trade.calculatedAmount,
                    stETH_balance_after - stETH_balance_before,
                    2
                );
            }
        }
    }

    function testSwapFuzzLidoEthWstEth(uint256 specifiedAmount, bool isBuy)
        public
    {
        OrderSide side = isBuy ? OrderSide.Buy : OrderSide.Sell;
        vm.assume(specifiedAmount > 1e10);

        uint256 wstETH_balance_before;
        uint256 ETH_balance_before;

        bytes32 pair = bytes32(0);

        uint256[] memory limits = adapter.getLimits(pair, ETH, stETH);
        if (side == OrderSide.Buy) {
            vm.assume(specifiedAmount < limits[1]);

            uint256 neededEth =
                IStETH(stETH).getPooledEthByShares(specifiedAmount);
            neededEth = neededEth + (neededEth / BUFFER);
            deal(address(this), neededEth);
            wstETH_balance_before = IERC20(wstETH).balanceOf(address(this));
            ETH_balance_before = address(this).balance;
            (bool sent_,) = address(adapter).call{value: neededEth}("");
            if (!sent_) revert();
        } else {
            vm.assume(specifiedAmount < limits[0]);

            deal(address(this), specifiedAmount);

            wstETH_balance_before = IERC20(wstETH).balanceOf(address(this));
            ETH_balance_before = address(this).balance;

            (bool sent_,) = address(adapter).call{value: specifiedAmount}("");
            if (!sent_) revert();
        }
        Trade memory trade =
            adapter.swap(pair, ETH, wstETH, side, specifiedAmount);

        uint256 wstETH_balance_after = IERC20(wstETH).balanceOf(address(this));
        uint256 ETH_balance_after = address(this).balance;

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    wstETH_balance_after - wstETH_balance_before
                );
                assertEq(
                    trade.calculatedAmount,
                    ETH_balance_before - ETH_balance_after
                );
            } else {
                assertEq(
                    specifiedAmount, ETH_balance_before - ETH_balance_after
                );
                assertApproxEqAbs(
                    trade.calculatedAmount,
                    wstETH_balance_after - wstETH_balance_before,
                    2
                );
            }
        }
    }

    function testSwapSellIncreasingLido() public {
        executeIncreasingSwapsLido(OrderSide.Sell);
    }

    function executeIncreasingSwapsLido(OrderSide side) internal {
        bytes32 pair = bytes32(0);

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        uint256 specifiedAmount = 1e8;

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = specifiedAmount + (i * 1e6);
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            if (side == OrderSide.Buy) {
                uint256 neededWstEth =
                    IStETH(stETH).getSharesByPooledEth(amounts[i]);
                neededWstEth = neededWstEth + (neededWstEth / BUFFER);
                deal(wstETH, address(this), neededWstEth);
                IERC20(wstETH).approve(address(adapter), neededWstEth);
                trades[i] =
                    adapter.swap(pair, wstETH, stETH, side, neededWstEth);
                vm.revertTo(beforeSwap);
            } else {
                uint256 neededWstEth = amounts[i] + (amounts[i] / BUFFER);
                deal(wstETH, address(this), neededWstEth);
                IERC20(wstETH).approve(address(adapter), neededWstEth);
                trades[i] =
                    adapter.swap(pair, wstETH, stETH, side, neededWstEth);
                vm.revertTo(beforeSwap);
            }
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
        }
    }

    function testSwapBuyIncreasingLido() public {
        executeIncreasingSwapsLido(OrderSide.Buy);
    }

    function testGetCapabilitiesLido(bytes32 pair, address t0, address t1)
        public
        view
    {
        Capability[] memory res = adapter.getCapabilities(pair, t0, t1);

        assertEq(res.length, 5);
    }

    function testGetLimitsLido() public view {
        bytes32 pair = bytes32(0);
        uint256[] memory limits = adapter.getLimits(pair, ETH, stETH);

        assertEq(limits.length, 2);
    }
}
