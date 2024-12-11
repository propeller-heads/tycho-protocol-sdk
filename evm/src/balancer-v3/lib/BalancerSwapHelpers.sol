//SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.26;

import "./BalancerInterfaces.sol";

/**
 * @title Balancer V3 Swap Helpers
 * @dev A wrapped library containing swap functions, helpers and storage for the Balancer V3 Swap Adapter contract
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
     * @dev Returns the amount of sellToken tokens to spend to get 'specifiedAmount' buyToken tokens
     * @param pool The address of the pool to trade in.
     * @param sellToken The token being sold.
     * @param buyToken The token being bought.
     * @param specifiedAmount The amount to be traded.
     * @return amountIn The amount of tokens to spend.
     */
    function getAmountIn(
        address pool,
        IERC20 sellToken,
        IERC20 buyToken,
        uint256 specifiedAmount
    ) internal returns (uint256 amountIn) {
        bytes memory userData; // empty bytes

        IBatchRouter.SwapPathStep memory step = IBatchRouter.SwapPathStep({
            pool: pool,
            tokenOut: buyToken,
            isBuffer: false
        });
        IBatchRouter.SwapPathStep[]
            memory steps = new IBatchRouter.SwapPathStep[](1);
        steps[0] = step;

        IBatchRouter.SwapPathExactAmountOut memory path = IBatchRouter
            .SwapPathExactAmountOut({
                tokenIn: sellToken,
                steps: steps,
                maxAmountIn: type(uint256).max,
                exactAmountOut: specifiedAmount
            });

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

    /**
     * @dev Returns the amount of buyToken tokens received by spending 'specifiedAmount' sellToken tokens
     * @param pool The address of the pool to trade in.
     * @param sellToken The token being sold.
     * @param buyToken The token being bought.
     * @param specifiedAmount The amount to be traded.
     * @return amountOut The amount of tokens to receive.
     */
    function getAmountOut(
        address pool,
        IERC20 sellToken,
        IERC20 buyToken,
        uint256 specifiedAmount
    ) internal returns (uint256 amountOut) {
        bytes memory userData; // empty bytes

        IBatchRouter.SwapPathStep memory step = IBatchRouter.SwapPathStep({
            pool: pool,
            tokenOut: buyToken,
            isBuffer: false
        });
        IBatchRouter.SwapPathStep[]
            memory steps = new IBatchRouter.SwapPathStep[](1);
        steps[0] = step;

        IBatchRouter.SwapPathExactAmountIn memory path = IBatchRouter
            .SwapPathExactAmountIn({
                tokenIn: sellToken,
                steps: steps,
                exactAmountIn: specifiedAmount,
                minAmountOut: 1
            });

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
     * @param performTransfer Whether to perform a transfer to msg.sender or not(keeping tokens in the contract)
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

        // prepare steps
        IBatchRouter.SwapPathStep memory step = IBatchRouter.SwapPathStep({
            pool: pool,
            tokenOut: buyToken,
            isBuffer: false
        });
        IBatchRouter.SwapPathStep[]
            memory steps = new IBatchRouter.SwapPathStep[](1);
        steps[0] = step;

        // prepare params
        IBatchRouter.SwapPathExactAmountIn memory path = IBatchRouter
            .SwapPathExactAmountIn({
                tokenIn: sellToken,
                steps: steps,
                exactAmountIn: specifiedAmount,
                minAmountOut: 1
            });
        IBatchRouter.SwapPathExactAmountIn[]
            memory paths = new IBatchRouter.SwapPathExactAmountIn[](1);
        paths[0] = path;

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
     * @param performTransfer Whether to perform a transfer to msg.sender or not(keeping tokens in the contract)
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

        // prepare steps
        IBatchRouter.SwapPathStep memory step = IBatchRouter.SwapPathStep({
            pool: pool,
            tokenOut: buyToken,
            isBuffer: false
        });
        IBatchRouter.SwapPathStep[]
            memory steps = new IBatchRouter.SwapPathStep[](1);
        steps[0] = step;

        // prepare params
        IBatchRouter.SwapPathExactAmountOut memory path = IBatchRouter
            .SwapPathExactAmountOut({
                tokenIn: sellToken,
                steps: steps,
                maxAmountIn: type(uint256).max,
                exactAmountOut: specifiedAmount
            });
        IBatchRouter.SwapPathExactAmountOut[]
            memory paths = new IBatchRouter.SwapPathExactAmountOut[](1);
        paths[0] = path;

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
            uint256 amountIn = getAmountIn(
                pool,
                sellToken,
                paths[0].steps[0].tokenOut,
                specifiedAmount
            );

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
     * @notice Perform a custom sell with wrap/unwrap
     * @dev
     * - Does not support ETH(gas), use wrapped ETH instead
     * - Using native vault's mint/redeem instead of BalancerV3's as it would use it when not enough liquidity
     *   and would also require unnecessary additional complexity
     * @param pool the ERC4626 pool containing sellToken.share() and buyToken.share()
     * @param _sellToken ERC4626 token being sold, of which .asset() is the buyToken
     * @param _buyToken ERC4626 token of which .asset() is the buyToken
     * @param specifiedAmount The amount of _sellToken.asset() tokens spent
     */
    function sellCustomWrap(
        address pool,
        address _sellToken,
        address _buyToken,
        uint256 specifiedAmount
    ) internal returns (uint256 calculatedAmount) {
        // prepare tokens
        IERC4626 sellToken = IERC4626(_sellToken);
        IERC4626 buyToken = IERC4626(_buyToken);
        IERC20 underlyingSellToken = IERC20(sellToken.asset());

        // transfer sellToken's asset to address(this)
        underlyingSellToken.safeTransferFrom(
            msg.sender,
            address(this),
            specifiedAmount
        );

        // buy sellToken shares (wrap)
        uint256 shares = sellToken.deposit(specifiedAmount, address(this));

        // perform swap: sellToken.shares() -> buyToken.shares()
        uint256 amountOut = sellERC20ForERC20(
            pool,
            IERC20(address(sellToken)),
            IERC20(address(buyToken)),
            shares,
            false
        );

        // redeem buyToken shares and return the underlying received
        calculatedAmount = buyToken.redeem(
            amountOut,
            address(msg.sender),
            address(this)
        );
    }

    /**
     * @notice Perform a custom sell with wrap/unwrap
     * @dev
     * - Does not support ETH(gas), use wrapped ETH instead
     * - Using native vault's mint/redeem instead of BalancerV3's as it would use it when not enough liquidity
     *   and would also require unnecessary additional complexity
     * @param pool the ERC4626 pool containing sellToken.share() and buyToken.share()
     * @param _sellToken ERC4626 token being sold, of which .asset() is the buyToken
     * @param _buyToken ERC4626 token of which .asset() is the buyToken
     * @param specifiedAmount The amount of _buyToken.asset() tokens to receive
     */
    function buyCustomWrap(
        address pool,
        address _sellToken,
        address _buyToken,
        uint256 specifiedAmount
    ) internal returns (uint256 calculatedAmount) {
        // prepare tokens
        IERC4626 sellToken = IERC4626(_sellToken);
        IERC4626 buyToken = IERC4626(_buyToken);
        IERC20 underlyingSellToken = IERC20(sellToken.asset());

        // get the amount of buyToken.shares() required to redeem "specifiedAmount" buyToken.asset()
        uint256 buyTokenSharesRequiredAmount = buyToken.previewWithdraw(
            specifiedAmount
        );

        // get sellToken.shares() required to get buyToken.shares() via Balancer swap
        uint256 sellTokenSharesRequiredAmount = getAmountIn(
            pool,
            IERC20(_sellToken),
            IERC20(_buyToken),
            buyTokenSharesRequiredAmount
        );

        // get sellToken.asset() required to get "sellTokenSharesRequiredAmount" sellToken.shares(), our final amount
        calculatedAmount = sellToken.previewRedeem(
            sellTokenSharesRequiredAmount
        );

        // transfer sellToken.asset() to address(this) and approve
        underlyingSellToken.safeTransferFrom(
            msg.sender,
            address(this),
            calculatedAmount
        );
        underlyingSellToken.safeIncreaseAllowance(
            address(router),
            sellTokenSharesRequiredAmount
        );

        // mint sellToken.shares()
        sellToken.mint(sellTokenSharesRequiredAmount, address(this));

        // perform the swap: sellToken.shares() -> buyToken.shares()
        buyERC20WithERC20(
            pool,
            IERC20(_sellToken),
            IERC20(_buyToken),
            buyTokenSharesRequiredAmount,
            false
        );

        // unwrap buyToken.shares()
        buyToken.redeem(specifiedAmount, address(msg.sender), address(this));
    }

    /**
     * @dev Perform a sell order for ERC4626 tokens
     * @param pool The pool containing sellToken.asset() and buyToken
     * @param sellToken ERC4626 token being sold(by unwrapping to sellToken.asset())
     * @param buyToken ERC20 token being bought
     * @param amount The amount of sellToken(ERC4626) tokens to sell
     */
    function sellERC4626ForERC20(
        address pool,
        address sellToken,
        address buyToken,
        uint256 amount
    ) internal returns (uint256 calculatedAmount) {}

    /**
     * @dev Perform a sell order for ERC4626 tokens
     * @param pool The pool containing sellToken.share() and buyToken
     * @param sellToken ERC4626 token being sold, of which .asset() is the sellToken
     * @param buyToken ERC20 token being bought
     * @param amount The amount of buyToken to buy
     */
    function buyERC4626WithERC20(
        address pool,
        address sellToken,
        address buyToken,
        uint256 amount
    ) internal returns (uint256 calculatedAmount) {}

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
                        buyERC4626WithERC20(
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

        // Fallback (used for ERC20<->ERC20 and ERC4626<->ERC4626 as inherits IERC20 logic)
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
            // If the call to asset() succeeds, the token likely implements ERC4626
            return true;
        } catch {
            // If the call fails, the token does not implement ERC4626
            return false;
        }
    }
}
