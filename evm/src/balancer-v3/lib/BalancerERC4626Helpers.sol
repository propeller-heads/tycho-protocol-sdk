//SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.26;

import "./BalancerCustomWrapHelpers.sol";

abstract contract BalancerERC4626Helpers is BalancerCustomWrapHelpers {
    using SafeERC20 for IERC20;

    enum ERC4626_SWAP_TYPE {
        ERC20_SWAP, // ERC20->ERC20->ERC4626
        ERC20_WRAP, // ERC20->ERC4626->ERC4626
        ERC4626_UNWRAP, // ERC4626->ERC20->ERC20
        ERC4626_SWAP // ERC4626->ERC4626->ERC20
    }

    function getERC4626PathType(
        address pool,
        address sellToken,
        address buyToken
    ) internal view returns (ERC4626_SWAP_TYPE kind, address outputAddress) {
        IERC20[] memory tokens = vault.getPoolTokens(pool);

        if (!isERC4626(sellToken) && isERC4626(buyToken)) {
            for (uint256 i = 0; i < tokens.length; i++) {
                address token = address(tokens[i]);

                if (isERC4626(token)) {
                    if (IERC4626(token).asset() == sellToken) {
                        // Path is e.g. dai-sDAI-sBTC, wrap DAI to sDAI, swap
                        // sDAI to sBTC
                        outputAddress = token; // share of sellToken
                        kind = ERC4626_SWAP_TYPE.ERC20_WRAP;
                        break;
                    }
                } else {
                    if (token == IERC4626(buyToken).asset()) {
                        // Path is e.g. dai-BTC-sBTC, swap dai to BTC, wrap BTC
                        // to sBTC
                        outputAddress = token; // asset of buyToken
                        kind = ERC4626_SWAP_TYPE.ERC20_SWAP;
                        break;
                    }
                }
            }
        } else {
            for (uint256 i = 0; i < tokens.length; i++) {
                address token = address(tokens[i]);

                if (isERC4626(token)) {
                    if (IERC4626(token).asset() == buyToken) {
                        // Path is e.g. sDAI-DAI-BTC, unwrap sDAI to DAI, swap
                        // sDAI to BTC
                        outputAddress = token; // unused
                        kind = ERC4626_SWAP_TYPE.ERC4626_UNWRAP;
                        break;
                    }
                } else {
                    if (token == buyToken) {
                        // Path is e.g. sDAI-sBTC-BTC, swap sDAI to sBTC, unwrap
                        // sBTC to BTC
                        outputAddress = token; // buyToken.share()
                        kind = ERC4626_SWAP_TYPE.ERC20_SWAP;
                        break;
                    }
                }
            }
        }
    }

    function prepareERC4626ForERC20SellOrBuy(
        address pool,
        address _sellToken,
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
        IERC4626 sellTokenShare = IERC4626(_sellToken);
        IERC20 underlyingSellToken = IERC20(sellTokenShare.asset());

        sellPath = new IBatchRouter.SwapPathExactAmountIn[](1);
        buyPath = new IBatchRouter.SwapPathExactAmountOut[](1);
        IBatchRouter.SwapPathStep[]
            memory steps = new IBatchRouter.SwapPathStep[](2);

        if (kind == ERC4626_SWAP_TYPE.ERC4626_UNWRAP) {
            // unwrap sellToken.shares() to sellToken.asset()
            (
                ,
                ,
                IBatchRouter.SwapPathStep memory step0
            ) = createWrapOrUnwrapPath(
                    _sellToken,
                    specifiedAmount,
                    IVault.WrappingDirection.UNWRAP,
                    false
                );
            steps[0] = step0;

            // swap sellToken.asset() to buyToken
            (, , IBatchRouter.SwapPathStep memory step1) = createERC20Path(
                pool,
                underlyingSellToken,
                IERC20(buyToken),
                specifiedAmount,
                false
            );
            steps[1] = step1;
        } else {
            // swap sellToken to buyToken.share()
            (, , IBatchRouter.SwapPathStep memory step0) = createERC20Path(
                pool,
                IERC20(_sellToken),
                IERC20(buyToken),
                specifiedAmount,
                false
            );
            steps[0] = step0;

            // unwrap buyToken.share() to buyToken
            (
                ,
                ,
                IBatchRouter.SwapPathStep memory step1
            ) = createWrapOrUnwrapPath(
                    outputAddress,
                    specifiedAmount,
                    IVault.WrappingDirection.UNWRAP,
                    false
                );
            steps[1] = step1;
        }

        if (!isBuy) {
            sellPath[0] = IBatchRouter.SwapPathExactAmountIn({
                tokenIn: IERC20(_sellToken),
                steps: steps,
                exactAmountIn: specifiedAmount,
                minAmountOut: type(uint256).max
            });
        } else {
            buyPath[0] = IBatchRouter.SwapPathExactAmountOut({
                tokenIn: IERC20(_sellToken),
                steps: steps,
                exactAmountOut: specifiedAmount,
                maxAmountIn: type(uint256).max
            });
        }
    }

    function prepareERC20ForERC4626SellOrBuy(
        address pool,
        address _sellToken,
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
        sellPath = new IBatchRouter.SwapPathExactAmountIn[](1);
        buyPath = new IBatchRouter.SwapPathExactAmountOut[](1);
        IBatchRouter.SwapPathStep[]
            memory steps = new IBatchRouter.SwapPathStep[](2);

        if (kind == ERC4626_SWAP_TYPE.ERC20_SWAP) {
            // swap sellToken to buyToken.asset()
            (, , IBatchRouter.SwapPathStep memory step0) = createERC20Path(
                pool,
                IERC20(_sellToken),
                IERC20(outputAddress),
                specifiedAmount,
                false
            );
            steps[0] = step0;

            // wrap buyToken.asset() to buyToken.shares()
            (
                ,
                ,
                IBatchRouter.SwapPathStep memory step1
            ) = createWrapOrUnwrapPath(
                    buyToken,
                    specifiedAmount,
                    IVault.WrappingDirection.WRAP,
                    false
                );
            steps[1] = step1;
        } else {
            // wrap sellToken to sellToken.shares()
            (
                ,
                ,
                IBatchRouter.SwapPathStep memory step0
            ) = createWrapOrUnwrapPath(
                    outputAddress,
                    specifiedAmount,
                    IVault.WrappingDirection.WRAP,
                    false
                );
            steps[0] = step0;

            // swap sellToken.shares() to buyToken
            (, , IBatchRouter.SwapPathStep memory step1) = createERC20Path(
                pool,
                IERC20(outputAddress),
                IERC20(buyToken),
                specifiedAmount,
                false
            );
            steps[1] = step1;
        }

        if (!isBuy) {
            buyPath[0] = IBatchRouter.SwapPathExactAmountOut({
                tokenIn: IERC20(_sellToken),
                steps: steps,
                maxAmountIn: type(uint256).max,
                exactAmountOut: specifiedAmount
            });
        } else {
            sellPath[0] = IBatchRouter.SwapPathExactAmountIn({
                tokenIn: IERC20(_sellToken),
                steps: steps,
                exactAmountIn: specifiedAmount,
                minAmountOut: type(uint256).max
            });
        }
    }

    /**
     * @dev Sell an ERC4626 token for an ERC20 token
     * @param pool the ERC4626.asset()->ERC20 pool
     * @param _sellToken ERC4626 token being sold(by unwrapping to
     * sellToken.asset())
     * @param buyToken ERC20 token being bought
     * @param specifiedAmount The amount of sellToken(ERC4626) tokens to sell
     */
    function sellERC4626ForERC20(
        address pool,
        address _sellToken,
        address buyToken,
        uint256 specifiedAmount,
        ERC4626_SWAP_TYPE kind,
        address outputAddress
    ) internal returns (uint256 calculatedAmount) {
        IERC4626 sellTokenShare = IERC4626(_sellToken);
        bytes memory userData;

        // transfer sellToken.share() to address(this)
        IERC20(sellTokenShare).safeTransferFrom(
            msg.sender,
            address(this),
            specifiedAmount
        );
        IERC20(sellTokenShare).safeIncreaseAllowance(permit2, specifiedAmount);
        IPermit2(permit2).approve(
            address(sellTokenShare),
            address(router),
            type(uint160).max,
            type(uint48).max
        );

        (
            IBatchRouter.SwapPathExactAmountIn[] memory paths,

        ) = prepareERC4626ForERC20SellOrBuy(
                pool,
                _sellToken,
                buyToken,
                specifiedAmount,
                kind,
                outputAddress,
                false
            );
        (, , uint256[] memory amountsOut) = router.swapExactIn(
            paths,
            type(uint256).max,
            false,
            userData
        );

        calculatedAmount = amountsOut[0];

        IERC20(buyToken).safeTransfer(msg.sender, calculatedAmount);
    }

    /**
     * @dev Perform a sell order for ERC4626 tokens
     * @param pool the ERC4626.asset()->ERC20 pool
     * @param _sellToken ERC4626 token being sold(by unwrapping to
     * sellToken.asset())
     * @param buyToken ERC20 token being bought
     * @param specifiedAmount The amount of sellToken(ERC4626) tokens to sell
     */
    function getAmountOutERC4626ForERC20(
        address pool,
        address _sellToken,
        address buyToken,
        uint256 specifiedAmount,
        ERC4626_SWAP_TYPE kind,
        address outputAddress
    ) internal returns (uint256 calculatedAmount) {
        (
            IBatchRouter.SwapPathExactAmountIn[] memory paths,

        ) = prepareERC4626ForERC20SellOrBuy(
                pool,
                _sellToken,
                buyToken,
                specifiedAmount,
                kind,
                outputAddress,
                false
            );
        (, , uint256[] memory amountsOut) = router.querySwapExactIn(
            paths,
            address(0),
            bytes("")
        );

        calculatedAmount = amountsOut[0];
    }

    /**
     * @dev Buy an ERC20 token with an ERC4626 token
     * @param pool the ERC4626.asset()->ERC20 pool
     * @param _sellToken ERC4626 token being sold, of which .asset() is the
     * sellToken
     * @param buyToken ERC20 token being bought
     * @param specifiedAmount The amount of buyToken to buy
     */
    function buyERC20WithERC4626(
        address pool,
        address _sellToken,
        address buyToken,
        uint256 specifiedAmount,
        ERC4626_SWAP_TYPE kind,
        address outputAddress
    ) internal returns (uint256 calculatedAmount) {
        bytes memory userData;

        // get balance of sender
        uint256 initialSenderBalance = IERC20(_sellToken).balanceOf(msg.sender);

        // transfer sellToken.share() to address(this)
        IERC20(_sellToken).safeTransferFrom(
            msg.sender,
            address(this),
            initialSenderBalance
        );

        IERC20(_sellToken).safeIncreaseAllowance(permit2, type(uint256).max);
        IPermit2(permit2).approve(
            address(_sellToken),
            address(router),
            type(uint160).max,
            type(uint48).max
        );

        (
            ,
            IBatchRouter.SwapPathExactAmountOut[] memory paths
        ) = prepareERC4626ForERC20SellOrBuy(
                pool,
                _sellToken,
                buyToken,
                specifiedAmount,
                kind,
                outputAddress,
                true
            );
        (, , uint256[] memory amountsIn) = router.swapExactOut(
            paths,
            type(uint256).max,
            false,
            userData
        );

        calculatedAmount = amountsIn[0];

        IERC20(buyToken).safeTransfer(msg.sender, specifiedAmount);

        // transfer back sellToken to sender
        IERC20(_sellToken).safeTransferFrom(
            address(this),
            msg.sender,
            initialSenderBalance - calculatedAmount
        );
    }

    /**
     * @param pool the ERC20->ERC4626.asset() pool
     */
    function sellERC20ForERC4626(
        address pool,
        address sellToken,
        address buyToken,
        uint256 specifiedAmount,
        ERC4626_SWAP_TYPE kind,
        address outputAddress
    ) internal returns (uint256 calculatedAmount) {
        bytes memory userData;

        // transfer and approve
        IERC20(sellToken).safeTransferFrom(
            msg.sender,
            address(this),
            specifiedAmount
        );
        IERC20(sellToken).safeIncreaseAllowance(permit2, specifiedAmount);
        IPermit2(permit2).approve(
            address(sellToken),
            address(router),
            type(uint160).max,
            type(uint48).max
        );

        (
            IBatchRouter.SwapPathExactAmountIn[] memory paths,

        ) = prepareERC20ForERC4626SellOrBuy(
                pool,
                sellToken,
                buyToken,
                specifiedAmount,
                kind,
                outputAddress,
                false
            );
        (, , uint256[] memory amountsOut) = router.swapExactIn(
            paths,
            type(uint256).max,
            false,
            userData
        );

        calculatedAmount = amountsOut[0];

        IERC20(buyToken).safeTransfer(msg.sender, calculatedAmount);
    }

    /**
     * @param pool the ERC20->ERC4626.asset() pool
     */
    function getAmountOutERC20ForERC4626(
        address pool,
        address _sellToken,
        address buyToken,
        uint256 specifiedAmount,
        ERC4626_SWAP_TYPE kind,
        address outputAddress
    ) internal returns (uint256 calculatedAmount) {
        (
            IBatchRouter.SwapPathExactAmountIn[] memory paths,

        ) = prepareERC20ForERC4626SellOrBuy(
                pool,
                _sellToken,
                buyToken,
                specifiedAmount,
                kind,
                outputAddress,
                false
            );
        (, , uint256[] memory amountsOut) = router.querySwapExactIn(
            paths,
            address(0),
            bytes("")
        );

        calculatedAmount = amountsOut[0];
    }

    /**
     * @param pool the ERC20->ERC4626.asset() pool
     */
    function buyERC4626WithERC20(
        address pool,
        address sellToken,
        address buyToken,
        uint256 specifiedAmount,
        ERC4626_SWAP_TYPE kind,
        address outputAddress
    ) internal returns (uint256 calculatedAmount) {
        bytes memory userData;

        // get balance of sender
        uint256 initialSenderBalance = IERC20(sellToken).balanceOf(msg.sender);

        // transfer and approve
        IERC20(sellToken).safeTransferFrom(
            msg.sender,
            address(this),
            initialSenderBalance
        );
        IERC20(sellToken).safeIncreaseAllowance(permit2, type(uint256).max);
        IPermit2(permit2).approve(
            address(sellToken),
            address(router),
            type(uint160).max,
            type(uint48).max
        );

        (
            ,
            IBatchRouter.SwapPathExactAmountOut[] memory paths
        ) = prepareERC20ForERC4626SellOrBuy(
                pool,
                sellToken,
                buyToken,
                specifiedAmount,
                kind,
                outputAddress,
                true
            );
        (, , uint256[] memory amountsIn) = router.swapExactOut(
            paths,
            type(uint256).max,
            false,
            userData
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

    function getLimitsERC4626ToERC20(
        bytes32 poolId,
        address sellToken,
        address buyToken
    ) internal view returns (uint256[] memory limits) {
        limits = new uint256[](2);
        address pool = address(bytes20(poolId));

        (IERC20[] memory tokens, , uint256[] memory balancesRaw, ) = vault
            .getPoolTokenInfo(pool);

        IERC20 underlyingSellToken = IERC20(IERC4626(sellToken).asset());
        IERC20 buyTokenERC = IERC20(buyToken);

        for (uint256 i = 0; i < tokens.length; i++) {
            if (tokens[i] == underlyingSellToken) {
                /**
                 * @dev Using IERC4626(sellToken).totalSupply() as getAmountIn
                 * is
                 * not possible since this limit will also
                 * be used for non-static calls
                 */
                limits[0] =
                    (IERC4626(sellToken).totalSupply() * RESERVE_LIMIT_FACTOR) /
                    10;
            }
            if (tokens[i] == buyTokenERC) {
                limits[1] = (balancesRaw[i] * RESERVE_LIMIT_FACTOR) / 10;
            }
        }
    }

    function getLimitsERC20ToERC4626(
        bytes32 poolId,
        address sellToken,
        address buyToken
    ) internal view returns (uint256[] memory limits) {
        address pool = address(bytes20(poolId));
        (IERC20[] memory tokens, , uint256[] memory balancesRaw, ) = vault
            .getPoolTokenInfo(pool);

        IERC20 underlyingBuyToken = IERC20(IERC4626(sellToken).asset());
        IERC20 sellTokenERC = IERC20(sellToken);

        for (uint256 i = 0; i < tokens.length; i++) {
            if (tokens[i] == underlyingBuyToken) {
                /**
                 * @dev Using IERC4626(buyToken).totalSupply() as getAmountIn is
                 * not possible since this limit will also
                 * be used for non-static calls
                 */
                limits[1] =
                    (IERC4626(buyToken).totalSupply() * RESERVE_LIMIT_FACTOR) /
                    10;
            }
            if (tokens[i] == sellTokenERC) {
                limits[0] = (balancesRaw[i] * RESERVE_LIMIT_FACTOR) / 10;
            }
        }
    }
}
