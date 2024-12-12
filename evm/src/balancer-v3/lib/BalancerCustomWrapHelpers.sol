//SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.26;

import "./BalancerERC20Helpers.sol";

abstract contract BalancerCustomWrapHelpers is BalancerERC20Helpers {
    using SafeERC20 for IERC20;

    /**
     * @notice Get amount out for custom wrap
     */
    function getAmountOutCustomWrap(
        address pool,
        address _sellToken,
        address _buyToken,
        uint256 specifiedAmount
    ) internal returns (uint256 calculatedAmount) {
        // WRAP: sellToken.asset() -> sellToken.shares()
        (IBatchRouter.SwapPathExactAmountIn memory sellPathWrap,) =
        createWrapOrUnwrapPath(
            _sellToken, specifiedAmount, IVault.WrappingDirection.WRAP, true
        );
        uint256 amountOutAfterWrap = getAmountOut(sellPathWrap);

        // SWAP: sellToken.shares() -> buyToken.shares()
        (IBatchRouter.SwapPathExactAmountIn memory sellPathSharesSwap,) =
        createERC20Path(
            pool,
            IERC20(_sellToken),
            IERC20(_buyToken),
            amountOutAfterWrap,
            false
        );
        uint256 amountOutAfterSharesSwap = getAmountOut(sellPathSharesSwap);

        // UNWRAP: buyToken.shares() -> buyToken.asset()
        (IBatchRouter.SwapPathExactAmountIn memory lastUnwrapPath,) =
        createWrapOrUnwrapPath(
            _buyToken,
            amountOutAfterSharesSwap,
            IVault.WrappingDirection.UNWRAP,
            false
        );
        calculatedAmount = getAmountOut(lastUnwrapPath);
    }

    /**
     * @notice Perform a custom sell with wrap/unwrap
     * @dev
     * - Does not support ETH(gas), use wrapped ETH instead
     * - Using native vault's mint/redeem instead of BalancerV3's as it would
     * use it when not enough liquidity
     *   and would also require unnecessary additional complexity
     * @param pool the ERC4626 pool containing sellToken.share() and
     * buyToken.share()
     * @param _sellToken ERC4626 token being sold, of which .asset() is the
     * buyToken
     * @param _buyToken ERC4626 token of which .asset() is the buyToken
     * @param specifiedAmount The amount of _sellToken.asset() tokens spent
     */
    function sellCustomWrap(
        address pool,
        address _sellToken,
        address _buyToken,
        uint256 specifiedAmount
    ) internal returns (uint256 calculatedAmount) {
        bytes memory userData;

        // a. WRAP: sellToken.asset() -> sellToken.shares()
        (IBatchRouter.SwapPathExactAmountIn memory pathA,) =
        createWrapOrUnwrapPath(
            _sellToken, specifiedAmount, IVault.WrappingDirection.WRAP, true
        );
        IBatchRouter.SwapPathExactAmountIn[] memory pathsA =
            new IBatchRouter.SwapPathExactAmountIn[](1);
        pathsA[0] = pathA;
        (,, uint256[] memory amountsOutA) =
            router.swapExactIn(pathsA, type(uint256).max, false, userData);

        // b. SWAP: sellToken.shares() -> buyToken.shares()
        (IBatchRouter.SwapPathExactAmountIn memory pathB,) = createERC20Path(
            pool, IERC20(_sellToken), IERC20(_buyToken), amountsOutA[0], false
        );
        IBatchRouter.SwapPathExactAmountIn[] memory pathsB =
            new IBatchRouter.SwapPathExactAmountIn[](1);
        pathsB[0] = pathB;
        (,, uint256[] memory amountsOutB) =
            router.swapExactIn(pathsB, type(uint256).max, false, userData);

        // c. UNWRAP: buyToken.shares() -> buyToken.asset()
        (IBatchRouter.SwapPathExactAmountIn memory pathC,) =
        createWrapOrUnwrapPath(
            _buyToken, amountsOutB[0], IVault.WrappingDirection.UNWRAP, false
        );
        calculatedAmount = getAmountOut(pathC);
    }

    /**
     * @notice Perform a custom sell with wrap/unwrap
     * @dev
     * - Does not support ETH(gas), use wrapped ETH instead
     * - Using native vault's mint/redeem instead of BalancerV3's as it would
     * use it when not enough liquidity
     *   and would also require unnecessary additional complexity
     * @param pool the ERC4626 pool containing sellToken.share() and
     * buyToken.share()
     * @param _sellToken ERC4626 token being sold, of which .asset() is the
     * buyToken
     * @param _buyToken ERC4626 token of which .asset() is the buyToken
     * @param specifiedAmount The amount of _buyToken.asset() tokens to receive
     */
    function buyCustomWrap(
        address pool,
        address _sellToken,
        address _buyToken,
        uint256 specifiedAmount
    ) internal returns (uint256 calculatedAmount) {
        // prepare data
        IERC20 underlyingSellToken = IERC20(IERC4626(_sellToken).asset());
        bytes memory userData; // empty bytes

        /**
         *
         * CALCULATE
         *
         */
        // a. UNWRAP (final step): buyToken.shares() -> BUY buyToken.asset()
        (, IBatchRouter.SwapPathExactAmountOut memory buyPathUnwrap) =
        createWrapOrUnwrapPath(
            _buyToken, specifiedAmount, IVault.WrappingDirection.UNWRAP, true
        );
        uint256 amountInUnwrap = getAmountIn(buyPathUnwrap);
        IBatchRouter.SwapPathExactAmountOut[] memory pathsA =
            new IBatchRouter.SwapPathExactAmountOut[](1);
        pathsA[0] = buyPathUnwrap;

        // b. SWAP: sellToken.shares() -> BUY buyToken.shares()
        (, IBatchRouter.SwapPathExactAmountOut memory buyPathSwap) =
        createERC20Path(
            pool, IERC20(_sellToken), underlyingSellToken, amountInUnwrap, true
        );
        uint256 amountInSwap = getAmountIn(buyPathSwap);
        IBatchRouter.SwapPathExactAmountOut[] memory pathsB =
            new IBatchRouter.SwapPathExactAmountOut[](1);
        pathsB[0] = buyPathSwap;

        // c. WRAP: sellToken.asset() -> sellToken.shares() - Our final amount
        (, IBatchRouter.SwapPathExactAmountOut memory buyPathFinal) =
        createWrapOrUnwrapPath(
            _sellToken, amountInSwap, IVault.WrappingDirection.WRAP, true
        );
        IBatchRouter.SwapPathExactAmountOut[] memory pathsC =
            new IBatchRouter.SwapPathExactAmountOut[](1);
        pathsC[0] = buyPathFinal;

        // Get amountIn (final)
        calculatedAmount = getAmountIn(buyPathFinal);

        /**
         *
         * TRANSFER
         *
         */
        underlyingSellToken.safeTransferFrom(
            msg.sender, address(this), calculatedAmount
        );
        underlyingSellToken.safeIncreaseAllowance(
            address(router), calculatedAmount
        );

        /**
         *
         * EXECUTE
         *
         */
        // c. WRAP: sellToken.asset() -> sellToken.shares()
        router.swapExactOut(pathsC, type(uint256).max, false, userData);

        // b. SWAP: sellToken.shares() -> BUY buyToken.shares()
        router.swapExactOut(pathsB, type(uint256).max, false, userData);

        // a. UNWRAP (final step): buyToken.shares() -> BUY buyToken.asset()
        router.swapExactOut(pathsA, type(uint256).max, false, userData);
    }

    /**
     * @notice Create a wrap or unwrap path in BalancerV3 router using buffer
     * pools
     * @param token (ERC4626) token to Wrap or Unwrap
     * @param amount Amount to buy if isBuy, amount to sell else
     * @param direction Wrap or Unwrap
     * @param isBuy True if buy, false if sell
     */
    function createWrapOrUnwrapPath(
        address token,
        uint256 amount,
        IVault.WrappingDirection direction,
        bool isBuy
    )
        internal
        view
        returns (
            IBatchRouter.SwapPathExactAmountIn memory sellPath,
            IBatchRouter.SwapPathExactAmountOut memory buyPath
        )
    {
        IBatchRouter.SwapPathStep memory step = IBatchRouter.SwapPathStep({
            pool: token,
            tokenOut: direction == IVault.WrappingDirection.UNWRAP
                ? IERC20(IERC4626(token).asset())
                : IERC20(token),
            isBuffer: true
        });
        IBatchRouter.SwapPathStep[] memory steps =
            new IBatchRouter.SwapPathStep[](1);
        steps[0] = step;

        if (isBuy) {
            buyPath = IBatchRouter.SwapPathExactAmountOut({
                tokenIn: direction == IVault.WrappingDirection.UNWRAP
                    ? IERC20(token)
                    : IERC20(IERC4626(token).asset()),
                steps: steps,
                maxAmountIn: type(uint256).max,
                exactAmountOut: amount
            });
        } else {
            sellPath = IBatchRouter.SwapPathExactAmountIn({
                tokenIn: direction == IVault.WrappingDirection.UNWRAP
                    ? IERC20(token)
                    : IERC20(IERC4626(token).asset()),
                steps: steps,
                exactAmountIn: amount,
                minAmountOut: 1
            });
        }
    }
}
