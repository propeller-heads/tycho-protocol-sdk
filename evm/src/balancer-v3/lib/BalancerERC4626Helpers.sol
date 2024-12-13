//SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.26;

import "./BalancerCustomWrapHelpers.sol";

abstract contract BalancerERC4626Helpers is BalancerCustomWrapHelpers {
    using SafeERC20 for IERC20;

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
        uint256 specifiedAmount
    ) internal returns (uint256 calculatedAmount) {
        IERC4626 sellTokenShare = IERC4626(_sellToken);
        IERC20 underlyingSellToken = IERC20(sellTokenShare.asset());
        bytes memory userData;

        // transfer sellToken.share() to address(this)
        IERC20(sellTokenShare).safeTransferFrom(
            msg.sender, address(this), specifiedAmount
        );

        // unwrap sellToken.shares() to sellToken.asset()
        (IBatchRouter.SwapPathExactAmountIn memory sellPathWrap,) =
        createWrapOrUnwrapPath(
            _sellToken, specifiedAmount, IVault.WrappingDirection.UNWRAP, false
        );
        IBatchRouter.SwapPathExactAmountIn[] memory paths =
            new IBatchRouter.SwapPathExactAmountIn[](1);
        paths[0] = sellPathWrap;
        IERC20(sellTokenShare).safeIncreaseAllowance(
            address(router), type(uint256).max
        );

        IERC20(sellTokenShare).safeIncreaseAllowance(permit2, type(uint256).max);
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
        (IBatchRouter.SwapPathExactAmountIn memory pathA,) =
        createWrapOrUnwrapPath(
            _sellToken, specifiedAmount, IVault.WrappingDirection.UNWRAP, false
        );
        uint256 availableAmount = getAmountOut(pathA);

        // b. SWAP: sellToken.asset() -> buyToken
        (IBatchRouter.SwapPathExactAmountIn memory pathB,) = createERC20Path(
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
        (, IBatchRouter.SwapPathExactAmountOut memory pathA) = createERC20Path(
            pool, underlyingSellToken, IERC20(buyToken), specifiedAmount, true
        );
        uint256 amountInSwap = getAmountIn(pathA);
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
        (, IBatchRouter.SwapPathExactAmountOut memory pathB) =
        createWrapOrUnwrapPath(
            _sellToken, amountInSwap, IVault.WrappingDirection.UNWRAP, true
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

        calculatedAmount = getAmountIn(pathB);

        /**
         *
         * TRANSFER
         *
         */
        IERC20(_sellToken).safeTransferFrom(
            msg.sender, address(this), calculatedAmount
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
        (IBatchRouter.SwapPathExactAmountIn memory pathA,) = createERC20Path(
            pool, IERC20(sellToken), buyTokenAsset, specifiedAmount, false
        );
        IBatchRouter.SwapPathExactAmountIn[] memory pathsA =
            new IBatchRouter.SwapPathExactAmountIn[](1);
        pathsA[0] = pathA;
        IERC20(sellToken).safeIncreaseAllowance(
            address(router), type(uint256).max
        );
        IERC20(sellToken).safeIncreaseAllowance(permit2, type(uint256).max);
        IPermit2(permit2).approve(
            sellToken, address(router), type(uint160).max, type(uint48).max
        );

        (,, uint256[] memory amountsInSwap) =
            router.swapExactIn(pathsA, type(uint256).max, false, userData);

        // b. WRAP: buyToken.asset() -> buyToken.share()
        (IBatchRouter.SwapPathExactAmountIn memory pathB,) =
        createWrapOrUnwrapPath(
            buyToken, amountsInSwap[0], IVault.WrappingDirection.WRAP, true
        );

        buyTokenAsset.safeIncreaseAllowance(address(router), type(uint256).max);
        buyTokenAsset.safeIncreaseAllowance(permit2, type(uint256).max);
        IPermit2(permit2).approve(
            address(buyTokenAsset),
            address(router),
            type(uint160).max,
            type(uint48).max
        );

        IBatchRouter.SwapPathExactAmountIn[] memory pathsB =
            new IBatchRouter.SwapPathExactAmountIn[](1);
        pathsB[0] = pathB;

        (,, uint256[] memory amountsOutUnwrap) =
            router.swapExactIn(pathsB, type(uint256).max, false, userData);

        // transfer
        IERC20(buyToken).safeTransfer(msg.sender, amountsOutUnwrap[0]);

        // return
        calculatedAmount = amountsOutUnwrap[0];
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
        (IBatchRouter.SwapPathExactAmountIn memory pathA,) = createERC20Path(
            pool, IERC20(sellToken), buyTokenAsset, specifiedAmount, false
        );
        uint256 availableAmount = getAmountOut(pathA);

        // b. WRAP: buyToken.asset() -> buyToken.shares()
        (IBatchRouter.SwapPathExactAmountIn memory pathB,) =
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
        (, IBatchRouter.SwapPathExactAmountOut memory pathA) =
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
        (, IBatchRouter.SwapPathExactAmountOut memory pathB) = createERC20Path(
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
}
