// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@src/libraries/PackedSwapStructs.sol";
import "@src/libraries/EfficientERC20.sol";
import "@src/libraries/PrefixLengthEncodedByteArray.sol";
import "./PropellerRouterStructs.sol";
/**
 * @title PropellerRouterInternal - private router methods
 * @author PropellerHeads Devs
 * @dev This contract contains generic routing logic for ERC20 token swaps.
 *  You need to specify how to execute swaps, this is usually achieved with
 *  the SwapExecutionDispatcher but it might make sense to include common
 *  protocols ("hot paths") directly in the main contracts byte code.
 */

abstract contract PropellerRouterInternal is PropellerRouterStructs {
    address private constant _USV3_FACTORY =
        0x1F98431c8aD98523631AE4a59f267346ea31F984;

    using EfficientERC20 for IERC20;
    using PrefixLengthEncodedByteArray for bytes;
    using PackedSwapStructs for bytes;

    /**
     * @dev Specify here how to execute a single swap, it is
     *  expected to have all token addresses required already present
     *  in protocolDataIncludingTokens. Should return the received amount.
     */
    function _executeSwap(
        uint8 exchange,
        uint256 amount,
        bytes calldata protocolDataIncludingTokens
    ) internal virtual returns (uint256 calculatedAmount);

    /**
     * @dev Same as _executeSwap but will only quote the calculatedAmount,
     * should not actually execute the swap.
     */
    function _quoteSwap(
        uint8 exchange,
        uint256 amount,
        bytes calldata protocolDataIncludingTokens
    ) internal virtual returns (uint256 calculatedAmount);

    /**
     * @dev Executes a single swap, but might do more actions through callbacks.
     */
    function _singleSwap(uint256 amount, bytes calldata swap)
        internal
        returns (uint256 calculatedAmount)
    {
        calculatedAmount =
            _executeSwap(swap.exchange(), amount, swap.protocolData());
    }

    function _singleExactOutChecked(uint256 amount, bytes calldata data)
        internal
        returns (uint256 calculatedAmount)
    {
        (
            uint256 maxUserAmount,
            IERC20 tokenIn,
            address payer,
            bytes calldata swap
        ) = data.decodeSingleCheckedArgs();
        // We need to measure spent amount via balanceOf, as
        // callbacks might execute additional swaps
        uint256 balanceBefore = tokenIn.balanceOf(payer);
        _singleSwap(amount, swap);
        calculatedAmount = balanceBefore - tokenIn.balanceOf(payer);
        if (calculatedAmount > maxUserAmount) {
            revert NegativeSlippage(calculatedAmount, maxUserAmount);
        }
    }

    function _singleExactInChecked(uint256 amount, bytes calldata data)
        internal
        returns (uint256 calculatedAmount)
    {
        (
            uint256 minUserAmount,
            IERC20 tokenOut,
            address receiver,
            bytes calldata swap
        ) = data.decodeSingleCheckedArgs();
        // We need to measure spent amount via balanceOf, as
        // callbacks might execute additional swaps
        uint256 balanceBefore = tokenOut.balanceOf(receiver);
        _singleSwap(amount, swap);
        calculatedAmount = tokenOut.balanceOf(receiver) - balanceBefore;
        if (calculatedAmount < minUserAmount) {
            revert NegativeSlippage(calculatedAmount, minUserAmount);
        }
    }

    /**
     * @dev Executes a sequence of exact in swaps
     */
    function _sequentialSwapExactIn(uint256 givenAmount, bytes calldata swaps)
        internal
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
    }

    /**
     * @dev Executes a sequence of exact out swaps, by first quoting
     *  backwards and then executing with corrected amounts a
     *  sequential exactIn swap
     *
     *  Note: All used executors must implement ISwapQuoter, for this
     *  method to work correctly.
     */
    function _sequentialSwapExactOut(
        uint256 givenAmount,
        bytes[] calldata swaps
    ) internal returns (uint256) {
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
        return amounts[0];
    }

    /**
     * @dev Same as sequentialExactOut but checks that the user
     * spends less than maxUserAmount of sellToken.
     */
    function _sequentialExactOutChecked(
        uint256 givenAmount,
        uint256 maxUserAmount,
        bytes[] calldata swaps
    ) internal returns (uint256 calculatedAmount) {
        calculatedAmount = _sequentialSwapExactOut(givenAmount, swaps);
        if (calculatedAmount > maxUserAmount) {
            revert NegativeSlippage(calculatedAmount, maxUserAmount);
        }
    }

    /**
     * @dev same as sequentialExactIn but checks that the user gets
     * more than minUserAmount of buyToken.
     */
    function _sequentialExactInChecked(
        uint256 givenAmount,
        uint256 minUserAmount,
        bytes calldata swaps
    ) internal returns (uint256 calculatedAmount) {
        calculatedAmount = _sequentialSwapExactIn(givenAmount, swaps);
        if (calculatedAmount < minUserAmount) {
            revert NegativeSlippage(calculatedAmount, minUserAmount);
        }
    }

    /**
     * @dev same as _splitSwapWithDataInclToken but checks that the user gets
     * more than minUserAmount of buyToken.
     */
    function _splitExactInChecked(
        uint256 amountIn,
        uint256 minUserAmount,
        uint256 nTokens,
        bytes calldata swaps_
    ) internal returns (uint256 calculatedAmount) {
        calculatedAmount =
            _splitSwapExactIn(amountIn, nTokens, swaps_);
        if (calculatedAmount < minUserAmount) {
            revert NegativeSlippage(calculatedAmount, minUserAmount);
        }
    }
        /**
     * @dev Executes a swap graph with internal splits token amount
     *  splits.
     *
     *  Assumes the swaps in swaps_ already contain any required token
     *  addresses.
     */
    function _splitSwapExactIn(
        uint256 amountIn,
        uint256 nTokens,
        bytes calldata swaps_
    ) internal returns (uint256) {
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
        return amounts[tokenOutIndex];
    }

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
        if (type_ == ActionType.SINGLE_IN_CHECKED) {
            calculatedAmount = _singleExactInChecked(amount, actionData);
        } else if (type_ == ActionType.SINGLE_OUT_CHECKED) {
            calculatedAmount = _singleExactOutChecked(amount, actionData);
        } else if (type_ == ActionType.SEQUENTIAL_IN_CHECKED) {
            (uint256 checkAmount, bytes calldata swaps) =
                actionData.decodeAmountAndBytes();
            calculatedAmount =
                _sequentialExactInChecked(amount, checkAmount, swaps);
        } else if (type_ == ActionType.SEQUENTIAL_OUT_CHECKED) {
            (uint256 checkAmount, bytes[] calldata swaps) =
                actionData.decodeAmountAndSwapArray();
            calculatedAmount =
                _sequentialExactOutChecked(amount, checkAmount, swaps);
        } else if (type_ == ActionType.SPLIT_EXACT_IN_CHECKED) {
            (
                uint256 checkAmount,
                IERC20[] calldata tokens,
                bytes[] calldata swaps
            ) = actionData.decodeSplitSwapWithTokenArrayCheckedArgs();
            calculatedAmount = _splitSwapWithoutTokensChecked(
                amount, checkAmount, tokens, swaps
            );
        }
        // No more action type = transfer. AFAIK this was used for USV3 callbacks
        // and is unnecessary if we simplify the USV3 executor logic.
        else {
            revert UnsupportedBatchData(uint8(type_));
        }
    }
}
