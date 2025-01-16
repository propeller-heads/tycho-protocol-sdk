// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {IWAMPL} from "./IWAMPL.sol";
import {ISTAMPL} from "./ISTAMPL.sol";
import {IBillBroker} from "./IBillBroker.sol";

/**
 * @title AmpleforthSwapAdapter
 *
 * @notice An adapter contract that handles swaps between:
 *         AMPL <> WAMPL, AMPL <> SPOT and SPOT <> USDC.
 *
 */
contract AmpleforthSwapAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    /// @notice Address of AMPL token.
    address public constant AMPL = 0xD46bA6D942050d489DBd938a2C909A5d5039A161;
    /// @notice Address of WAMPL token (wrapped AMPL).
    address public constant WAMPL = 0xEDB171C18cE90B633DB442f2A6F72874093b49Ef;
    /// @notice Address of SPOT token (AMPL senior perpetual).
    address public constant SPOT = 0xC1f33e0cf7e40a67375007104B929E49a581bafE;
    /// @notice Address of stAMPL contract (AMPL junior perpetual).
    address public constant STAMPL = 0x82A91a0D599A45d8E9Af781D67f695d7C72869Bd;
    /// @notice Address of USDC token.
    address public constant USDC = 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48;
    /// @notice Address of BillBroker contract.
    address public constant BILL_BROKER =
        0xA088Aef966CAD7fE0B38e28c2E07590127Ab4ccB;

    /// @notice Constant that defines the limit factor for a POOL's reserve.
    uint256 public constant RESERVE_LIMIT_FACTOR = 10;

    /// @notice Constant that defines the limit factor for the STAMPL pool.
    uint256 public constant STAMPL_AMPL_RESERVE_LIMIT_FACTOR = 50;

    /// @inheritdoc ISwapAdapter
    function swap(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        OrderSide, /* side */
        uint256 specifiedAmount
    ) external override returns (Trade memory trade) {
        // No-op if zero amount
        if (specifiedAmount == 0) {
            return trade;
        }

        uint256 gasBefore = gasleft();
        address pool = address(bytes20(poolId));

        if (pool == WAMPL) {
            trade.calculatedAmount =
                _swapWAMPL(sellToken, buyToken, specifiedAmount);
        } else if (pool == STAMPL) {
            trade.calculatedAmount =
                _swapSTAMPL(sellToken, buyToken, specifiedAmount);
        } else if (pool == BILL_BROKER) {
            trade.calculatedAmount =
                _swapBillBroker(sellToken, buyToken, specifiedAmount);
        } else {
            revert Unavailable("PoolNotRecognized");
        }

        trade.gasUsed = gasBefore - gasleft();
        trade.price = Fraction(trade.calculatedAmount, specifiedAmount);
    }

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        uint256[] memory specifiedAmounts
    ) external override returns (Fraction[] memory prices) {
        address pool = address(bytes20(poolId));
        prices = new Fraction[](specifiedAmounts.length);

        if (pool == WAMPL) {
            _calculatePriceWAMPL(sellToken, buyToken, specifiedAmounts, prices);
        } else if (pool == STAMPL) {
            _calculatePriceSTAMPL(sellToken, buyToken, specifiedAmounts, prices);
        } else if (pool == BILL_BROKER) {
            _calculatePriceBillBroker(
                sellToken, buyToken, specifiedAmounts, prices
            );
        } else {
            revert Unavailable("PoolNotRecognized");
        }

        return prices;
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32 poolId, address sellToken, address buyToken)
        external
        returns (uint256[] memory limits)
    {
        limits = new uint256[](2);
        address pool = address(bytes20(poolId));

        if (pool == WAMPL) {
            _getWAMPLPoolLimits(limits, sellToken, buyToken);
        } else if (pool == STAMPL) {
            _getSTAMPLPoolLimits(limits, sellToken, buyToken);
        } else if (pool == BILL_BROKER) {
            _getBillBrokerPoolLimits(limits, sellToken, buyToken);
        } else {
            revert Unavailable("PoolNotRecognized");
        }
    }

    /// @inheritdoc ISwapAdapter
    function getCapabilities(
        bytes32 poolId,
        address, /* sellToken */
        address /* buyToken */
    ) external pure returns (Capability[] memory capabilities) {
        address pool = address(bytes20(poolId));
        if (pool == WAMPL) {
            capabilities = new Capability[](4);
            capabilities[0] = Capability.SellOrder;
            capabilities[1] = Capability.PriceFunction;
            capabilities[2] = Capability.ConstantPrice;
            capabilities[3] = Capability.HardLimits;
        } else if (pool == STAMPL) {
            capabilities = new Capability[](3);
            capabilities[0] = Capability.SellOrder;
            capabilities[1] = Capability.PriceFunction;
            capabilities[2] = Capability.HardLimits;
        } else if (pool == BILL_BROKER) {
            capabilities = new Capability[](3);
            capabilities[0] = Capability.SellOrder;
            capabilities[1] = Capability.PriceFunction;
            capabilities[2] = Capability.HardLimits;
        } else {
            revert Unavailable("PoolNotRecognized");
        }
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(bytes32 poolId)
        external
        pure
        returns (address[] memory tokens)
    {
        address pool = address(bytes20(poolId));
        if (pool == WAMPL) {
            tokens = new address[](2);
            tokens[0] = AMPL;
            tokens[1] = WAMPL;
        } else if (pool == STAMPL) {
            tokens = new address[](2);
            tokens[0] = AMPL;
            tokens[1] = SPOT;
        } else if (pool == BILL_BROKER) {
            tokens = new address[](2);
            tokens[0] = SPOT;
            tokens[1] = USDC;
        } else {
            revert Unavailable("PoolNotRecognized");
        }
    }

    /// @inheritdoc ISwapAdapter
    function getPoolIds(uint256, /* start */ uint256 /* end */ )
        external
        pure
        override
        returns (bytes32[] memory ids)
    {
        ids = new bytes32[](1);
        ids[0] = bytes32(bytes20(WAMPL));

        // NOTE: stAMPL and billbroker are pending substream integration
        // ids = new bytes32[](3);
        // ids[0] = bytes32(bytes20(WAMPL));
        // ids[1] = bytes32(bytes20(STAMPL));
        // ids[2] = bytes32(bytes20(BILL_BROKER));
    }

    /*//////////////////////////////////////////////////////////////
                           PRIVATE SWAP LOGIC
    //////////////////////////////////////////////////////////////*/

    /**
     * @dev Handles swapping when the pool is WAMPL.
     */
    function _swapWAMPL(
        address sellToken,
        address buyToken,
        uint256 specifiedAmount
    ) private returns (uint256) {
        if (sellToken == AMPL && buyToken == WAMPL) {
            IERC20(AMPL).safeTransferFrom(
                msg.sender, address(this), specifiedAmount
            );
            IERC20(AMPL).approve(WAMPL, specifiedAmount);
            return IWAMPL(WAMPL).depositFor(msg.sender, specifiedAmount);
        } else if (sellToken == WAMPL && buyToken == AMPL) {
            IERC20(WAMPL).safeTransferFrom(
                msg.sender, address(this), specifiedAmount
            );
            return IWAMPL(WAMPL).burnTo(msg.sender, specifiedAmount);
        } else {
            revert Unavailable("SwapRouteUndefined");
        }
    }

    /**
     * @dev Handles swapping when the pool is STAMPL.
     */
    function _swapSTAMPL(
        address sellToken,
        address buyToken,
        uint256 specifiedAmount
    ) private returns (uint256) {
        if (sellToken == AMPL && buyToken == SPOT) {
            IERC20(AMPL).safeTransferFrom(
                msg.sender, address(this), specifiedAmount
            );
            IERC20(AMPL).approve(STAMPL, specifiedAmount);
            uint256 amountOut =
                ISTAMPL(STAMPL).swapUnderlyingForPerps(specifiedAmount);
            IERC20(SPOT).safeTransfer(msg.sender, amountOut);
            return amountOut;
        } else if (sellToken == SPOT && buyToken == AMPL) {
            IERC20(SPOT).safeTransferFrom(
                msg.sender, address(this), specifiedAmount
            );
            IERC20(SPOT).approve(STAMPL, specifiedAmount);
            uint256 amountOut =
                ISTAMPL(STAMPL).swapPerpsForUnderlying(specifiedAmount);
            IERC20(AMPL).safeTransfer(msg.sender, amountOut);
            return amountOut;
        } else {
            revert Unavailable("SwapRouteUndefined");
        }
    }

    /**
     * @dev Handles swapping when the pool is BILL_BROKER.
     */
    function _swapBillBroker(
        address sellToken,
        address buyToken,
        uint256 specifiedAmount
    ) private returns (uint256) {
        if (sellToken == SPOT && buyToken == USDC) {
            IERC20(SPOT).safeTransferFrom(
                msg.sender, address(this), specifiedAmount
            );
            IERC20(SPOT).approve(BILL_BROKER, specifiedAmount);
            uint256 amountOut =
                IBillBroker(BILL_BROKER).swapPerpsForUSD(specifiedAmount, 0);
            IERC20(USDC).safeTransfer(msg.sender, amountOut);
            return amountOut;
        } else if (sellToken == USDC && buyToken == SPOT) {
            IERC20(USDC).safeTransferFrom(
                msg.sender, address(this), specifiedAmount
            );
            IERC20(USDC).approve(BILL_BROKER, specifiedAmount);
            uint256 amountOut =
                IBillBroker(BILL_BROKER).swapUSDForPerps(specifiedAmount, 0);
            IERC20(SPOT).safeTransfer(msg.sender, amountOut);
            return amountOut;
        } else {
            revert Unavailable("SwapRouteUndefined");
        }
    }

    /*//////////////////////////////////////////////////////////////
                          PRIVATE PRICING LOGIC
    //////////////////////////////////////////////////////////////*/

    /**
     * @dev Computes price quotes for WAMPL swaps.
     */
    function _calculatePriceWAMPL(
        address sellToken,
        address buyToken,
        uint256[] memory specifiedAmounts,
        Fraction[] memory prices
    ) private view {
        if (sellToken == AMPL && buyToken == WAMPL) {
            // AMPL -> WAMPL
            for (uint256 i = 0; i < specifiedAmounts.length; i++) {
                uint256 outAmt =
                    IWAMPL(WAMPL).underlyingToWrapper(specifiedAmounts[i]);
                prices[i] = Fraction(outAmt, specifiedAmounts[i]);
            }
        } else if (sellToken == WAMPL && buyToken == AMPL) {
            // WAMPL -> AMPL
            for (uint256 i = 0; i < specifiedAmounts.length; i++) {
                uint256 outAmt =
                    IWAMPL(WAMPL).wrapperToUnderlying(specifiedAmounts[i]);
                prices[i] = Fraction(outAmt, specifiedAmounts[i]);
            }
        } else {
            revert Unavailable("SwapRouteUndefined");
        }
    }

    /**
     * @dev Computes price quotes for STAMPL swaps.
     */
    function _calculatePriceSTAMPL(
        address sellToken,
        address buyToken,
        uint256[] memory specifiedAmounts,
        Fraction[] memory prices
    ) private {
        if (sellToken == AMPL && buyToken == SPOT) {
            // AMPL -> SPOT
            for (uint256 i = 0; i < specifiedAmounts.length; i++) {
                uint256 outAmt = ISTAMPL(STAMPL).computeUnderlyingToPerpSwapAmt(
                    specifiedAmounts[i]
                );
                prices[i] = Fraction(outAmt, specifiedAmounts[i]);
            }
        } else if (sellToken == SPOT && buyToken == AMPL) {
            // SPOT -> AMPL
            for (uint256 i = 0; i < specifiedAmounts.length; i++) {
                uint256 outAmt = ISTAMPL(STAMPL).computePerpToUnderlyingSwapAmt(
                    specifiedAmounts[i]
                );
                prices[i] = Fraction(outAmt, specifiedAmounts[i]);
            }
        } else {
            revert Unavailable("SwapRouteUndefined");
        }
    }

    /**
     * @dev Computes price quotes for BillBroker swaps.
     */
    function _calculatePriceBillBroker(
        address sellToken,
        address buyToken,
        uint256[] memory specifiedAmounts,
        Fraction[] memory prices
    ) private {
        if (sellToken == SPOT && buyToken == USDC) {
            // SPOT -> USDC
            for (uint256 i = 0; i < specifiedAmounts.length; i++) {
                uint256 outAmt = IBillBroker(BILL_BROKER)
                    .computePerpToUSDSwapAmt(specifiedAmounts[i]);
                prices[i] = Fraction(outAmt, specifiedAmounts[i]);
            }
        } else if (sellToken == USDC && buyToken == SPOT) {
            // USDC -> SPOT
            for (uint256 i = 0; i < specifiedAmounts.length; i++) {
                uint256 outAmt = IBillBroker(BILL_BROKER)
                    .computeUSDToPerpSwapAmt(specifiedAmounts[i]);
                prices[i] = Fraction(outAmt, specifiedAmounts[i]);
            }
        } else {
            revert Unavailable("SwapRouteUndefined");
        }
    }

    /*//////////////////////////////////////////////////////////////
                          PRIVATE LIMIT LOGIC
    //////////////////////////////////////////////////////////////*/

    /**
     * @dev Computes swap limits for WAMPL.
     */
    function _getWAMPLPoolLimits(
        uint256[] memory limits,
        address sellToken,
        address buyToken
    ) private view {
        if (sellToken == AMPL && buyToken == WAMPL) {
            limits[1] = IERC20(WAMPL).totalSupply() / RESERVE_LIMIT_FACTOR;
            limits[0] = IWAMPL(WAMPL).wrapperToUnderlying(limits[1]);
        } else if (sellToken == WAMPL && buyToken == AMPL) {
            limits[1] = IERC20(AMPL).balanceOf(WAMPL);
            limits[0] = IWAMPL(WAMPL).underlyingToWrapper(limits[1]);
        } else {
            revert Unavailable("SwapRouteUndefined");
        }
    }

    /**
     * @dev Computes swap limits for STAMPL.
     */
    function _getSTAMPLPoolLimits(
        uint256[] memory limits,
        address sellToken,
        address buyToken
    ) private {
        if (sellToken == AMPL && buyToken == SPOT) {
            limits[0] = IERC20(AMPL).balanceOf(STAMPL)
                / STAMPL_AMPL_RESERVE_LIMIT_FACTOR;
            limits[1] =
                ISTAMPL(STAMPL).computeUnderlyingToPerpSwapAmt(limits[0]);
        } else if (sellToken == SPOT && buyToken == AMPL) {
            limits[0] =
                IERC20(SPOT).totalSupply() / STAMPL_AMPL_RESERVE_LIMIT_FACTOR;
            limits[1] =
                ISTAMPL(STAMPL).computePerpToUnderlyingSwapAmt(limits[0]);
        } else {
            revert Unavailable("SwapRouteUndefined");
        }
    }

    /**
     * @dev Computes swap limits for BillBroker.
     */
    function _getBillBrokerPoolLimits(
        uint256[] memory limits,
        address sellToken,
        address buyToken
    ) private {
        if (sellToken == SPOT && buyToken == USDC) {
            limits[1] =
                IBillBroker(BILL_BROKER).usdBalance() / RESERVE_LIMIT_FACTOR;
            limits[0] =
                IBillBroker(BILL_BROKER).computeUSDToPerpSwapAmt(limits[1]);
        } else if (sellToken == USDC && buyToken == SPOT) {
            limits[1] =
                IBillBroker(BILL_BROKER).perpBalance() / RESERVE_LIMIT_FACTOR;
            limits[0] =
                IBillBroker(BILL_BROKER).computePerpToUSDSwapAmt(limits[1]);
        } else {
            revert Unavailable("SwapRouteUndefined");
        }
    }
}
