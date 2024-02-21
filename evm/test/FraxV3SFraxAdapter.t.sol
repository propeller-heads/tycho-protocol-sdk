// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "forge-std/console.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";
import "src/libraries/FractionMath.sol";
import "src/frax-v3/FraxV3SFraxAdapter.sol";


/// @title TemplateSwapAdapterTest
/// @dev This is a template for a swap adapter test.
/// Test all functions that are implemented in your swap adapter, the two test included here are just an example.
/// Feel free to use UniswapV2SwapAdapterTest and BalancerV2SwapAdapterTest as a reference.
contract FraxV3SFraxAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    FraxV3SFraxAdapter adapter;
    ISFrax constant ISFRAX = ISFrax(0xA663B02CF0a4b149d2aD41910CB81e23e1c41c32);
    IERC20 constant FRAX = IERC20(0x853d955aCEf822Db058eb8505911ED77F175b99e);
    IERC20 constant SFRAX = IERC20(address(ISFRAX));
    address constant FRAX_ADDRESS = address(FRAX);
    address constant SFRAX_ADDRESS = address(SFRAX);

    uint256 constant TEST_ITERATIONS = 100;
    uint256 constant AMOUNT0 = 1000000000000000000;

    function setUp() public {
        uint256 forkBlock = 19270612;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);

        adapter = new FraxV3SFraxAdapter(ISFRAX);
    }

    /// @dev set lower limit to greater than 1, because previewDeposit returns 0
    /// with an amountIn == 1
    function testPriceFuzzFraxV3SFrax(uint256 amount0, uint256 amount1) public {
        uint256[] memory limits = adapter.getLimits(bytes32(0), FRAX, SFRAX);
        vm.assume(amount0 < limits[0]);
        vm.assume(amount0 > 1);
        vm.assume(amount1 < limits[1]);
        vm.assume(amount1 > 1);

        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;

        Fraction[] memory prices = adapter.price(bytes32(0), FRAX, SFRAX, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testOneIncreasingPriceFoundFraxV3SFrax() public {
        
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        for (uint256 i = 1; i < TEST_ITERATIONS + 1; i++) {
            amounts[i-1] = 1000 * i * 10 ** 18;
        }

        Fraction[] memory prices = adapter.price(bytes32(0), FRAX, SFRAX, amounts);

        bool foundIncreasingPrice = false; // Flag variable to track if increasing price is found

        for (uint256 i = 0; i < TEST_ITERATIONS - 1; i++) {
            if (prices[i].compareFractions(prices[i + 1]) == 1) {
                foundIncreasingPrice = true;
                break; // If one increasing price is found, we can exit the loop
            }
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }

        // Assert that at least one increasing price is found
        assertTrue(foundIncreasingPrice, "No increasing price found");

    }

    function testGetLimitsFraxV3SFrax() public {
        uint256[] memory limits = adapter.getLimits(bytes32(0), FRAX, SFRAX);
        console.logUint(limits[0]);
        console.logUint(limits[1]);
        assertEq(limits.length, 2);
    }

    function testGetTokensFraxV3SFrax() public {
        IERC20[] memory tokens = adapter.getTokens(bytes32(0));

        assertEq(address(tokens[0]), FRAX_ADDRESS);
        assertEq(address(tokens[1]), SFRAX_ADDRESS);
    }

    function testGetCapabilitiesFraxV3SFrax() public {
    Capability[] memory res =
        adapter.getCapabilities(bytes32(0), FRAX, SFRAX);

    assertEq(res.length, 3);
    }

    function testGetAmountOutSFrax() public view {
        uint256 amountInFrax = 1;
        uint256 amountOutSFrax = ISFRAX.previewDeposit(amountInFrax);

        console.log("FRAX in:", amountInFrax);
        console.log("SFRAX out:", amountOutSFrax);

        assert(amountOutSFrax > 0);
    }

    function testGetAmountOutFrax() public view {
        uint256 amountInSFrax = AMOUNT0;
        uint256 amountOutFrax = ISFRAX.previewRedeem(amountInSFrax);

        console.log("SFRAX in:", amountInSFrax);
        console.log("FRAX out:", amountOutFrax);

        assert(amountOutFrax > 0);
    }

    function testGetAmountInFrax() public view {
        uint256 amountOutSFrax = AMOUNT0;
        uint256 amountInFrax = ISFRAX.previewMint(amountOutSFrax);

        console.log("SFRAX out:", amountOutSFrax);
        console.log("FRAX in:", amountInFrax);

        assert(amountInFrax > 0);
    }

    function testGetAmountInSFrax() public view {
        uint256 amountOutFrax = AMOUNT0;
        uint256 amountInSFrax = ISFRAX.previewWithdraw(amountOutFrax);

        console.log("FRAX out:", amountOutFrax);
        console.log("SFRAX in:", amountInSFrax);

        assert(amountInSFrax > 0);
    }

    function testGetPriceAtFraxV3SFrax() public {

        uint256 amountInFrax = AMOUNT0;
        Fraction memory fractionFraxIn = adapter.getPriceAt(FRAX, amountInFrax);

        uint256 amountInSFrax = AMOUNT0;
        Fraction memory fractionSFraxIn = adapter.getPriceAt(SFRAX, amountInSFrax);

        console.log("Numerator Frax In: ", fractionFraxIn.numerator);
        console.log("Denominator Frax In: ", fractionFraxIn.denominator);
        console.log("---------------------SFRAX IN--------------------------------");
        console.log("Numerator SFrax In: ", fractionSFraxIn.numerator);
        console.log("Denominator SFrax In: ", fractionSFraxIn.denominator);

        assertGt(fractionFraxIn.numerator, 0);
        assertGt(fractionFraxIn.denominator, 0);

        assertGt(fractionSFraxIn.numerator, 0);
        assertGt(fractionSFraxIn.denominator, 0);
    }

}