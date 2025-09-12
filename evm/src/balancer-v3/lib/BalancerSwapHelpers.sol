//SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.26;

import "./BalancerERC4626Helpers.sol";

/**
 * @title Balancer V3 Swap Helpers
 * @dev A wrapped library containing swap functions, helpers and storage for the
 * Balancer V3 Swap Adapter contract
 */
abstract contract BalancerSwapHelpers is
    BalancerERC4626Helpers,
    ISwapAdapter
{
    function getAmountOutMiddleware(
        bytes32 pool,
        address sellToken,
        address buyToken,
        uint256 specifiedAmount
    ) internal returns (uint256 amountOut) {
        address poolAddress = address(bytes20(pool));

        // liquidityBuffer
        if (isLiquidityBuffer(poolAddress)) {
            address wrappedToken = poolAddress;

            if (sellToken == wrappedToken) {
                // ERC4626 -> ERC20
                return getAmountOutBuffer(
                    wrappedToken,
                    specifiedAmount,
                    IVault.WrappingDirection.UNWRAP
                );
            } else if (buyToken == wrappedToken) {
                // ERC20 -> ERC4626
                return getAmountOutBuffer(
                    wrappedToken, specifiedAmount, IVault.WrappingDirection.WRAP
                );
            }
        }

        // getTokens() -> [token0, token1] -> if([sellToken,buyToken) in
        // [token0, token1]) -> direct
        IERC20[] memory tokens = vault.getPoolTokens(poolAddress);

        bool sellTokenFound;
        bool buyTokenFound;
        if (sellToken == address(0) || buyToken == address(0)) {
            sellTokenFound = true;
            buyTokenFound = true;
        } else {
            for (uint256 i = 0; i < tokens.length; i++) {
                address token = address(tokens[i]);
                if (token == sellToken) {
                    sellTokenFound = true;
                } else if (token == buyToken) {
                    buyTokenFound = true;
                }
            }
        }

        if (sellTokenFound && buyTokenFound) {
            // Direct Swap
            (IBatchRouter.SwapPathExactAmountIn memory sellPath,,) =
            createERC20Path(
                poolAddress,
                IERC20(sellToken),
                IERC20(buyToken),
                specifiedAmount,
                false,
                sellToken == address(0) || buyToken == address(0)
            );
            return getAmountOut(sellPath);
        } else if (!sellTokenFound && !buyTokenFound) {
            // 3 step (4 tokens)
            (
                CUSTOM_WRAP_KIND kindWrap,
                address sellTokenOutput,
                address buyTokenOutput
            ) = getCustomWrap(sellToken, buyToken, poolAddress);
            return getAmountOutCustomWrap(
                poolAddress,
                sellToken,
                buyToken,
                specifiedAmount,
                kindWrap,
                sellTokenOutput,
                buyTokenOutput
            );
        } else {
            // 2 step (3 tokens)
            (ERC4626_SWAP_TYPE kind, address outputAddress) = getERC4626PathType(
                poolAddress, sellToken, buyToken, sellTokenFound
            );

            if (kind != ERC4626_SWAP_TYPE.NONE) {
                return getAmountOutERC4626AndERC20(
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
        // liquidityBuffer
        if (isLiquidityBuffer(address(bytes20(pool)))) {
            address wrappedToken = poolAddress;
            if (sellToken == wrappedToken) {
                // ERC4626 -> ERC20
                IVault.SwapKind swapKind = side == OrderSide.Sell
                    ? IVault.SwapKind.EXACT_IN
                    : IVault.SwapKind.EXACT_OUT;
                return abi.decode(
                    vault.unlock(
                        abi.encodeCall(
                            this._swapBufferCallback,
                            (
                                wrappedToken,
                                buyToken,
                                msg.sender,
                                specifiedAmount,
                                swapKind,
                                IVault.WrappingDirection.UNWRAP
                            )
                        )
                    ),
                    (uint256)
                );
            } else if (buyToken == wrappedToken) {
                // ERC20 -> ERC4626
                IVault.SwapKind swapKind = side == OrderSide.Sell
                    ? IVault.SwapKind.EXACT_IN
                    : IVault.SwapKind.EXACT_OUT;
                return abi.decode(
                    vault.unlock(
                        abi.encodeCall(
                            this._swapBufferCallback,
                            (
                                wrappedToken,
                                sellToken,
                                msg.sender,
                                specifiedAmount,
                                swapKind,
                                IVault.WrappingDirection.WRAP
                            )
                        )
                    ),
                    (uint256)
                );
            }
        }

        // getTokens() -> [token0, token1] -> if([sellToken,buyToken) in
        // [token0, token1]) -> direct
        IERC20[] memory tokens = vault.getPoolTokens(poolAddress);

        bool sellTokenFound;
        bool buyTokenFound;
        if (sellToken == address(0) || buyToken == address(0)) {
            sellTokenFound = true;
            buyTokenFound = true;
        } else {
            for (uint256 i = 0; i < tokens.length; i++) {
                address token = address(tokens[i]);
                if (token == sellToken) {
                    sellTokenFound = true;
                } else if (token == buyToken) {
                    buyTokenFound = true;
                }
            }
        }

        if (sellTokenFound && buyTokenFound) {
            // Direct Swap
            // Fallback (used for ERC20<->ERC20 and ERC4626<->ERC4626 as
            // inherits
            // IERC20 logic)
            if (side == OrderSide.Buy) {
                return buyERC20WithERC20(
                    poolAddress,
                    IERC20(sellToken),
                    IERC20(buyToken),
                    specifiedAmount,
                    true
                );
            } else {
                return sellERC20ForERC20(
                    poolAddress,
                    IERC20(sellToken),
                    IERC20(buyToken),
                    specifiedAmount,
                    true
                );
            }
        } else if (!sellTokenFound && !buyTokenFound) {
            // 3 step (4 tokens)
            (
                CUSTOM_WRAP_KIND kindWrap,
                address sellTokenOutput,
                address buyTokenOutput
            ) = getCustomWrap(sellToken, buyToken, poolAddress);

            if (side == OrderSide.Sell) {
                return sellCustomWrap(
                    poolAddress,
                    sellToken,
                    buyToken,
                    specifiedAmount,
                    kindWrap,
                    sellTokenOutput,
                    buyTokenOutput
                );
            } else {
                return buyCustomWrap(
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
            // 2 step (3 tokens)
            (ERC4626_SWAP_TYPE kind, address outputAddress) = getERC4626PathType(
                poolAddress, sellToken, buyToken, sellTokenFound
            );

            return swapERC4626AndERC20(
                poolAddress,
                sellToken,
                buyToken,
                specifiedAmount,
                kind,
                outputAddress,
                side == OrderSide.Buy
            );
        }
    }

    function getLimitsMiddleware(
        bytes32 poolId,
        address sellToken,
        address buyToken
    ) internal view returns (uint256[] memory limits) {
        address poolAddress = address(bytes20(poolId));
        // liquidityBuffer
        if (isLiquidityBuffer(address(bytes20(poolId)))) {
            address wrappedToken = poolAddress;
            limits = new uint256[](2);
            (uint256 underlyingBalanceRaw, uint256 wrappedBalanceRaw) =
                vaultExplorer.getBufferBalance(IERC4626(wrappedToken));
            limits[0] = wrappedBalanceRaw; // wrapped token balance
            limits[1] = underlyingBalanceRaw; // underlying token balance
            return limits;
        }

        // getTokens() -> [token0, token1] -> if([sellToken,buyToken) in
        // [token0, token1]) -> direct
        IERC20[] memory tokens = vault.getPoolTokens(poolAddress);

        bool sellTokenFound;
        bool buyTokenFound;
        if (sellToken == address(0) || buyToken == address(0)) {
            sellTokenFound = true;
            buyTokenFound = true;
        } else {
            for (uint256 i = 0; i < tokens.length; i++) {
                address token = address(tokens[i]);
                if (token == sellToken) {
                    sellTokenFound = true;
                } else if (token == buyToken) {
                    buyTokenFound = true;
                }
            }
        }

        if (sellTokenFound && buyTokenFound) {
            // Direct Swap
            return getLimitsERC20(poolId, sellToken, buyToken);
        } else if (!sellTokenFound && !buyTokenFound) {
            // 3 step (4 tokens)
            (
                CUSTOM_WRAP_KIND kindWrap,
                address sellTokenOutput,
                address buyTokenOutput
            ) = getCustomWrap(sellToken, buyToken, poolAddress);

            return getLimitsCustomWrap(
                poolId,
                sellToken,
                buyToken,
                kindWrap,
                sellTokenOutput,
                buyTokenOutput
            );
        } else {
            // 2 step (3 tokens)
            (ERC4626_SWAP_TYPE kind, address outputAddress) = getERC4626PathType(
                poolAddress, sellToken, buyToken, sellTokenFound
            );
            return getLimitsERC4626AndERC20(
                poolId, sellToken, buyToken, kind, outputAddress
            );
        }
    }

    // Callback functions for vault unlock pattern
    function getAmountOutBuffer(
        address wrappedToken,
        uint256 specifiedAmount,
        IVault.WrappingDirection direction
    ) public view returns (uint256) {
        uint256 amountCalculatedRaw;
        if (direction == IVault.WrappingDirection.WRAP) {
            // ERC20 -> ERC4626: use previewDeposit
            amountCalculatedRaw =
                IERC4626(wrappedToken).previewDeposit(specifiedAmount);
        } else {
            // ERC4626 -> ERC20: use previewRedeem
            amountCalculatedRaw =
                IERC4626(wrappedToken).previewRedeem(specifiedAmount);
        }
        return amountCalculatedRaw;
    }

    function _swapBufferCallback(
        address wrappedToken,
        address underlyingToken,
        address sender,
        uint256 specifiedAmount,
        IVault.SwapKind swapKind,
        IVault.WrappingDirection direction
    ) external returns (uint256 amountCalculatedRaw) {
        require(msg.sender == address(vault), "Only vault can call");

        // Perform the buffer operation
        (amountCalculatedRaw,,) = vault.erc4626BufferWrapOrUnwrap(
            IVault.BufferWrapOrUnwrapParams({
                kind: swapKind,
                direction: direction,
                wrappedToken: IERC4626(wrappedToken),
                amountGivenRaw: specifiedAmount,
                limitRaw: swapKind == IVault.SwapKind.EXACT_IN
                    ? 0
                    : type(uint256).max
            })
        );

        // Determine input/output tokens and amounts
        bool isWrap = direction == IVault.WrappingDirection.WRAP;
        bool isExactIn = swapKind == IVault.SwapKind.EXACT_IN;

        address inputToken = isWrap ? underlyingToken : wrappedToken;
        address outputToken = isWrap ? wrappedToken : underlyingToken;
        uint256 inputAmount = isExactIn ? specifiedAmount : amountCalculatedRaw;
        uint256 outputAmount = isExactIn ? amountCalculatedRaw : specifiedAmount;

        // Transfer input tokens and settle
        IERC20(inputToken).transferFrom(sender, address(vault), inputAmount);
        vault.settle(IERC20(inputToken), inputAmount);
        vault.sendTo(IERC20(outputToken), sender, outputAmount);
    }
}
