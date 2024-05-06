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

    address constant SAVINGS_DAI_ADDRESS = 0x83F20F44975D03b1b09e64809B757c47f942BEeA;

    function setUp() public {
        uint256 forkBlock = 18835309;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new sDaiSwapAdapter(SAVINGS_DAI_ADDRESS);
    }

    function testGetAssetAddress() public {
        address dai = adapter.getAssetAddress();
        console.log("Dai address", dai);
    }

}
