// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import "@interfaces/batch-swap-router/IBatchSwapRouterV1Structs.sol";
import "@interfaces/ICallbackVerifier.sol";

/**
 * @title Dispatch callback verification to external contracts
 * @author PropellerHeads Devs
 * @dev Provides the ability to delegate callback verification to external
 *  contracts. This allows dynamically adding new supported protocols
 *  without needing to upgrade any contracts. External contracts will
 *  be called using delegatecall so they can share state with the main
 *  contract if needed.
 *
 *  Note Verifier contracts need to implement the ICallbackVerifier interface
 */
contract CallbackVerificationDispatcher is PropellerRouterStructs {
    mapping(bytes4 => address) public callbackVerifiers;

    /**
     * @dev Registers a new callback verifier contract.
     */
    function _setCallbackVerifier(bytes4 selector, address target) internal {
        callbackVerifiers[selector] = target;
    }

    /**
     * @dev Set multiple callback verifier with a single call.
     */
    function _setCallbackVerifierBatch(CallbackVerifierEntry[] calldata batch)
        internal
    {
        for (uint8 i = 0; i < (batch.length); i++) {
            callbackVerifiers[batch[i].selector] = batch[i].verifier;
        }
    }

    /**
     * @dev Calls a callback verifier. This should revert if the callback verification fails.
     * This function returns the offset of the GenericCallbackHeader in data.
     * This offset depends on the protocol and is required to parse the callback data in the fallback function.
     */
    function _callVerifyCallback(bytes calldata data)
        internal
        returns (
            uint256 amountOwed,
            uint256 amountReceived,
            address tokenOwed,
            uint16 dataOffset
        )
    {
        bytes4 verifySelector = ICallbackVerifier.verifyCallback.selector;
        address sender = msg.sender;
        address verifier = callbackVerifiers[bytes4(data[:4])];
        if (verifier != address(0)) {
            assembly {
                let ptr := mload(0x40)
                let inpSize := add(100, data.length)

                // function selector
                mstore(ptr, verifySelector)
                // sender
                mstore(add(ptr, 4), sender)
                // offset for dynamic data
                mstore(add(ptr, 36), 64)
                // dynamic part
                // length of byte array
                mstore(add(ptr, 68), data.length)
                // byte array contents
                calldatacopy(add(ptr, 100), add(data.offset, 4), data.length)

                //if delegate call successed
                if iszero(delegatecall(gas(), verifier, ptr, inpSize, ptr, 128))
                {
                    // forward revert reason
                    let retSize := returndatasize()
                    returndatacopy(ptr, 0, retSize)
                    revert(ptr, retSize)
                }
                // load returned data in res
                amountOwed := mload(ptr)
                amountReceived := mload(add(ptr, 32))
                tokenOwed := mload(add(ptr, 64))
                dataOffset := and(mload(add(ptr, 96)), 0xFFFF)
            }
        } else {
            revert UnknownSelector(bytes4(data[:4]));
        }
    }
}
