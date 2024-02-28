// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "forge-std/console.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";
import "src/frax-v3-frxEth/FraxV3FrxEthAdapter.sol";

contract FraxV3FrxEthAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    FraxV3FrxEthAdapter adapter; 

    address constant FRAXETH_ADDRESS = 0x5E8422345238F34275888049021821E8E08CAa1f;
    address constant SFRAXETH_ADDRESS = 0xac3E018457B222d93114458476f3E3416Abbe38F;
    address constant FRAXETHMINTER_ADDRESS = 0xbAFA44EFE7901E04E39Dad13167D089C559c1138;

    IERC20 constant FRAXETH = IERC20(0x5E8422345238F34275888049021821E8E08CAa1f);
    IERC20 constant SFRAXETH = IERC20(0xac3E018457B222d93114458476f3E3416Abbe38F);
    IERC20 constant ETH = IERC20(address(0));
    IERC20 constant WBTC = IERC20(0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599);

    uint256 constant TEST_ITERATIONS = 10;
    uint256 constant AMOUNT0 = 1000000000000000000;
    bytes32 constant PAIR = bytes32(0);

    function setUp() public {
        uint256 forkBlock = 19327125;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);

        adapter = new FraxV3FrxEthAdapter(FRAXETH_ADDRESS, FRAXETHMINTER_ADDRESS, SFRAXETH_ADDRESS);
    }

    /// @dev The price is kept among swaps if no FRAX rewards are distributed in
    /// the contract during time
    function testPriceKeepingSellFraxEthFraxEthV3() public {
        bytes32 pair = bytes32(0);
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * (i + 1) * 10 ** 18;
        }

        Fraction[] memory prices =
            adapter.price(PAIR, FRAXETH, SFRAXETH, amounts);

        for (uint256 i = 0; i < TEST_ITERATIONS -1; i++) {

            // console.log("Iteration: ", i);
            // console.log("AmountIn: ", amounts[i]);
            // console.log("Denominator: ", prices[i].denominator);
            assertEq(prices[i].compareFractions(prices[i + 1]), 0);
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    function testPriceKeepingSellSFraxEthFraxEthV3() public {
        bytes32 pair = bytes32(0);
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * (i + 1) * 10 ** 18;
        }

        Fraction[] memory prices =
            adapter.price(PAIR, SFRAXETH, FRAXETH, amounts);

        for (uint256 i = 0; i < TEST_ITERATIONS -1; i++) {

            assertEq(prices[i].compareFractions(prices[i + 1]), 0);
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    function testPriceKeepingSellEthFraxEthV3() public {
        bytes32 pair = bytes32(0);
        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);

        for (uint256 i = 0; i < TEST_ITERATIONS; i++) {
            amounts[i] = 1000 * (i + 1) * 10 ** 18;
        }

        Fraction[] memory prices =
            adapter.price(PAIR, ETH, SFRAXETH, amounts);

        for (uint256 i = 0; i < TEST_ITERATIONS -1; i++) {

            assertEq(prices[i].compareFractions(prices[i + 1]), 0);
            assertGt(prices[i].denominator, 0);
            assertGt(prices[i + 1].denominator, 0);
        }
    }

    function testGetTokensFraxEthV3() public {
        IERC20[] memory tokens = adapter.getTokens(bytes32(0));

        assertEq(address(tokens[0]), address(0));
        assertEq(address(tokens[1]), FRAXETH_ADDRESS);
        assertEq(address(tokens[2]), SFRAXETH_ADDRESS);
    }
    
    function testGetLimitsFraxEthV3() public {
    uint256[] memory limits =
        adapter.getLimits(bytes32(0), FRAXETH, SFRAXETH);
    assertEq(limits.length, 2);
    }

    function testGetCapabilitiesFraxEthV3() public {
        Capability[] memory res =
            adapter.getCapabilities(bytes32(0), ETH, FRAXETH);

        assertEq(res.length, 3);
    }

}