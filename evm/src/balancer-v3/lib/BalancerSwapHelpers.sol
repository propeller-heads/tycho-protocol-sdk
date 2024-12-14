//SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.26;

import "./BalancerERC4626Helpers.sol";

/**
 * @title Balancer V3 Swap Helpers
 * @dev A wrapped library containing swap functions, helpers and storage for the
 * Balancer V3 Swap Adapter contract
 */
abstract contract BalancerSwapHelpers is BalancerERC4626Helpers, ISwapAdapter {
    function getAmountOutMiddleware(
        bytes32 pool,
        address sellToken,
        address buyToken,
        uint256 specifiedAmount
    ) internal returns (uint256 amountOut) {
        address poolAddress = address(bytes20(pool));

        (
            CUSTOM_WRAP_KIND kindWrap,
            address sellTokenOutput,
            address buyTokenOutput
        ) = getCustomWrap(sellToken, buyToken, poolAddress);

        if (kindWrap != CUSTOM_WRAP_KIND.NONE) {
            return
                getAmountOutCustomWrap(
                    poolAddress,
                    sellToken,
                    specifiedAmount,
                    kindWrap,
                    sellTokenOutput,
                    buyTokenOutput
                );
        } else {
            if (
                (isERC4626(sellToken) && !isERC4626(buyToken)) ||
                (!isERC4626(sellToken) && isERC4626(buyToken))
            ) {
                (
                    ERC4626_SWAP_TYPE kind,
                    address outputAddress
                ) = getERC4626PathType(poolAddress, sellToken, buyToken);

                if (isERC4626(sellToken) && !isERC4626(buyToken)) {
                    return
                        getAmountOutERC4626ForERC20(
                            poolAddress,
                            sellToken,
                            buyToken,
                            specifiedAmount,
                            kind,
                            outputAddress
                        );
                } else {
                    return
                        getAmountOutERC20ForERC4626(
                            poolAddress,
                            sellToken,
                            buyToken,
                            specifiedAmount,
                            kind,
                            outputAddress
                        );
                }
            }
        }

        (
            IBatchRouter.SwapPathExactAmountIn memory sellPath,
            ,

        ) = createERC20Path(
                poolAddress,
                IERC20(sellToken),
                IERC20(buyToken),
                specifiedAmount,
                false
            );
        return getAmountOut(sellPath);
    }

    /**
     * @notice Middleware for swaps
     */
    function swapMiddleware(
        bytes32 pool,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) internal returns (uint256) {
        address poolAddress = address(bytes20(pool));

        (
            CUSTOM_WRAP_KIND kindWrap,
            address sellTokenOutput,
            address buyTokenOutput
        ) = getCustomWrap(sellToken, buyToken, poolAddress);

        if (kindWrap != CUSTOM_WRAP_KIND.NONE) {
            if (side == OrderSide.Sell) {
                return
                    sellCustomWrap(
                        poolAddress,
                        sellToken,
                        buyToken,
                        specifiedAmount,
                        kindWrap,
                        sellTokenOutput,
                        buyTokenOutput
                    );
            } else {
                return
                    buyCustomWrap(
                        poolAddress,
                        sellToken,
                        buyToken,
                        specifiedAmount,
                        kindWrap,
                        sellTokenOutput,
                        buyTokenOutput
                    );
            }
        } else {
            if (
                (isERC4626(sellToken) && !isERC4626(buyToken)) ||
                (!isERC4626(sellToken) && isERC4626(buyToken))
            ) {
                (
                    ERC4626_SWAP_TYPE kind,
                    address outputAddress
                ) = getERC4626PathType(poolAddress, sellToken, buyToken);

                if (isERC4626(sellToken) && !isERC4626(buyToken)) {
                    // perform swap: ERC4626(share)->ERC20(token)
                    if (side == OrderSide.Sell) {
                        return
                            sellERC4626ForERC20(
                                poolAddress,
                                sellToken,
                                buyToken,
                                specifiedAmount,
                                kind,
                                outputAddress
                            );
                    } else {
                        // return
                        // buyERC20WithERC4626(
                        //     poolAddress,
                        //     sellToken,
                        //     buyToken,
                        //     specifiedAmount
                        // );
                    }
                } else if (!isERC4626(sellToken) && isERC4626(buyToken)) {
                    // perform swap: ERC20(token)->ERC4626(share)
                    if (side == OrderSide.Sell) {
                        return
                            sellERC20ForERC4626(
                                poolAddress,
                                sellToken,
                                buyToken,
                                specifiedAmount,
                                kind,
                                outputAddress
                            );
                    } else {
                        // return buyERC4626WithERC20(
                        //     poolAddress, sellToken, buyToken, specifiedAmount
                        // );
                    }
                }
            }
            // swap ERC20<->ERC20, fallback to next code block
        }

        // Fallback (used for ERC20<->ERC20 and ERC4626<->ERC4626 as inherits
        // IERC20 logic)
        if (side == OrderSide.Buy) {
            return
                buyERC20WithERC20(
                    poolAddress,
                    IERC20(sellToken),
                    IERC20(buyToken),
                    specifiedAmount,
                    true
                );
        } else {
            return
                sellERC20ForERC20(
                    poolAddress,
                    IERC20(sellToken),
                    IERC20(buyToken),
                    specifiedAmount,
                    true
                );
        }
    }

    function getLimitsMiddleware(
        bytes32 poolId,
        address sellToken,
        address buyToken
    ) internal view returns (uint256[] memory limits) {
        limits = new uint256[](2);

        // custom wrap
        if (
            isERC4626(sellToken) &&
            isERC4626(buyToken) &&
            CustomBytesAppend.hasPrefix(abi.encodePacked(poolId))
        ) {
            return getLimitsCustomWrap(poolId, sellToken, buyToken);
        }

        // ERC4626<->ERC20
        if (isERC4626(sellToken) && !isERC4626(buyToken)) {
            return getLimitsERC4626ToERC20(poolId, sellToken, buyToken);
        }

        // ERC20->ERC4626
        if (!isERC4626(sellToken) && isERC4626(buyToken)) {
            return getLimitsERC20ToERC4626(poolId, sellToken, buyToken);
        }

        // fallback: ERC20<->ERC20, ERC4626<->ERC4626
        return getLimitsERC20(poolId, sellToken, buyToken);
    }
}
