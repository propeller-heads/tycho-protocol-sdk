// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import "@interfaces/batch-swap-router/ICowSwapRouterPublic.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@src/libraries/PackedSwapStructs.sol";
import "@src/libraries/EfficientERC20.sol";
import "./PropellerRouterInternal.sol";
import "./SwapExecutionDispatcher.sol";
import "./CallbackVerificationDispatcher.sol";
import "./ApprovalManagement.sol";
import "./SwapContext.sol";

contract PropellerRouter is
    PropellerRouterInternal,
    SwapContext,
    SwapExecutionDispatcher,
    CallbackVerificationDispatcher,
    ApprovalManagement,
    AccessControl
{
    using PackedSwapStructs for bytes;
    using EfficientERC20 for IERC20;

    //keccak256("EXECUTOR_ROLE") : save gas on deployment
    bytes32 public constant EXECUTOR_ROLE =
        0xd8aa0f3194971a2a116679f7c2090f6939c8d4e01a2a8d7e41d55e5351469e63;

    constructor() {
        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
    }

    function _executeSwap(
        uint8 exchange,
        uint256 amount,
        bytes calldata protocolData
    ) internal override returns (uint256 calculatedAmount) {
        // USV2 and USV3 will also be swap executors - no special case
        calculatedAmount = _callSwapExecutor(
            exchange, amount, protocolData, false
        );
    }

    function _verifyCallback(bytes calldata data)
        internal
        override
        returns (
            uint256 amountOwed,
            uint256 amountReceived,
            address tokenOwed,
            uint16 dataOffset
        )
    {
        (amountOwed, amountReceived, tokenOwed, dataOffset) =
            _callVerifyCallback(data);
    }

    function _quoteSwap(
        uint8 exchange,
        uint256 amount,
        bytes calldata protocolData
    ) internal override returns (uint256 calculatedAmount) {
        // USV2 and USV3 will also be swap executors - no special case
        calculatedAmount = _callSwapExecutor(
            exchange, amount, protocolData, true
        );
    }

    function singleExactInChecked(uint256 givenAmount, bytes calldata swap)
        external
        override
        onlyRole(EXECUTOR_ROLE)
        withSwapContext
        returns (uint256 calculatedAmount)
    {
        calculatedAmount = _singleExactInChecked(givenAmount, swap);
    }

    function singleExactOutChecked(uint256 givenAmount, bytes calldata swap)
        external
        override
        onlyRole(EXECUTOR_ROLE)
        withSwapContext
        returns (uint256 calculatedAmount)
    {
        calculatedAmount = _singleExactOutChecked(givenAmount, swap);
    }

    function sequentialExactInChecked(
        uint256 givenAmount,
        uint256 minUserAmount,
        bytes calldata swaps
    )
        external
        override
        onlyRole(EXECUTOR_ROLE)
        withSwapContext
        returns (uint256 calculatedAmount)
    {
        calculatedAmount =
            _sequentialExactInChecked(givenAmount, minUserAmount, swaps);
    }

    function sequentialExactOutChecked(
        uint256 givenAmount,
        uint256 maxUserAmount,
        bytes[] calldata swaps
    )
        external
        override
        onlyRole(EXECUTOR_ROLE)
        withSwapContext
        returns (uint256 calculatedAmount)
    {
        calculatedAmount =
            _sequentialExactOutChecked(givenAmount, maxUserAmount, swaps);
    }

    function splitExactInChecked(
        uint256 amount,
        uint256 minUserAmount,
        SplitSwapExactInParameters calldata parameters
    )
        external
        override
        onlyRole(EXECUTOR_ROLE)
        withSwapContext
        returns (uint256 amountOut)
    {
        amountOut = _splitExactInChecked(
            amount, minUserAmount, parameters.nTokens, parameters.swaps
        );
    }

    /**
     * @dev Entrypoint to add or replace a swap method contract address
     * @param id for this method
     * @param target address of the swap method contract
     */
    function setExecutorMethod(uint8 id, address target)
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
    {
        _setSwapExecutor(id, target);
    }

    /**
     * @dev Entrypoint to add or replace multiple swap executor contract address
     * @param batch one entry per method
     */
    function setSwapExecutorBatch(SwapExecutorEntry[] calldata batch)
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
    {
        _setSwapExecutorBatch(batch);
    }

    /**
     * @dev Entrypoint to add or replace a callback verifier contract address
     * @param selector for this method
     * @param target address of the swap method contract
     */
    function setCallbackVerifier(bytes4 selector, address target)
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
    {
        _setCallbackVerifier(selector, target);
    }

    /**
     * @dev Entrypoint to add or replace multiple callback verifier contract address
     * @param batch one entry per method
     */
    function setCallbackVerifierBatch(CallbackVerifierEntry[] calldata batch)
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
    {
        _setCallbackVerifierBatch(batch);
    }

    /**
     * @dev Entrypoint to set allowances for multiple addresses on a set of ERC20 tokens
     * @param approvals an array of ExternalApproval structs, each of which specifies a token, an allowance, and an array of addresses for which the allowance should be set
     *
     * This function will iterate over each ExternalApproval, and for each, it will iterate over the provided addresses,
     * calling the safeApprove function to set the provided allowance on the token for each address.
     */
    function setApprovals(ExternalApproval[] calldata approvals)
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
    {
        _setApprovals(approvals);
    }

    /**
     * @dev We use the fallback function to allow flexibility on callback.
     */
    fallback() external {
        // TODO
    }

    /**
     * @dev Allows granting roles to multiple accounts in a single call.
     */
    function batchGrantRole(bytes32 role, address[] memory accounts)
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
    {
        for (uint256 i = 0; i < accounts.length; i++) {
            _grantRole(role, accounts[i]);
        }
    }

    /**
     * @dev Allows withdrawing any ERC20 funds if funds get stuck in case of a bug,
     * the contract should every only hold dust amounts of tokens for
     * security reasons.
     */
    function withdraw(IERC20[] memory tokens, address receiver)
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
    {
        for (uint256 i = 0; i < tokens.length; i++) {
            uint256 tokenBalance = tokens[i].balanceOf(address(this));
            tokens[i].safeTransfer(receiver, tokenBalance);
        }
    }

    /**
     * @dev Allows withdrawing any ETH funds if funds get stuck in case of a bug,
     * the contract should never hold any ETH for security reasons.
     */
    function withdrawETH() external onlyRole(DEFAULT_ADMIN_ROLE) {
        (bool success,) = msg.sender.call{value: address(this).balance}("");
        require(success);
    }

    /**
     * @dev Allows this contract to receive native token
     */
    receive() external payable {}
}
