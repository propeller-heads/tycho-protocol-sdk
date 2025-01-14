// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import "@interfaces/batch-swap-router/ICowSwapRouterPublic.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@src/libraries/PackedSwapStructs.sol";
import "@src/libraries/EfficientERC20.sol";
import "./SwapExecutionDispatcher.sol";
import "./CallbackVerificationDispatcher.sol";
import "./ApprovalManagement.sol";
import "./SwapContext.sol";

contract PropellerRouter is
    PropellerRouterStructs,
    SwapContext,
    SwapExecutionDispatcher,
    CallbackVerificationDispatcher,
    ApprovalManagement,
    AccessControl
{
    using PrefixLengthEncodedByteArray for bytes;
    using PackedSwapStructs for bytes;
    using EfficientERC20 for IERC20;

    //keccak256("EXECUTOR_ROLE") : save gas on deployment
    bytes32 public constant EXECUTOR_ROLE =
        0xd8aa0f3194971a2a116679f7c2090f6939c8d4e01a2a8d7e41d55e5351469e63;

    constructor() {
        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
    }

    function _executeSwap(
        address exchange,
        bytes calldata protocolData // includes fn selector
    ) internal override returns (uint256 calculatedAmount) {

        require(exchange != address(0), "Invalid exchange address");
        require(selector.length == 4, "Invalid function selector");
        bytes memory data = abi.encodePacked(selector, protocolData);
        (bool success, bytes memory returnData) = exchange.call(data);
        require(success, "Swap execution failed");

        // USV2 and USV3 will also be swap executors - no special case
        calculatedAmount = _callSwapExecutor(
            exchange, amount, protocolData, false
        );
        calculatedAmount = abi.decode(returnData, (uint256));
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

    /**
     * @dev Executes a single swap
     */
    function _singleSwap(uint256 amount, bytes calldata swap)
        internal
        returns (uint256 calculatedAmount)
    {
        calculatedAmount =
            _executeSwap(swap.exchange(), amount, swap.protocolData());
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

    function singleExactIn(uint256 givenAmount, bytes calldata swap)
        external
        override
        onlyRole(EXECUTOR_ROLE)
        withSwapContext
        returns (uint256 calculatedAmount)
    {
        (
            bool wrapEth, // This means ETH is the sell token
            bool unwrapEth, // This means ETH is the buy token
            uint256 minUserAmount,
            address tokenOut,
            address receiver,
            bytes calldata swap
        ) = data.decodeSingleCheckedArgs();

        uint256 balanceBefore;

        if (tokenOut == address(0)) {
            uint256 balanceBefore = payer.balance;
            _singleSwap(givenAmount, swap);
            calculatedAmount = balanceBefore - payer.balance;
        } else {
            uint256 balanceBefore = IERC20(tokenOut).balanceOf(payer);
            _singleSwap(givenAmount, swap);
            calculatedAmount = balanceBefore - IERC20(tokenOut).balanceOf(payer);
        }

        // Wrap ETH if it's the in token -> sender sends ETH -> wrap it before pool
        // TODO fix this wrapping unwrapping logic - it doesn't account for tokenIn
        if (wrapEth) {
            // In the case where we are wrapping ETH, the out token will be WETH
            _wrapETH(givenAmount);
            balanceBefore = tokenOut.balanceOf(address(this));
        }
            balanceBefore = tokenOut.balanceOf(receiver);
        }

        // PERFORM MAIN SWAP
         _singleSwap(givenAmount, swap);

        if (unwrapEth) {
            calculatedAmount = tokenOut.balanceOf(address(this)) - balanceBefore;
            _unwrapETH(calculatedAmount);
            address(receiver).transfer(calculatedAmount);
        } else {
            calculatedAmount = tokenOut.balanceOf(receiver) - balanceBefore;
        }

        // We used to get calculatedAmount from the final eth balance in the user
        // account

        if (calculatedAmount < minUserAmount) {
            revert NegativeSlippage(calculatedAmount, minUserAmount);
        }
    }

    function singleExactOut(uint256 givenAmount, bytes calldata swap)
        external
        override
        onlyRole(EXECUTOR_ROLE)
        withSwapContext
        returns (uint256 calculatedAmount)
    {
        (
            uint256 maxUserAmount,
            address tokenIn,
            address payer,
            bytes calldata swap
        ) = data.decodeSingleCheckedArgs();

        // We need to measure spent amount via balanceOf, as
        // callbacks might execute additional swaps

        // TODO deal with wraps and unwraps in this method
        if (tokenIn == address(0)) {
            uint256 balanceBefore = payer.balance;
            _singleSwap(givenAmount, swap);
            calculatedAmount = balanceBefore - payer.balance;
        } else {
            uint256 balanceBefore = IERC20(tokenIn).balanceOf(payer);
            _singleSwap(givenAmount, swap);
            calculatedAmount = balanceBefore - IERC20(tokenIn).balanceOf(payer);
        }

        if (calculatedAmount > maxUserAmount) {
            revert NegativeSlippage(calculatedAmount, maxUserAmount);
        }
    }

    /**
     * @dev Executes a sequence of exact in swaps, checking that the user gets more
     * than minUserAmount of buyToken.
     */
    function sequentialExactIn(
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
        uint8 exchange;
        bytes calldata swap;
        calculatedAmount = givenAmount;

        while (swaps.length > 0) {
            (swap, swaps) = swaps.next();
            exchange = swap.exchange();
            calculatedAmount =
                _executeSwap(exchange, calculatedAmount, swap.protocolData());
        }
        if (calculatedAmount < minUserAmount) {
            revert NegativeSlippage(calculatedAmount, minUserAmount);
        }
    }

    /**
     * @dev Executes a sequence of exact out swaps, by first quoting
     *  backwards and then executing with corrected amounts a
     *  sequential exactIn swap
     *
     * This method checks that the user spends no more than maxUserAmount of sellToken
     *  Note: All used executors must implement ISwapQuoter, for this
     *  method to work correctly.
     */
    function sequentialExactOut(
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
        // Idea: On v2, reserve 14 bytes for calculatedAmount and replace them here
        //  to save some quotes, if these 14 bytes are all zero the swap call won't
        //  recalculate the quote else, it will simply execute with the calculatedAmount
        //  that was passed along.
        // TODO: Check if replacing 14 bytes (requires us to construct new swap in mempy)
        //  amortises the repeated quoting (quoting again costs at least 500 gas, from
        //  a quick calculation it should amortise)
        bytes calldata swap;
        uint256[] memory amounts = new uint256[](swaps.length + 1);
        amounts[swaps.length] = givenAmount;

        // backwards pass to get correct in amount
        for (uint256 i = swaps.length; i > 0; i--) {
            swap = swaps[i - 1];
            amounts[i - 1] =
                _quoteSwap(swap.exchange(), amounts[i], swap.protocolData());
        }
        for (uint8 i = 0; i < swaps.length; i++) {
            swap = swaps[i];
            _executeSwap(swap.exchange(), amounts[i + 1], swap.protocolData());
        }
        calculatedAmount = amounts[0];

        if (calculatedAmount > maxUserAmount) {
            revert NegativeSlippage(calculatedAmount, maxUserAmount);
        }
    }

    /**
     * @dev Executes a swap graph with internal splits token amount
     *  splits, checking that the user gets more than minUserAmount of buyToken.
     *
     *  Assumes the swaps in swaps_ already contain any required token
     *  addresses.
     */
    function splitExactIn(
        uint256 amountIn,
        uint256 minUserAmount,
        SplitSwapExactInParameters calldata parameters
    )
        external
        override
        onlyRole(EXECUTOR_ROLE)
        withSwapContext
        returns (uint256 amountOut)
    {
        uint256 nTokens = parameters.nTokens;
        bytes calldata swaps_ = parameters.swaps;

        uint256 currentAmountIn;
        uint256 currentAmountOut;
        uint8 tokenInIndex;
        uint8 tokenOutIndex;
        uint24 split;
        bytes calldata swap;

        uint256[] memory remainingAmounts = new uint256[](nTokens);
        uint256[] memory amounts = new uint256[](nTokens);
        amounts[0] = amountIn;
        remainingAmounts[0] = amountIn;

        while (swaps_.length > 0) {
            (swap, swaps_) = swaps_.next();
            split = swap.splitPercentage();
            tokenInIndex = swap.tokenInIndex();
            tokenOutIndex = swap.tokenOutIndex();
            currentAmountIn = split > 0
                ? (amounts[tokenInIndex] * split) / 0xffffff
                : remainingAmounts[tokenInIndex];
            currentAmountOut = _executeSwap(
                swap.exchange(), currentAmountIn, swap.protocolData()
            );

            amounts[tokenOutIndex] += currentAmountOut;
            remainingAmounts[tokenOutIndex] += currentAmountOut;
            remainingAmounts[tokenInIndex] -= currentAmountIn;
        }
        calculatedAmount = amounts[tokenOutIndex];
        if (calculatedAmount < minUserAmount) {
            revert NegativeSlippage(calculatedAmount, minUserAmount);
        }
    }

    function batchExecute(bytes calldata data) internal {
        uint256 amount;
        bytes calldata actionData;
        bytes calldata batchElement;
        ActionType actionType;
        while (data.length > 0) {
            (batchElement, data) = data.next();
            (actionType, actionData) = batchElement.decodeBatchExecute();
            _executeAction(actionType, actionData);
        }
    }

    /**
     * @dev Allows to route amount into any other action type supported
     * by this contract. This allows for more flexibility during
     * batchExecute or within callbacks.
     *  @param amount the amount to forward into the next action
     *  @param type_ what kind of action to take
     *  @param actionData data with the encoding for each action. See the
     *      individual methods for more information.
     */
    function _executeAction(
        uint256 amount,
        ActionType type_,
        bytes calldata actionData
    ) internal returns (uint256 calculatedAmount) {
        if (type_ == ActionType.SINGLE_IN) {
            (uint256 amount, uint256 checkAmount, bytes calldata swaps) =
                actionData.decodeAmountAndBytes();
            calculatedAmount = singleExactIn(amount, checkAmount, swaps);
        } else if (type_ == ActionType.SINGLE_OUT) {
            (uint256 amount, uint256 checkAmount, bytes calldata swaps) =
                actionData.decodeAmountAndBytes();
            calculatedAmount = singleExactOut(amount, actionData);
        } else if (type_ == ActionType.SEQUENTIAL_IN) {
            (uint256 amount, uint256 checkAmount, bytes calldata swaps) =
                actionData.decodeAmountAndBytes();
            calculatedAmount =
                sequentialExactIn(amount, checkAmount, swaps);
        } else if (type_ == ActionType.SEQUENTIAL_OUT) {
            (uint256 amount, uint256 checkAmount, bytes calldata swaps) =
                actionData.decodeAmountAndBytes();
            calculatedAmount =
                sequentialExactOut(amount, checkAmount, swaps);
        } else if (type_ == ActionType.SPLIT_IN) {
            (uint256 amount, uint256 checkAmount, bytes calldata swaps) =
                actionData.decodeAmountAndBytes();
            calculatedAmount = splitExactIn(
                amount, checkAmount, swaps
            );
        } else {
            revert UnsupportedBatchData(uint8(type_));
        }
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


    /**
     * @dev Transfers ERC20 tokens or ETH out. Meant to transfer to the final receiver.
     *
     *  Note Can also transfer the complete balance but keeping 1
     *  atomic unit dust for gas optimisation reasons. This is
     *  automatically done if the transfer amount is 0.
     */
    function _transfer(uint256 amount, address receiver, IERC20 token)
        internal
    {
        if (address(token) == address(0)) {
            _transferNative(amount, payable(receiver));
        } else {
            if (amount == 0) {
                token.transferBalanceLeavingDust(receiver);
            } else {
                token.safeTransfer(receiver, amount);
            }
        }
    }

    /**
     * @dev Transfers ETH out. Meant to transfer to the final receiver.
     */
    function _transferNative(uint256 amount, address payable receiver)
        internal
    {
        bool sent;
        // ETH transfer via call see https://solidity-by-example.org/sending-ether/
        assembly {
            sent := call(gas(), receiver, amount, 0, 0, 0, 0)
        }
        if (!sent) {
            revert InvalidTransfer(receiver, address(0), amount);
        }
    }


    /**
     * @dev Wrap a defined amount of ETH.
     * @param amount of native ETH to wrap.
     */
    function _wrapETH(uint256 amount) internal {
        if (msg.value > 0 && msg.value != amount) {
            revert MessageValueMismatch(msg.value, amount);
        }
        _weth.deposit{value: amount}();
    }

    /**
     * @dev Unwrap a defined amount of WETH.
     * @param amount of WETH to unwrap.
     */
    function _unwrapETH(uint256 amount) internal {
        uint256 unwrapAmount =
            amount == 0 ? _weth.balanceOf(address(this)) : amount;
        _weth.withdraw(unwrapAmount);
    }
}
