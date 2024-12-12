//SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.26;

import "./BalancerCustomWrapHelpers.sol";

abstract contract BalancerERC4626Helpers is BalancerCustomWrapHelpers {
    using SafeERC20 for IERC20;

    /**
     * @dev Perform a sell order for ERC4626 tokens
     * @param pool The pool containing sellToken.asset() and buyToken
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
            _sellToken, specifiedAmount, IVault.WrappingDirection.WRAP, false
        );
        IBatchRouter.SwapPathExactAmountIn[] memory paths =
            new IBatchRouter.SwapPathExactAmountIn[](1);
        paths[0] = sellPathWrap;
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
     * @param pool The pool containing sellToken.asset() and buyToken
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

        // a. sellToken.shares() to sellToken.asset()
        (IBatchRouter.SwapPathExactAmountIn memory pathA,) =
        createWrapOrUnwrapPath(
            _sellToken, specifiedAmount, IVault.WrappingDirection.WRAP, false
        );
        uint256 availableAmount = getAmountOut(pathA);

        // b. sellToken.asset() -> buyToken
        (IBatchRouter.SwapPathExactAmountIn memory pathB,) = createERC20Path(
            pool, underlyingSellToken, IERC20(buyToken), availableAmount, false
        );
        calculatedAmount = getAmountOut(pathB);
    }

    /**
     * @dev Perform a sell order for ERC4626 tokens
     * @param pool The pool containing sellToken.share() and buyToken
     * @param sellToken ERC4626 token being sold, of which .asset() is the
     * sellToken
     * @param buyToken ERC20 token being bought
     * @param amount The amount of buyToken to buy
     */
    function buyERC20WithERC4626(
        address pool,
        address sellToken,
        address buyToken,
        uint256 amount
    ) internal returns (uint256 calculatedAmount) {
        bytes memory userData; // empty bytes

        // a. SWAP: sellToken.asset() -> buyToken.asset()
        (, IBatchRouter.SwapPathExactAmountOut memory pathA) = createERC20Path(
            pool, IERC20(sellToken), IERC20(buyToken), amount, true
        );
        IBatchRouter.SwapPathExactAmountOut[] memory pathsA =
            new IBatchRouter.SwapPathExactAmountOut[](1);
        pathsA[0] = pathA;
        (,, uint256[] memory amountsInSwap) =
            router.swapExactOut(pathsA, type(uint256).max, false, userData);

        // b. UNWRAP: sellToken.shares() -> sellToken.asset()
        (, IBatchRouter.SwapPathExactAmountOut memory pathB) =
        createWrapOrUnwrapPath(
            sellToken, amountsInSwap[0], IVault.WrappingDirection.UNWRAP, true
        );
        IBatchRouter.SwapPathExactAmountOut[] memory pathsB =
            new IBatchRouter.SwapPathExactAmountOut[](1);
        pathsB[0] = pathB;
        (,, uint256[] memory amountsInUnwrap) =
            router.swapExactOut(pathsB, type(uint256).max, false, userData);

        // return
        calculatedAmount = amountsInUnwrap[0];
    }
}
