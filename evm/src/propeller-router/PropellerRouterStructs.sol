// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

interface PropellerRouterStructs {
    error AmountTooLow();
    error ZeroAmount();
    error UnsupportedBatchData(uint8 actionType);
    error UnknownExchangeMethod(uint8 method);
    error UnknownSelector(bytes4 selector);
    error CallbackVerificationFailed();
    error InvalidTransfer(address to, address token, uint256 amount);
    error InvalidData(string);
    error NegativeSlippage(uint256 got, uint256 expected);
    error MessageValueMismatch(uint256 value, uint256 amount);

    /**
     * @dev parameters for executing a split swap, tokens must be included in swaps protocolData
     *
     * The swap attribute holds the encoded swap with `protocolDataIncludingTokenData`.
     * For `protocolDataIncludingTokenData` see the individual SwapMethods.
     * For encoding of the swap see the `CompressedSwapBytesV2` library.
     *
     * The swaps will be executed in order. So they must be ordered such that all
     *  transfers have been
     *  completed before starting swaps that spend the token.
     */
    struct SplitSwapExactInParameters {
        uint256 nTokens;
        bytes swaps;
    }

    /**
     * @dev parameters for adding a new swapExecutor
     * `id` the uint8 id of this executor
     * `executor` the contract address of this executor
     */
    struct SwapExecutorEntry {
        uint8 id;
        address executor;
    }

    /**
     * @dev parameters for adding a new callback verification logic
     * `selector` the bytes4 signature of the callback function
     * `verificationExecutor` the contract address of this verification executor
     */
    struct CallbackVerifierEntry {
        bytes4 selector;
        address verifier;
    }

    /**
     * @dev parameters for adding new token approvals
     * `token` the token to approve
     * `addresses` the addresses that we want to give allowance to
     * `allowance` the amount we allow
     */
    struct ExternalApproval {
        IERC20 token;
        address[] addresses;
        uint256 allowance;
    }
}
