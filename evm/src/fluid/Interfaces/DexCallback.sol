// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;
interface IDexCallback {
    function dexCallback(address token_, uint256 amount_) external;
}
