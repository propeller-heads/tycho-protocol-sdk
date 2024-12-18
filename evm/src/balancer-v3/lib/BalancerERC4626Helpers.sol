//SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.26;

import "./BalancerCustomWrapHelpers.sol";

abstract contract BalancerERC4626Helpers is BalancerCustomWrapHelpers {
    using SafeERC20 for IERC20;

    function getERC4626PathType(
        address pool,
        address sellToken,
        address buyToken,
        bool sellTokenFound,
        bool buyTokenFound // cannot be true if sellToken is
    ) internal view returns (ERC4626_SWAP_TYPE kind, address outputAddress) {
        IERC20[] memory tokens = vault.getPoolTokens(pool);

        if (!isERC4626(sellToken) && isERC4626(buyToken)) {
            if (sellTokenFound) {
                kind = ERC4626_SWAP_TYPE.SWAP_WRAP;
            } else if (buyTokenFound) {
                for (uint256 i = 0; i < tokens.length; i++) {
                    address token = address(tokens[i]);
                    if (
                        isERC4626(token) && IERC4626(token).asset() == sellToken
                    ) {
                        outputAddress = token; // sellToken share
                        break;
                    }
                }
                require(outputAddress != address(0), "Token not found in pool");
                kind = ERC4626_SWAP_TYPE.WRAP_SWAP;
            }
        } else if (isERC4626(sellToken) && !isERC4626(buyToken)) {
            if (sellTokenFound) {
                for (uint256 i = 0; i < tokens.length; i++) {
                    address token = address(tokens[i]);
                    if (isERC4626(token) && IERC4626(token).asset() == buyToken)
                    {
                        outputAddress = token; // buyToken share
                        break;
                    }
                }
                require(outputAddress != address(0), "Token not found in pool");
                kind = ERC4626_SWAP_TYPE.SWAP_UNWRAP;
            } else if (buyTokenFound) {
                // output address is unused the UNWRAP action just takes
                // sellToken which is indeed a share
                kind = ERC4626_SWAP_TYPE.UNWRAP_SWAP;
            }
        } else if (!isERC4626(buyToken) && !isERC4626(sellToken)) {
            if (sellTokenFound) {
                for (uint256 i = 0; i < tokens.length; i++) {
                    address token = address(tokens[i]);
                    if (isERC4626(token) && IERC4626(token).asset() == buyToken)
                    {
                        outputAddress = token; // buyToken share
                        break;
                    }
                }
                require(outputAddress != address(0), "Token not found in pool");
                kind = ERC4626_SWAP_TYPE.SWAP_UNWRAP;
            } else if (buyTokenFound) {
                for (uint256 i = 0; i < tokens.length; i++) {
                    address token = address(tokens[i]);
                    if (
                        isERC4626(token) && IERC4626(token).asset() == sellToken
                    ) {
                        outputAddress = token; // sellToken share
                        break;
                    }
                }
                require(outputAddress != address(0), "Token not found in pool");
                kind = ERC4626_SWAP_TYPE.WRAP_SWAP;
            }
        } else if (isERC4626(buyToken) && isERC4626(sellToken)) {
            if (sellTokenFound) {
                outputAddress = IERC4626(buyToken).asset(); // buy token asset
                kind = ERC4626_SWAP_TYPE.SWAP_WRAP;
            } else if (buyTokenFound) {
                kind = ERC4626_SWAP_TYPE.UNWRAP_SWAP;
            }
        }
    }

    function prepareERC4626SellOrBuy(
        address pool,
        address sellToken,
        address buyToken,
        uint256 specifiedAmount,
        ERC4626_SWAP_TYPE kind,
        address outputAddress,
        bool isBuy
    )
        internal
        view
        returns (
            IBatchRouter.SwapPathExactAmountIn[] memory sellPath,
            IBatchRouter.SwapPathExactAmountOut[] memory buyPath
        )
    {
        IBatchRouter.SwapPathStep[] memory steps;

        if (kind == ERC4626_SWAP_TYPE.SWAP_WRAP) {
            // !isERC4626(sellToken) && isERC4626(buyToken) and
            // isERC4626(buyToken) && isERC4626(sellToken)
            steps = new IBatchRouter.SwapPathStep[](2);

            // swap: sellToken -> buyToken.asset()
            (,, steps[0]) = createERC20Path(
                pool,
                IERC20(sellToken),
                IERC20(IERC4626(buyToken).asset()),
                specifiedAmount,
                false,
                false
            );

            // wrap: buyToken.asset() -> buyToken.shares()
            (,, steps[1]) = createWrapOrUnwrapPath(
                buyToken, specifiedAmount, IVault.WrappingDirection.WRAP, false
            );
        } else if (kind == ERC4626_SWAP_TYPE.SWAP_UNWRAP) {
            // isERC4626(sellToken) && !isERC4626(buyToken) and
            // !isERC4626(buyToken) && !isERC4626(sellToken)
            steps = new IBatchRouter.SwapPathStep[](2);

            // swap: sellToken -> buyToken.shares()
            (,, steps[0]) = createERC20Path(
                pool,
                IERC20(sellToken),
                IERC20(outputAddress),
                specifiedAmount,
                false,
                false
            );

            // unwrap: buyToken.shares() -> buyToken.asset()
            (,, steps[1]) = createWrapOrUnwrapPath(
                outputAddress,
                specifiedAmount,
                IVault.WrappingDirection.UNWRAP,
                false
            );
        } else if (kind == ERC4626_SWAP_TYPE.WRAP_SWAP) {
            // input is ERC20, output is ERC4626
            steps = new IBatchRouter.SwapPathStep[](2);

            // wrap: sellToken.shares() -> sellToken.asset()
            (,, steps[0]) = createWrapOrUnwrapPath(
                outputAddress,
                specifiedAmount,
                IVault.WrappingDirection.WRAP,
                false
            );
            // swap: sellToken.asset() -> buyToken
            (,, steps[1]) = createERC20Path(
                pool,
                IERC20(outputAddress),
                IERC20(buyToken),
                specifiedAmount,
                false,
                false
            );
        } else if (kind == ERC4626_SWAP_TYPE.UNWRAP_SWAP) {
            steps = new IBatchRouter.SwapPathStep[](2);

            // unwrap: sellToken.shares() -> sellToken.asset()
            (,, steps[0]) = createWrapOrUnwrapPath(
                sellToken,
                specifiedAmount,
                IVault.WrappingDirection.UNWRAP,
                false
            );

            // swap: sellToken.asset() -> buyToken
            (,, steps[1]) = createERC20Path(
                pool,
                IERC20(sellToken),
                IERC20(buyToken),
                specifiedAmount,
                false,
                false
            );
        }

        if (isBuy) {
            buyPath = new IBatchRouter.SwapPathExactAmountOut[](1);
            buyPath[0] = IBatchRouter.SwapPathExactAmountOut({
                tokenIn: IERC20(sellToken),
                steps: steps,
                maxAmountIn: IERC20(sellToken).balanceOf(address(this)),
                exactAmountOut: specifiedAmount
            });
        } else {
            sellPath = new IBatchRouter.SwapPathExactAmountIn[](1);
            sellPath[0] = IBatchRouter.SwapPathExactAmountIn({
                tokenIn: IERC20(sellToken),
                steps: steps,
                exactAmountIn: specifiedAmount,
                minAmountOut: 1
            });
        }
    }

    function swapERC4626AndERC20(
        address pool,
        address sellToken,
        address buyToken,
        uint256 specifiedAmount,
        ERC4626_SWAP_TYPE kind,
        address outputAddress,
        bool isBuy
    ) internal returns (uint256 calculatedAmount) {
        // approve
        uint256 approvalAmount = specifiedAmount;
        if (isBuy) {
            approvalAmount = IERC20(sellToken).balanceOf(msg.sender);
        }
        IERC20(sellToken).safeIncreaseAllowance(permit2, approvalAmount);
        IPermit2(permit2).approve(
            address(sellToken),
            address(router),
            type(uint160).max,
            type(uint48).max
        );

        if (!isBuy) {
            IERC20(sellToken).safeTransferFrom(
                msg.sender, address(this), approvalAmount
            );

            (IBatchRouter.SwapPathExactAmountIn[] memory sellPath,) =
            prepareERC4626SellOrBuy(
                pool,
                sellToken,
                buyToken,
                specifiedAmount,
                kind,
                outputAddress,
                isBuy
            );

            (,, uint256[] memory amountsOut) = router.swapExactIn(
                sellPath, type(uint256).max, false, bytes("")
            );

            calculatedAmount = amountsOut[0];

            IERC20(buyToken).safeTransfer(msg.sender, calculatedAmount);
        } else {
            uint256 initialSenderBalance =
                IERC20(sellToken).balanceOf(msg.sender);
            IERC20(sellToken).safeTransferFrom(
                msg.sender, address(this), approvalAmount
            );

            (, IBatchRouter.SwapPathExactAmountOut[] memory buyPath) =
            prepareERC4626SellOrBuy(
                pool,
                sellToken,
                buyToken,
                specifiedAmount,
                kind,
                outputAddress,
                true
            );

            (,, uint256[] memory amountsIn) = router.swapExactOut(
                buyPath, type(uint256).max, false, bytes("")
            );

            calculatedAmount = amountsIn[0];

            IERC20(buyToken).safeTransfer(msg.sender, specifiedAmount);

            // transfer back sellToken to sender
            IERC20(sellToken).safeTransferFrom(
                address(this),
                msg.sender,
                initialSenderBalance - calculatedAmount
            );
        }
    }

    function getAmountOutERC4626AndERC20(
        address pool,
        address sellToken,
        address buyToken,
        uint256 specifiedAmount,
        ERC4626_SWAP_TYPE kind,
        address outputAddress
    ) internal returns (uint256 calculatedAmount) {
        (IBatchRouter.SwapPathExactAmountIn[] memory paths,) =
        prepareERC4626SellOrBuy(
            pool,
            sellToken,
            buyToken,
            specifiedAmount,
            kind,
            outputAddress,
            false
        );
        (,, uint256[] memory amountsOut) =
            router.querySwapExactIn(paths, address(0), bytes(""));
        calculatedAmount = amountsOut[0];
    }

    function getLimitsERC4626AndERC20(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        ERC4626_SWAP_TYPE kind,
        address outputAddress
    ) internal view returns (uint256[] memory limits) {
        limits = new uint256[](2);
        address pool = address(bytes20(poolId));
        (IERC20[] memory tokens,, uint256[] memory balancesRaw,) =
            vault.getPoolTokenInfo(pool);

        uint256 tokenLimit;

        if (kind == ERC4626_SWAP_TYPE.SWAP_WRAP) {
            for (uint256 i = 0; i < tokens.length; i++) {
                address token = address(tokens[i]);
                if (token == sellToken) {
                    limits[0] = balancesRaw[i] * RESERVE_LIMIT_FACTOR / 10;
                }

                if (token == IERC4626(buyToken).asset()) {
                    tokenLimit = balancesRaw[i] * RESERVE_LIMIT_FACTOR / 10;
                }
            }
            limits[1] = IERC4626(buyToken).previewDeposit(tokenLimit);
        } else if (kind == ERC4626_SWAP_TYPE.SWAP_UNWRAP) {
            for (uint256 i = 0; i < tokens.length; i++) {
                address token = address(tokens[i]);
                if (token == sellToken) {
                    limits[0] = balancesRaw[i] * RESERVE_LIMIT_FACTOR / 10;
                } else if (token == outputAddress) {
                    tokenLimit = balancesRaw[i] * RESERVE_LIMIT_FACTOR / 10;
                }
            }
            limits[1] = IERC4626(outputAddress).previewRedeem(tokenLimit);
        } else if (kind == ERC4626_SWAP_TYPE.WRAP_SWAP) {
            for (uint256 i = 0; i < tokens.length; i++) {
                address token = address(tokens[i]);

                if (token == outputAddress) {
                    limits[0] = IERC4626(outputAddress).previewRedeem(
                        balancesRaw[i] * RESERVE_LIMIT_FACTOR / 10
                    );
                }

                if (token == buyToken) {
                    limits[1] = balancesRaw[i] * RESERVE_LIMIT_FACTOR / 10;
                }
            }
        } else if (kind == ERC4626_SWAP_TYPE.UNWRAP_SWAP) {
            for (uint256 i = 0; i < tokens.length; i++) {
                address token = address(tokens[i]);

                if (token == buyToken) {
                    limits[0] = balancesRaw[i] * RESERVE_LIMIT_FACTOR / 10;
                }

                if (token == IERC4626(sellToken).asset()) {
                    limits[1] = IERC4626(sellToken).previewDeposit(
                        balancesRaw[i] * RESERVE_LIMIT_FACTOR / 10
                    );
                }
            }
        }
    }
}
