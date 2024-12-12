//SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.26;

import "./BalancerInterfaces.sol";

/**
 * @title Balancer V3 Swap Helpers
 * @dev A wrapped library containing swap functions, helpers and storage for the
 * Balancer V3 Swap Adapter contract
 */
abstract contract BalancerSwapHelpers is ISwapAdapter {
    using SafeERC20 for IERC20;

    // Balancer V3 constants
    uint256 constant RESERVE_LIMIT_FACTOR = 3; // 0.3 as being divided by 10
    uint256 constant SWAP_DEADLINE_SEC = 1000;

    // Balancer V3 contracts
    IVault immutable vault;
    IBatchRouter immutable router;

    // ETH and Wrapped ETH addresses, using ETH as address(0)
    address constant WETH_ADDRESS = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
    address constant ETH_ADDRESS = address(0);

    /**
     * @dev Returns the amount of sellToken tokens to spend for a trade
     * @param path The path to get amountIn for
     * @return amountIn The amount of tokens to spend.
     */
    function getAmountIn(
        IBatchRouter.SwapPathExactAmountOut memory path
    ) internal returns (uint256 amountIn) {
        bytes memory userData; // empty bytes

        IBatchRouter.SwapPathExactAmountOut[]
            memory paths = new IBatchRouter.SwapPathExactAmountOut[](1);
        paths[0] = path;

        (, , uint256[] memory amountsIn) = router.querySwapExactOut(
            paths,
            address(0),
            userData
        );

        amountIn = amountsIn[0];
    }

    function getAmountOutMiddleware(
        bytes32 pool,
        address sellToken,
        address buyToken,
        uint256 specifiedAmount
    ) internal returns (uint256 amountOut) {
        address poolAddress;
        if (isERC4626(sellToken) && isERC4626(buyToken)) {
            if (CustomBytesAppend.hasPrefix(abi.encodePacked(pool))) {
                poolAddress = CustomBytesAppend.extractAddress(pool);
                return
                    getAmountOutCustomWrap(
                        poolAddress,
                        sellToken,
                        buyToken,
                        specifiedAmount
                    );
            }
        } else {
            if (isERC4626(sellToken) && !isERC4626(buyToken)) {
                // return
                //     getAmountOutERC4626ForERC20(
                //         pool,
                //         sellToken,
                //         buyToken,
                //         specifiedAmount
                //     );
            } else if (!isERC4626(sellToken) && isERC4626(buyToken)) {
                // return
                //     getAmountOutERC20ForERC4626(
                //         pool,
                //         sellToken,
                //         buyToken,
                //         specifiedAmount
                //     );
            }
        }

        // Fallback (used for ERC20<->ERC20 and ERC4626<->ERC4626 as inherits
        // IERC20 logic)
        poolAddress = address(bytes20(pool));
        (
            IBatchRouter.SwapPathExactAmountIn memory sellPath,

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
     * @dev Returns the amount of buyToken tokens received from a trade
     * @param path The path of the trade.
     * @return amountOut The amount of tokens to receive.
     */
    function getAmountOut(
        IBatchRouter.SwapPathExactAmountIn memory path
    ) internal returns (uint256 amountOut) {
        bytes memory userData; // empty bytes

        IBatchRouter.SwapPathExactAmountIn[]
            memory paths = new IBatchRouter.SwapPathExactAmountIn[](1);
        paths[0] = path;

        (, , uint256[] memory amountsOut) = router.querySwapExactIn(
            paths,
            address(0),
            userData
        );

        amountOut = amountsOut[0];
    }

    /**
     * @dev Perform a sell order for ERC20 tokens
     * @param pool The address of the pool to trade in.
     * @param sellToken The token being sold.
     * @param buyToken The token being bought.
     * @param specifiedAmount The amount to be traded.
     * @param performTransfer Whether to perform a transfer to msg.sender or
     * not(keeping tokens in the contract)
     * @return calculatedAmount The amount of tokens received.
     */
    function sellERC20ForERC20(
        address pool,
        IERC20 sellToken,
        IERC20 buyToken,
        uint256 specifiedAmount,
        bool performTransfer
    ) internal returns (uint256 calculatedAmount) {
        // prepare constants
        bytes memory userData;
        bool isETHSell = address(sellToken) == address(0);
        bool isETHBuy = address(sellToken) == address(0);

        // prepare path
        (
            IBatchRouter.SwapPathExactAmountIn memory sellPath,

        ) = createERC20Path(pool, sellToken, buyToken, specifiedAmount, false);
        IBatchRouter.SwapPathExactAmountIn[]
            memory paths = new IBatchRouter.SwapPathExactAmountIn[](1);
        paths[0] = sellPath;

        // prepare swap
        uint256[] memory amountsOut;
        if (isETHSell) {
            paths[0].tokenIn = IERC20(WETH_ADDRESS);
        } else {
            if (isETHBuy) {
                // adjust parameters for ETH buy
                paths[0].steps[0].tokenOut = IERC20(WETH_ADDRESS);
            }
            // Approve and Transfer ERC20 token
            sellToken.safeTransferFrom(
                msg.sender,
                address(this),
                specifiedAmount
            );
            sellToken.safeIncreaseAllowance(address(router), specifiedAmount);
        }

        // Swap (incl. WETH)
        (, , amountsOut) = router.swapExactIn(
            paths,
            type(uint256).max,
            isETHSell || isETHBuy,
            userData
        );

        // transfer if required
        if (performTransfer) {
            if (isETHBuy) {
                (bool sent, ) = payable(msg.sender).call{value: amountsOut[0]}(
                    ""
                );
                require(sent, "Failed to transfer ETH");
            } else {
                buyToken.safeTransfer(msg.sender, amountsOut[0]);
            }
        }

        // return amount
        calculatedAmount = amountsOut[0];
    }

    /**
     * @dev Perform a sell order for ERC20 tokens
     * @param pool The address of the pool to trade in.
     * @param sellToken The token being sold.
     * @param buyToken The token being bought.
     * @param specifiedAmount The amount to be traded.
     * @param performTransfer Whether to perform a transfer to msg.sender or
     * not(keeping tokens in the contract)
     * @return calculatedAmount The amount of tokens received.
     */
    function buyERC20WithERC20(
        address pool,
        IERC20 sellToken,
        IERC20 buyToken,
        uint256 specifiedAmount,
        bool performTransfer
    ) internal returns (uint256 calculatedAmount) {
        // prepare constants
        bytes memory userData;
        bool isETHSell = address(sellToken) == address(0);
        bool isETHBuy = address(sellToken) == address(0);

        // prepare path
        (
            ,
            IBatchRouter.SwapPathExactAmountOut memory buyPath
        ) = createERC20Path(pool, sellToken, buyToken, specifiedAmount, false);
        IBatchRouter.SwapPathExactAmountOut[]
            memory paths = new IBatchRouter.SwapPathExactAmountOut[](1);
        paths[0] = buyPath;

        // prepare swap
        uint256[] memory amountsIn;
        if (isETHSell) {
            // Set token in as WETH
            paths[0].tokenIn = IERC20(WETH_ADDRESS);
        } else {
            if (isETHBuy) {
                // adjust parameters for ETH buy
                paths[0].steps[0].tokenOut = IERC20(WETH_ADDRESS);
            }

            // Get amountIn
            uint256 amountIn = getAmountIn(paths[0]);

            // Approve and Transfer ERC20 token
            sellToken.safeTransferFrom(msg.sender, address(this), amountIn);
        }

        // perform swap
        (, , amountsIn) = router.swapExactOut(
            paths,
            type(uint256).max,
            isETHSell || isETHBuy,
            userData
        );

        // transfer if required
        if (performTransfer) {
            if (isETHBuy) {
                (bool sent, ) = payable(msg.sender).call{
                    value: specifiedAmount
                }("");
                require(sent, "Failed to transfer ETH");
            } else {
                buyToken.safeTransfer(msg.sender, specifiedAmount);
            }
        }

        // return amount
        calculatedAmount = amountsIn[0];
    }

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
        (
            IBatchRouter.SwapPathExactAmountIn memory sellPathWrap,

        ) = createWrapOrUnwrapPath(_sellToken, specifiedAmount, IVault.WrappingDirection.WRAP, true);
        uint256 amountOutAfterWrap = getAmountOut(sellPathWrap);

        // SWAP: sellToken.shares() -> buyToken.shares()
        (
            IBatchRouter.SwapPathExactAmountIn memory sellPathSharesSwap,
        ) = createERC20Path(
                pool,
                IERC20(_sellToken),
                IERC20(_buyToken),
                amountOutAfterWrap,
                false
            );
        uint256 amountOutAfterSharesSwap = getAmountOut(sellPathSharesSwap);

        // UNWRAP: buyToken.shares() -> buyToken.asset()
        (
            IBatchRouter.SwapPathExactAmountIn memory lastUnwrapPath,

        ) = createWrapOrUnwrapPath(
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
        (
            IBatchRouter.SwapPathExactAmountIn memory pathA,

        ) = createWrapOrUnwrapPath(_sellToken, specifiedAmount, IVault.WrappingDirection.WRAP, true);
        IBatchRouter.SwapPathExactAmountIn[] memory pathsA = new IBatchRouter.SwapPathExactAmountIn[](1);
        pathsA[0] = pathA;
        (, , uint256[] memory amountsOutA) = router.swapExactIn(pathsA, type(uint256).max, false, userData);

        // b. SWAP: sellToken.shares() -> buyToken.shares()
        (
            IBatchRouter.SwapPathExactAmountIn memory pathB,
        ) = createERC20Path(
                pool,
                IERC20(_sellToken),
                IERC20(_buyToken),
                amountsOutA[0],
                false
            );
        IBatchRouter.SwapPathExactAmountIn[] memory pathsB = new IBatchRouter.SwapPathExactAmountIn[](1);
        pathsB[0] = pathB;
        (, , uint256[] memory amountsOutB) = router.swapExactIn(pathsB, type(uint256).max, false, userData);

        // c. UNWRAP: buyToken.shares() -> buyToken.asset()
        (
            IBatchRouter.SwapPathExactAmountIn memory pathC,

        ) = createWrapOrUnwrapPath(
                _buyToken,
                amountsOutB[0],
                IVault.WrappingDirection.UNWRAP,
                false
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

        /************
         * CALCULATE
        *************/
        // a. UNWRAP (final step): buyToken.shares() -> BUY buyToken.asset()
        (, IBatchRouter.SwapPathExactAmountOut memory buyPathUnwrap) =
            createWrapOrUnwrapPath(_buyToken, specifiedAmount, IVault.WrappingDirection.UNWRAP, true);
        uint256 amountInUnwrap = getAmountIn(buyPathUnwrap);
        IBatchRouter.SwapPathExactAmountOut[] memory pathsA = new IBatchRouter.SwapPathExactAmountOut[](1);
        pathsA[0] = buyPathUnwrap;

        // b. SWAP: sellToken.shares() -> BUY buyToken.shares()
        (, IBatchRouter.SwapPathExactAmountOut memory buyPathSwap) =
        createERC20Path(pool, IERC20(_sellToken), underlyingSellToken, amountInUnwrap, true);
        uint256 amountInSwap = getAmountIn(buyPathSwap);
        IBatchRouter.SwapPathExactAmountOut[] memory pathsB = new IBatchRouter.SwapPathExactAmountOut[](1);
        pathsB[0] = buyPathSwap;

        // c. WRAP: sellToken.asset() -> sellToken.shares() - Our final amount
        (, IBatchRouter.SwapPathExactAmountOut memory buyPathFinal) =
            createWrapOrUnwrapPath(_sellToken, amountInSwap, IVault.WrappingDirection.WRAP, true);
        IBatchRouter.SwapPathExactAmountOut[] memory pathsC = new IBatchRouter.SwapPathExactAmountOut[](1);
        pathsC[0] = buyPathFinal;

        // Get amountIn (final)
        calculatedAmount = getAmountIn(buyPathFinal);

        /************
         * TRANSFER
        *************/
        underlyingSellToken.safeTransferFrom(
            msg.sender,
            address(this),
            calculatedAmount
        );
        underlyingSellToken.safeIncreaseAllowance(
            address(router),
            calculatedAmount
        );

        /************
         * EXECUTE
        *************/
        // c. WRAP: sellToken.asset() -> sellToken.shares()
        router.swapExactOut(pathsC, type(uint256).max, false, userData);

        // b. SWAP: sellToken.shares() -> BUY buyToken.shares()
        router.swapExactOut(pathsB, type(uint256).max, false, userData);

        // a. UNWRAP (final step): buyToken.shares() -> BUY buyToken.asset()
        router.swapExactOut(pathsA, type(uint256).max, false, userData);
    }

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

        // transfer sellToken.share() to address(this)
        IERC20(sellTokenShare).safeTransferFrom(
            msg.sender,
            address(this),
            specifiedAmount
        );

        // redeem sellToken.shares() to underlying
        uint256 availableAmount = sellTokenShare.redeem(
            specifiedAmount,
            address(this),
            address(this)
        );

        // perform swap: sellToken.asset() -> buyToken
        calculatedAmount = sellERC20ForERC20(
            pool,
            underlyingSellToken,
            IERC20(buyToken),
            availableAmount,
            true
        );
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
        // prepare path
        // (
        //     ,
        //     IBatchRouter.SwapPathExactAmountOut memory buyPath
        // ) =
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
        address poolAddress;
        if (isERC4626(sellToken) && isERC4626(buyToken)) {
            // custom swap
            if (CustomBytesAppend.hasPrefix(abi.encodePacked(pool))) {
                poolAddress = CustomBytesAppend.extractAddress(pool);
                // perform custom swap (ERC20<->ERC20, with ERC4626 inputs)
                if (side == OrderSide.Buy) {
                    return
                        buyCustomWrap(
                            poolAddress,
                            sellToken,
                            buyToken,
                            specifiedAmount
                        );
                } else {
                    return
                        sellCustomWrap(
                            poolAddress,
                            sellToken,
                            buyToken,
                            specifiedAmount
                        );
                }
            }
            // swap ERC4626<->ERC4626, fallback to next code block
        } else {
            poolAddress = address(bytes20(pool));
            if (isERC4626(sellToken) && !isERC4626(buyToken)) {
                // perform swap: ERC4626(share)<->ERC20(token)
                if (side == OrderSide.Buy) {
                    return
                        sellERC4626ForERC20(
                            poolAddress,
                            sellToken,
                            buyToken,
                            specifiedAmount
                        );
                } else {
                    return
                        buyERC20WithERC4626(
                            poolAddress,
                            sellToken,
                            buyToken,
                            specifiedAmount
                        );
                }
            } else if (!isERC4626(sellToken) && isERC4626(buyToken)) {
                // perform swap: ERC20(token)<->ERC4626(share)
                // if (side == OrderSide.Buy) {
                //     return
                //         sellERC20ForERC4626(
                //             poolAddress,
                //             sellToken,
                //             buyToken,
                //             specifiedAmount
                //         );
                // } else {
                //     return
                //         sellERC4626ForERC20(
                //             poolAddress,
                //             sellToken,
                //             buyToken,
                //             specifiedAmount
                //         );
                // }
            }
            // swap ERC20<->ERC20, fallback to next code block
        }

        // Fallback (used for ERC20<->ERC20 and ERC4626<->ERC4626 as inherits
        // IERC20 logic)
        poolAddress = address(bytes20(pool));
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

    function isERC4626(address token) internal view returns (bool) {
        try IERC4626(token).asset() {
            // If the call to asset() succeeds, the token likely implements
            // ERC4626
            return true;
        } catch {
            // If the call fails, the token does not implement ERC4626
            return false;
        }
    }

    /**
     * @notice Create a wrap or unwrap path in BalancerV3 router using buffer pools
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
        IBatchRouter.SwapPathStep[]
            memory steps = new IBatchRouter.SwapPathStep[](1);
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

    /**
     * @notice Create a ERC20 swap path in BalancerV3 router
     * @param sellToken (ERC20) token to Sell
     * @param buyToken (ERC20) token to Buy
     * @param specifiedAmount Amount to buy if isBuy, amount to sell else
     * @param isBuy True if buy, false if sell
     */
    function createERC20Path(
        address pool,
        IERC20 sellToken,
        IERC20 buyToken,
        uint256 specifiedAmount,
        bool isBuy
    )
        internal
        pure
        returns (
            IBatchRouter.SwapPathExactAmountIn memory sellPath,
            IBatchRouter.SwapPathExactAmountOut memory buyPath
        )
    {
        // prepare steps
        IBatchRouter.SwapPathStep memory step = IBatchRouter.SwapPathStep({
            pool: pool,
            tokenOut: buyToken,
            isBuffer: false
        });
        IBatchRouter.SwapPathStep[]
            memory steps = new IBatchRouter.SwapPathStep[](1);
        steps[0] = step;

        if (isBuy) {
            buyPath = IBatchRouter.SwapPathExactAmountOut({
                tokenIn: sellToken,
                steps: steps,
                maxAmountIn: type(uint256).max,
                exactAmountOut: specifiedAmount
            });
        } else {
            sellPath = IBatchRouter.SwapPathExactAmountIn({
                tokenIn: sellToken,
                steps: steps,
                exactAmountIn: specifiedAmount,
                minAmountOut: 1
            });
        }
    }
}
