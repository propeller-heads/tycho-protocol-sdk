// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@src/libraries/EfficientERC20.sol";
import "@src/libraries/PackedSwapStructs.sol";
import "@interfaces/ISwapExecutor.sol";
import "@interfaces/ISwapQuoter.sol";
import {PropellerRouterStructs} from "./PropellerRouterStructs.sol";

/**
 * @title SwapExecutionDispatcher - Dispatch swap execution to external contracts
 * @author PropellerHeads Devs
 * @dev Provides the ability to delegate execution of swaps to external
 *  contracts. This allows dynamically adding new supported protocols
 *  without needing to upgrade any contracts. External contracts will
 *  be called using delegatecall so they can share state with the main
 *  contract if needed.
 *
 *  Note Executor contracts need to implement the ISwapExecutor interface
 *  and can optionally implement ISwapQuoter.
 */
contract SwapExecutionDispatcher is PropellerRouterStructs {
    mapping(uint8 => address) public swapExecutors;

    using PackedSwapStructs for bytes;
    using EfficientERC20 for IERC20;

    /**
     * @dev Registers a new SwapExecutors contract for swap execution.
     */
    function _setSwapExecutor(uint8 id, address target) internal {
        swapExecutors[id] = target;
    }

    /**
     * @dev Set multiple executors with a single call.
     */
    function _setSwapExecutorBatch(SwapExecutorEntry[] calldata batch)
        internal
    {
        for (uint8 i = 0; i < (batch.length); i++) {
            swapExecutors[batch[i].id] = batch[i].executor;
        }
    }

    /**
     * @dev calls an executor, assumes swap.protocolData contains
     *  token addresses if required by the executor.
     */
    function _callSwapExecutor(
        uint8 exchange,
        uint256 amount,
        bytes calldata protocolDataIncludingTokens,
        bool quote
    ) internal returns (uint256 calculatedAmount) {
        bytes4 swapSelector = ISwapExecutor.swap.selector;
        bytes4 quoteSelector = ISwapQuoter.quote.selector;
        address method = swapExecutors[exchange];
        if (method != address(0)) {
            assembly {
                let ptr := mload(0x40)
                let inpSize := add(100, protocolDataIncludingTokens.length)
                // selector bytes
                switch quote
                case 1 { mstore(ptr, quoteSelector) }
                default { mstore(ptr, swapSelector) }
                // amount
                mstore(add(ptr, 4), amount)
                // offset for dynamic data
                mstore(add(ptr, 36), 64)
                // dynamic part
                // length of byte array
                mstore(add(ptr, 68), protocolDataIncludingTokens.length)
                // byte array contents
                calldatacopy(
                    add(ptr, 100),
                    protocolDataIncludingTokens.offset,
                    protocolDataIncludingTokens.length
                )
                if iszero(delegatecall(gas(), method, ptr, inpSize, ptr, 32)) {
                    // forward revert reason
                    let l := returndatasize()
                    returndatacopy(ptr, 0, l)
                    revert(ptr, l)
                }
                calculatedAmount := mload(ptr)
            }
        } else {
            revert UnknownExchangeMethod(exchange);
        }
    }
}
