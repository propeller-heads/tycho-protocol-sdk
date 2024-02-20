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

    function setUp() public {
        uint256 forkBlock = 19268842;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);

        adapter = new FraxV3SFraxAdapter(ISFRAX);
    }

    function testGetLimitsFraxV3SFrax() public {
        uint256[] memory limits = adapter.getLimits(bytes32(0), FRAX, SFRAX);
        assertEq(limits.length, 2);
    }

}