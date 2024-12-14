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
        address sellToken, address buyToken, address pool
    ) internal view returns (ERC4626_SWAP_TYPE kind,
            address outputAddress
    ) {
        IERC20[] memory tokens = vault.getPoolTokens(pool);
        
        if(!isERC4626(sellToken) && isERC4626(buyToken)) {

            for (uint256 i = 0; i < tokens.length; i++) {
                address token = address(tokens[i]);

                if(isERC4626(token)) {
                    if(IERC4626(token).asset() == sellToken) {
                        // Path is e.g. dai-sDAI-sBTC, wrap DAI to sDAI, swap sDAI to sBTC
                        outputAddress = token; // share of sellToken
                        kind = ERC4626_SWAP_TYPE.ERC20_WRAP;
                        break;
                    }
                }
                else {
                    if(token == sellToken) {
                        // Path is e.g. dai-BTC-sBTC, swap dai to BTC, wrap BTC to sBTC
                        outputAddress = token; // unused
                        kind = ERC4626_SWAP_TYPE.ERC20_SWAP;
                        break;
                    }
                }

            }
        }
        else {
            for (uint256 i = 0; i < tokens.length; i++) {
                address token = address(tokens[i]);

                if(isERC4626(token)) {
                    if(IERC4626(token).asset() == buyToken) {
                        // Path is e.g. sDAI-DAI-BTC, unwrap sDAI to DAI, swap sDAI to BTC
                        outputAddress = token; // unused
                        kind = ERC4626_SWAP_TYPE.ERC4626_UNWRAP;
                        break;
                    }
                }
                else {
                    if(token == buyToken) {
                        // Path is e.g. sDAI-sBTC-BTC, swap sDAI to sBTC, unwrap sBTC to BTC
                        outputAddress = token; // buyToken.share()
                        kind = ERC4626_SWAP_TYPE.ERC20_SWAP;
                        break;
                    }
                }

            }
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
        IERC20 underlyingSellToken = IERC20(sellTokenShare.asset());
        bytes memory userData;

        // transfer sellToken.share() to address(this)
        IERC20(sellTokenShare).safeTransferFrom(
            msg.sender, address(this), specifiedAmount
        );

        if(kind == ERC4626_SWAP_TYPE.ERC4626_UNWRAP) {    
            // unwrap sellToken.shares() to sellToken.asset()
            (IBatchRouter.SwapPathExactAmountIn memory sellPathWrap,,) =
            createWrapOrUnwrapPath(
                _sellToken, specifiedAmount, IVault.WrappingDirection.UNWRAP, false
            );
            IBatchRouter.SwapPathExactAmountIn[] memory paths =
                new IBatchRouter.SwapPathExactAmountIn[](1);
            paths[0] = sellPathWrap;

            IERC20(sellTokenShare).safeIncreaseAllowance(permit2, specifiedAmount);
            IPermit2(permit2).approve(
                address(sellTokenShare),
                address(router),
                type(uint160).max,
                type(uint48).max
            );

            (,, uint256[] memory availableAmounts) =
                router.swapExactIn(paths, type(uint256).max, false, userData);

            // perform swap: sellToken.asset() -> buyToken
            calculatedAmount = sellERC20ForERC20(
                pool,
                underlyingSellToken,
                IERC20(buyToken),
                availableAmounts[0],
                true
            );
        }
        else {
            IBatchRouter.SwapPathExactAmountIn[] memory paths = new IBatchRouter.SwapPathExactAmountIn[](1);
            IBatchRouter.SwapPathStep[] memory steps = new IBatchRouter.SwapPathStep[](2);

            IERC20(sellTokenShare).safeIncreaseAllowance(permit2, specifiedAmount);
            IPermit2(permit2).approve(
                address(sellTokenShare),
                address(router),
                type(uint160).max,
                type(uint48).max
            );
            (,,IBatchRouter.SwapPathStep memory step0) =
            createERC20Path(pool, IERC20(_sellToken), IERC20(buyToken), specifiedAmount, false);
            steps[0] = step0;

            // unwrap buyToken.share() to buyToken
            (,,IBatchRouter.SwapPathStep memory step1) =
            createWrapOrUnwrapPath(
                outputAddress, specifiedAmount, IVault.WrappingDirection.UNWRAP, false
            );
            steps[1] = step1;

            (,, uint256[] memory amountsOut) =
            router.swapExactIn(paths, type(uint256).max, false, userData);

            calculatedAmount = amountsOut[0];

            IERC20(buyToken).safeTransfer(msg.sender, calculatedAmount);
        }
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
        uint256 specifiedAmount
    ) internal returns (uint256 calculatedAmount) {
        IERC4626 sellTokenShare = IERC4626(_sellToken);
        IERC20 underlyingSellToken = IERC20(sellTokenShare.asset());

        // a. UNWRAP: sellToken.shares() to sellToken.asset()
        (IBatchRouter.SwapPathExactAmountIn memory pathA,,) =
        createWrapOrUnwrapPath(
            _sellToken, specifiedAmount, IVault.WrappingDirection.UNWRAP, false
        );
        uint256 availableAmount = getAmountOut(pathA);

        // b. SWAP: sellToken.asset() -> buyToken
        (IBatchRouter.SwapPathExactAmountIn memory pathB,,) = createERC20Path(
            pool, underlyingSellToken, IERC20(buyToken), availableAmount, false
        );
        calculatedAmount = getAmountOut(pathB);
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
        uint256 specifiedAmount
    ) internal returns (uint256 calculatedAmount) {
        bytes memory userData; // empty bytes
        IERC20 underlyingSellToken = IERC20(IERC4626(_sellToken).asset());

        /**
         *
         * CALCULATE
         *
         */
        // a. SWAP (final step): sellToken.asset() -> BUY buyToken
        (, IBatchRouter.SwapPathExactAmountOut memory pathA,) = createERC20Path(
            pool, underlyingSellToken, IERC20(buyToken), specifiedAmount, true
        );
        IBatchRouter.SwapPathExactAmountOut[] memory pathsA =
            new IBatchRouter.SwapPathExactAmountOut[](1);
        pathsA[0] = pathA;
        underlyingSellToken.safeIncreaseAllowance(
            address(router), type(uint256).max
        );
        underlyingSellToken.safeIncreaseAllowance(permit2, type(uint256).max);
        IPermit2(permit2).approve(
            address(underlyingSellToken),
            address(router),
            type(uint160).max,
            type(uint48).max
        );

        // b. UNWRAP: sellToken.shares() -> sellToken.asset()
        (, IBatchRouter.SwapPathExactAmountOut memory pathB,) =
        createWrapOrUnwrapPath(
            _sellToken,
            IERC4626(_sellToken).balanceOf(msg.sender),
            IVault.WrappingDirection.UNWRAP,
            true
        );
        IBatchRouter.SwapPathExactAmountOut[] memory pathsB =
            new IBatchRouter.SwapPathExactAmountOut[](1);
        pathsB[0] = pathB;

        IERC20(_sellToken).safeIncreaseAllowance(
            address(router), type(uint256).max
        );
        IERC20(_sellToken).safeIncreaseAllowance(permit2, type(uint256).max);
        IPermit2(permit2).approve(
            _sellToken, address(router), type(uint160).max, type(uint48).max
        );

        // TODO

        /**
         *
         * TRANSFER
         *
         */
        IERC20(_sellToken).safeTransferFrom(
            msg.sender,
            address(this),
            IERC4626(_sellToken).balanceOf(msg.sender)
        );

        /**
         *
         * EXECUTE
         *
         */
        // b. UNWRAP: sellToken.shares() -> sellToken.asset()
        router.swapExactOut(pathsB, type(uint256).max, false, userData);
        // a. SWAP: sellToken.asset() -> BUY buyToken
        router.swapExactOut(pathsA, type(uint256).max, false, userData);

        IERC20(buyToken).safeTransfer(msg.sender, specifiedAmount);
    }

    /**
     * @param pool the ERC20->ERC4626.asset() pool
     */
    function sellERC20ForERC4626(
        address pool,
        address sellToken,
        address buyToken,
        uint256 specifiedAmount
    ) internal returns (uint256 calculatedAmount) {
        bytes memory userData; // empty bytes
        IERC20 buyTokenAsset = IERC20(IERC4626(buyToken).asset());

        // a. SWAP: sellToken.asset() -> buyToken.asset()
        uint256 amountOutSwap = sellERC20ForERC20(
            pool, IERC20(sellToken), buyTokenAsset, specifiedAmount, false
        );

        // b. WRAP: buyToken.asset() -> buyToken.share()
        (IBatchRouter.SwapPathExactAmountIn memory pathB,,) =
        createWrapOrUnwrapPath(
            buyToken, amountOutSwap, IVault.WrappingDirection.WRAP, true
        );

        buyTokenAsset.safeIncreaseAllowance(permit2, amountOutSwap);
        IPermit2(permit2).approve(
            address(buyTokenAsset),
            address(router),
            type(uint160).max,
            type(uint48).max
        );

        IBatchRouter.SwapPathExactAmountIn[] memory pathsB =
            new IBatchRouter.SwapPathExactAmountIn[](1);
        pathsB[0] = pathB;

        (,, uint256[] memory amountsOutWrap) =
            router.swapExactIn(pathsB, type(uint256).max, false, userData);

        // transfer
        IERC20(buyToken).safeTransfer(msg.sender, amountsOutWrap[0]);

        // return
        calculatedAmount = amountsOutWrap[0];
    }

    /**
     * @param pool the ERC20->ERC4626.asset() pool
     */
    function getAmountOutERC20ForERC4626(
        address pool,
        address sellToken,
        address buyToken,
        uint256 specifiedAmount
    ) internal returns (uint256 calculatedAmount) {
        IERC20 buyTokenAsset = IERC20(IERC4626(buyToken).asset());

        // a. SWAP: sellToken -> buyToken.asset()
        (IBatchRouter.SwapPathExactAmountIn memory pathA,,) = createERC20Path(
            pool, IERC20(sellToken), buyTokenAsset, specifiedAmount, false
        );
        uint256 availableAmount = getAmountOut(pathA);

        // b. WRAP: buyToken.asset() -> buyToken.shares()
        (IBatchRouter.SwapPathExactAmountIn memory pathB,,) =
        createWrapOrUnwrapPath(
            buyToken, availableAmount, IVault.WrappingDirection.WRAP, false
        );
        calculatedAmount = getAmountOut(pathB);
    }

    /**
     * @param pool the ERC20->ERC4626.asset() pool
     */
    function buyERC4626WithERC20(
        address pool,
        address sellToken,
        address buyToken,
        uint256 specifiedAmount
    ) internal returns (uint256 calculatedAmount) {
        bytes memory userData; // empty bytes
        IERC20 buyTokenAsset = IERC20(IERC4626(buyToken).asset());

        /**
         *
         * CALCULATE and Approve
         *
         */
        // a. WRAP (final step): buyToken.asset() -> buyToken.share
        (, IBatchRouter.SwapPathExactAmountOut memory pathA,) =
        createWrapOrUnwrapPath(
            buyToken, specifiedAmount, IVault.WrappingDirection.WRAP, true
        );
        IBatchRouter.SwapPathExactAmountOut[] memory pathsA =
            new IBatchRouter.SwapPathExactAmountOut[](1);
        pathsA[0] = pathA;

        buyTokenAsset.safeIncreaseAllowance(address(router), type(uint256).max);
        buyTokenAsset.safeIncreaseAllowance(address(router), type(uint256).max);
        buyTokenAsset.safeIncreaseAllowance(permit2, type(uint256).max);
        IPermit2(permit2).approve(
            address(buyTokenAsset),
            address(router),
            type(uint160).max,
            type(uint48).max
        );

        (,, uint256[] memory amountsA) =
            router.querySwapExactOut(pathsA, msg.sender, userData);

        // b. SWAP: sellToken.asset() -> buyToken.asset()
        (, IBatchRouter.SwapPathExactAmountOut memory pathB,) = createERC20Path(
            pool, IERC20(sellToken), buyTokenAsset, amountsA[0], true
        );
        IERC20(sellToken).safeIncreaseAllowance(
            address(router), type(uint256).max
        );
        IERC20(sellToken).safeIncreaseAllowance(permit2, type(uint256).max);
        IPermit2(permit2).approve(
            sellToken, address(router), type(uint160).max, type(uint48).max
        );
        IBatchRouter.SwapPathExactAmountOut[] memory pathsB =
            new IBatchRouter.SwapPathExactAmountOut[](1);
        pathsB[0] = pathB;

        (,, uint256[] memory amountsB) =
            router.querySwapExactOut(pathsB, msg.sender, userData);

        /**
         *
         * EXECUTE
         *
         */
        // b. SWAP: sellToken.asset() -> buyToken.asset()
        router.swapExactOut(pathsB, type(uint256).max, false, userData);
        // a. WRAP: buyToken.asset() -> buyToken.shares()
        router.swapExactOut(pathsA, type(uint256).max, false, userData);

        // transfer
        IERC20(buyToken).safeTransfer(msg.sender, specifiedAmount);

        // return
        calculatedAmount = amountsB[0];
    }

    function getLimitsERC4626ToERC20(
        bytes32 poolId,
        address sellToken,
        address buyToken
    ) internal view returns (uint256[] memory limits) {
        limits = new uint256[](2);
        address pool = address(bytes20(poolId));

        (IERC20[] memory tokens,, uint256[] memory balancesRaw,) =
            vault.getPoolTokenInfo(pool);

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
                limits[0] = (
                    IERC4626(sellToken).totalSupply() * RESERVE_LIMIT_FACTOR
                ) / 10;
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
        (IERC20[] memory tokens,, uint256[] memory balancesRaw,) =
            vault.getPoolTokenInfo(pool);

        IERC20 underlyingBuyToken = IERC20(IERC4626(sellToken).asset());
        IERC20 sellTokenERC = IERC20(sellToken);

        for (uint256 i = 0; i < tokens.length; i++) {
            if (tokens[i] == underlyingBuyToken) {
                /**
                 * @dev Using IERC4626(buyToken).totalSupply() as getAmountIn is
                 * not possible since this limit will also
                 * be used for non-static calls
                 */
                limits[1] = (
                    IERC4626(buyToken).totalSupply() * RESERVE_LIMIT_FACTOR
                ) / 10;
            }
            if (tokens[i] == sellTokenERC) {
                limits[0] = (balancesRaw[i] * RESERVE_LIMIT_FACTOR) / 10;
            }
        }
    }
}
