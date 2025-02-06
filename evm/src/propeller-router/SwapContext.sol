// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@src/libraries/EfficientERC20.sol";

/**
 * @title SwapContext
 * @author PropellerHeads Devs
 * @dev This contract is used to make assertions on the initial caller.
 * As msg.sender can change if the execution involve subcalls (for exemple a callback), we need
 * to store the initial caller to be able to retrieve it at any time.
 *
 * Note: This is currently use as a security for transferFrom.
 */
contract SwapContextStorage {
    address internal _currentSender;
}

contract SwapContext is SwapContextStorage {
    modifier withSwapContext() {
        _currentSender = msg.sender;
        _;
        delete _currentSender;
    }
}
