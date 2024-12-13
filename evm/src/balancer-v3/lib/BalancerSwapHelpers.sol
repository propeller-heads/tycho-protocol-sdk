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
        address poolAddress;
        if (isERC4626(sellToken) && isERC4626(buyToken)) {
            if (CustomBytesAppend.hasPrefix(abi.encodePacked(pool))) {
                poolAddress = CustomBytesAppend.extractAddress(pool);
                return getAmountOutCustomWrap(
                    poolAddress, sellToken, buyToken, specifiedAmount
                );
            }
        } else {
            poolAddress = address(bytes20(pool));
            if (isERC4626(sellToken) && !isERC4626(buyToken)) {
                return getAmountOutERC4626ForERC20(
                    poolAddress, sellToken, buyToken, specifiedAmount
                );
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

        (IBatchRouter.SwapPathExactAmountIn memory sellPath,) = createERC20Path(
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
        address poolAddress;

        if (isERC4626(sellToken) && isERC4626(buyToken)) {
            // custom swap
            if (CustomBytesAppend.hasPrefix(abi.encodePacked(pool))) {
                poolAddress = CustomBytesAppend.extractAddress(pool);
                // perform custom swap (ERC20<->ERC20, with ERC4626 inputs)
                if (side == OrderSide.Buy) {
                    return buyCustomWrap(
                        poolAddress, sellToken, buyToken, specifiedAmount
                    );
                } else {
                    return sellCustomWrap(
                        poolAddress, sellToken, buyToken, specifiedAmount
                    );
                }
            }
            // swap ERC4626<->ERC4626, fallback to next code block
        } else {
            poolAddress = address(bytes20(pool));
            if (isERC4626(sellToken) && !isERC4626(buyToken)) {
                // perform swap: ERC4626(share)->ERC20(token)
                if (side == OrderSide.Buy) {
                    return buyERC20WithERC4626(
                        poolAddress, sellToken, buyToken, specifiedAmount
                    );
                } else {
                    return sellERC4626ForERC20(
                        poolAddress, sellToken, buyToken, specifiedAmount
                    );
                }
            } else if (!isERC4626(sellToken) && isERC4626(buyToken)) {
                // perform swap: ERC20(token)->ERC4626(share)
                if (side == OrderSide.Buy) {
                    return buyERC4626WithERC20(
                        poolAddress, sellToken, buyToken, specifiedAmount
                    );
                } else {
                    return sellERC20ForERC4626(
                        poolAddress, sellToken, buyToken, specifiedAmount
                    );
                }
            }
            // swap ERC20<->ERC20, fallback to next code block
        }

        // Fallback (used for ERC20<->ERC20 and ERC4626<->ERC4626 as inherits
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
}
