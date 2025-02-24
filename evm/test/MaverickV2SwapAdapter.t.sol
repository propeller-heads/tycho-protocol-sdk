// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";

contract MaverickV2SwapAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;

    function testPriceFuzz(uint256 amount0, uint256 amount1) public {}

    function testSwapFuzz(uint256 specifiedAmount) public {}
}
