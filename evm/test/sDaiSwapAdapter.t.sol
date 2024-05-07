// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";
import "src/sDai/sDaiSwapAdapter.sol";
import "forge-std/console.sol";
import "forge-std/console2.sol";

/// @title sDaiSwapAdapterTest

contract sDaiSwapAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    sDaiSwapAdapter adapter;
    ISavingsDai savingsDai;

    address constant DAI_ADDRESS = 0x6B175474E89094C44Da98b954EedeAC495271d0F;
    address constant SAVINGS_DAI_ADDRESS = 0x83F20F44975D03b1b09e64809B757c47f942BEeA;
    address constant SAVINGS_DAI_PARAMETERS_ADDRESS = 0x197E90f9FAD81970bA7976f33CbD77088E5D7cf7;
    
    bytes32 constant PAIR = bytes32(0);

    function setUp() public {
        uint256 forkBlock = 18835309;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new sDaiSwapAdapter(SAVINGS_DAI_ADDRESS);
    }

    function testGetTokensSDai() public {
        address[] memory tokens = adapter.getTokens(PAIR);

        assertEq(tokens[0], DAI_ADDRESS);
        assertEq(tokens[1], SAVINGS_DAI_ADDRESS);
    }

    function testGetAssetAddress() public {
        address dai = adapter.getAssetAddress();
        console.log("Dai address", dai);
    }

}
