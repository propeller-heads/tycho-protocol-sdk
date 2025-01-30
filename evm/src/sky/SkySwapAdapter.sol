pragma solidity ^0.8.13;
// SPDX-License-Identifier: AGPL-3.0-or-later

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {IERC20Metadata} from
    "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {
    IERC20,
    SafeERC20
} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC4626} from
    "openzeppelin-contracts/contracts/interfaces/IERC4626.sol";
import "forge-std/Test.sol";

/**
 * @title SkySwapAdapter
 * @notice Adapter for SkySwap
 * @dev This adapter supports the following token pairs:
 *      - DAI <-> sDAI
 *      - DAI <-> USDC | 1:1
 *      - DAI <-> USDS | 1:1
 *      - USDS <-> USDC | 1:1
 *      - USDS <-> sUSDS | 1:1
 *      - MKR <-> SKY | 1:24000
 */
contract SkySwapAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;
    using SafeERC20 for ISavingsDai;

    struct Pair {
        address token0;
        address token1;
    }

    mapping(bytes32 => Pair) public pairs;

    uint256 private constant PRECISION = 10 ** 18;
    uint256 private constant MKR_TO_SKY_RATE = 24000;
    uint256 private constant RESERVE_FACTOR = 10;

    /**
     * @dev SavingsDai swaps DAI <-> sDAI.
     *      Address: 0x83F20F44975D03b1b09e64809B757c47f942BEeA
     */
    ISavingsDai immutable sDai;
    /**
     * @dev DSSLitePsm swaps DAI <-> USDC (referred to as "gem") at a fixed
     * ratio of 1:1.
     *      Fees `tin` and `tout` might apply.
     *      `gem` balance is kept in `pocket` instead of this contract.
     *      Address: 0xf6e72Db5454dd049d0788e411b06CfAF16853042
     */
    IDssLitePSM immutable daiLitePSM;
    /**
     * @dev DaiUsdsConverter converts DAI <-> USDS at a fixed ratio of 1:1.
     *      No fees assessed. Fees cannot be enabled on this route in the
     * future.
     *      Address: 0x3225737a9Bbb6473CB4a45b7244ACa2BeFdB276A
     */
    IDaiUsdsConverter immutable daiUsdsConverter;
    /**
     * @dev USDSPSMWrapper swaps USDS <-> USDC (referred to as "gem") at a fixed
     * ratio of 1:1.
     *      Fees `tin` and `tout` might apply.
     *      It uses DAI as the intermediary token.
     *      Address: 0xA188EEC8F81263234dA3622A406892F3D630f98c
     */
    IUsdsPsmWrapper immutable usdsPsmWrapper;
    /**
     * @dev sUSDS swaps USDS <-> sUSDS
     *      Contract contains an ERC4626 compatible interface to allow users to
     *      deposit USDS to receive sUSDS or withdraw USDS with their sUSDS
     * balance.
     *      No fees assessed. Fees cannot be enabled on this route in the
     * future.
     *      Address: 0xa3931d71877C0E7a3148CB7Eb4463524FEc27fbD
     */
    ISUsds immutable sUsds;
    /**
     * @dev MkMkrSkyConverter converts MKR <-> SKY at a fixed ratio of 1:24000.
     *      if minting capabilities are removed from either token, conversion
     * for that token will no longer be possible.
     *      Address: 0xBDcFCA946b6CDd965f99a839e4435Bcdc1bc470B
     */
    IMkrSkyConverter immutable mkrSkyConverter;

    IERC20 immutable dai; // 0x6B175474E89094C44Da98b954EedeAC495271d0F
    IERC20 immutable usds; // 0xdC035D45d973E3EC169d2276DDab16f1e407384F
    IERC20 immutable usdc; // 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
    IERC20 immutable mkr; // 0x9f8F72aA9304c8B593d555F12eF6589cC3A579A2
    IERC20 immutable sky; // 0x56072C95FAA701256059aa122697B133aDEd9279

    constructor(
        address savingsDai_,
        address daiLitePSM_,
        address daiUsdsConverter_,
        address usdsPsmWrapper_,
        address sUsds_,
        address mkrSkyConverter_,
        address dai_,
        address usds_,
        address usdc_,
        address mkr_,
        address sky_
    ) {
        sDai = ISavingsDai(savingsDai_);
        daiLitePSM = IDssLitePSM(daiLitePSM_);
        daiUsdsConverter = IDaiUsdsConverter(daiUsdsConverter_);
        usdsPsmWrapper = IUsdsPsmWrapper(usdsPsmWrapper_);
        sUsds = ISUsds(sUsds_);
        mkrSkyConverter = IMkrSkyConverter(mkrSkyConverter_);
        dai = IERC20(dai_);
        usds = IERC20(usds_);
        usdc = IERC20(usdc_);
        mkr = IERC20(mkr_);
        sky = IERC20(sky_);

        pairs[bytes32(bytes20(address(sDai)))] =
            Pair(address(dai), address(sDai));
        pairs[bytes32(bytes20(address(daiLitePSM)))] =
            Pair(address(dai), address(usdc));
        pairs[bytes32(bytes20(address(daiUsdsConverter)))] =
            Pair(address(dai), address(usds));
        pairs[bytes32(bytes20(address(usdsPsmWrapper)))] =
            Pair(address(usds), address(usdc));
        pairs[bytes32(bytes20(address(sUsds)))] =
            Pair(address(usds), address(sUsds));
        pairs[bytes32(bytes20(address(mkrSkyConverter)))] =
            Pair(address(mkr), address(sky));
    }

    /**
     * @dev Check if swap between provided sellToken and buyToken are supported
     *      by this adapter.
     */
    modifier checkInputPoolIdAndTokens(bytes32 poolId, address sellToken, address buyToken) {
        if (poolId != bytes32(bytes20(address(sDai)))
            && poolId != bytes32(bytes20(address(daiLitePSM)))
            && poolId != bytes32(bytes20(address(daiUsdsConverter)))
            && poolId != bytes32(bytes20(address(usdsPsmWrapper)))
            && poolId != bytes32(bytes20(address(sUsds)))
            && poolId != bytes32(bytes20(address(mkrSkyConverter)))) {
            revert Unavailable("SkySwapAdapter: Unsupported Pool");
        }
        
        if (sellToken == buyToken) {
            revert Unavailable("SkySwapAdapter: sellToken and buyToken cannot be the same");
        }

        if (poolId == bytes32(bytes20(address(sDai)))) {
            if (sellToken == address(dai) && buyToken != address(sDai) || sellToken == address(sDai) && buyToken != address(dai)) {
                revert Unavailable("SkySwapAdapter: Unsupported token pair");
            }
        }

        if (poolId == bytes32(bytes20(address(daiLitePSM)))) {
            if (sellToken == address(usdc) && buyToken != address(dai) || sellToken == address(dai) && buyToken != address(usdc)) {
                revert Unavailable("SkySwapAdapter: Unsupported token pair");
            }
        }

        if (poolId == bytes32(bytes20(address(daiUsdsConverter)))) {
            if (sellToken == address(usds) && buyToken != address(dai) || sellToken == address(dai) && buyToken != address(usds)) {
                revert Unavailable("SkySwapAdapter: Unsupported token pair");
            }
        }

        if (poolId == bytes32(bytes20(address(usdsPsmWrapper)))) {
            if (sellToken == address(usdc) && buyToken != address(usds) || sellToken == address(usds) && buyToken != address(usdc)) {
                revert Unavailable("SkySwapAdapter: Unsupported token pair");
            }
        }

        if (poolId == bytes32(bytes20(address(sUsds)))) {
            if (sellToken == address(usds) && buyToken != address(sUsds) || sellToken == address(sUsds) && buyToken != address(usds)) {
                revert Unavailable("SkySwapAdapter: Unsupported token pair");
            }
        }

        if (poolId == bytes32(bytes20(address(mkrSkyConverter)))) {
            if (sellToken == address(mkr) && buyToken != address(sky) || sellToken == address(sky) && buyToken != address(mkr)) {
                revert Unavailable("SkySwapAdapter: Unsupported token pair");
            }
        }

        _;
    }

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        uint256[] memory specifiedAmounts
    )
        external
        view
        override
        checkInputPoolIdAndTokens(poolId, sellToken, buyToken)
        returns (Fraction[] memory prices)
    {
        prices = new Fraction[](specifiedAmounts.length);

        Fraction memory outputPrice = getPriceAt(poolId, sellToken);

        for (uint256 i = 0; i < specifiedAmounts.length; i++) {
            prices[i] = outputPrice;
        }
    }

    /// @inheritdoc ISwapAdapter
    function swap(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    )
        external
        override
        checkInputPoolIdAndTokens(poolId, sellToken, buyToken)
        returns (Trade memory trade)
    {
        if (specifiedAmount == 0) {
            return trade;
        }

        uint256 gasBefore = gasleft();

        if (side == OrderSide.Sell) {
            trade.calculatedAmount = sell(poolId, sellToken, specifiedAmount);
        } else {
            trade.calculatedAmount = buy(poolId, buyToken, specifiedAmount);
        }

        trade.gasUsed = gasBefore - gasleft();

        trade.price = getPriceAt(poolId, sellToken);
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32 poolId, address sellToken, address buyToken)
        external
        view
        override
        checkInputPoolIdAndTokens(poolId, sellToken, buyToken)
        returns (uint256[] memory limits)
    {
        limits = new uint256[](2);

        // DAI <-> sDAI
        if (poolId == bytes32(bytes20(address(sDai)))) {
            if (sellToken == address(dai)) {
                limits[0] = dai.totalSupply() / RESERVE_FACTOR;
                limits[1] = sDai.previewDeposit(limits[0]);
            } else {
                uint256 totalAssets = sDai.totalAssets();
                limits[0] = sDai.previewWithdraw(totalAssets / RESERVE_FACTOR);
                limits[1] = totalAssets / RESERVE_FACTOR;
            }
            return limits;
        }

        // DAI <-> USDC & USDS <-> USDC
        if (
            poolId == bytes32(bytes20(address(daiLitePSM)))
                || poolId == bytes32(bytes20(address(usdsPsmWrapper)))
        ) {
            if (sellToken == address(usdc)) {
                uint256 daiBalanceLitePSM = dai.balanceOf(address(daiLitePSM));
                limits[0] = daiBalanceLitePSM
                    / (daiLitePSM.to18ConversionFactor() * RESERVE_FACTOR);
                limits[1] = daiBalanceLitePSM / RESERVE_FACTOR;
            } else {
                uint256 usdcBalancePocket = usdc.balanceOf(daiLitePSM.pocket());
                limits[0] = (
                    usdcBalancePocket * daiLitePSM.to18ConversionFactor()
                ) / RESERVE_FACTOR;
                limits[1] = usdcBalancePocket / RESERVE_FACTOR;
            }

            return limits;
        }

        // DAI <-> USDS
        if (poolId == bytes32(bytes20(address(daiUsdsConverter)))) {
            uint256 daiTotalSupply = dai.totalSupply();
            uint256 usdsTotalSupply = usds.totalSupply();

            if (daiTotalSupply <= usdsTotalSupply) {
                limits[0] = daiTotalSupply / RESERVE_FACTOR;
                limits[1] = limits[0];
            } else {
                limits[0] = usdsTotalSupply / RESERVE_FACTOR;
                limits[1] = limits[0];
            }

            return limits;
        }

        // USDS <-> sUSDS
        if (poolId == bytes32(bytes20(address(sUsds)))) {
            uint256 usdsTotalSupply = usds.totalSupply();
            uint256 totalAssets = sUsds.totalAssets();

            if (sellToken == address(usds)) {
                limits[0] = usdsTotalSupply / RESERVE_FACTOR;
                limits[1] = sUsds.previewDeposit(limits[0]);
            } else {
                limits[0] = sUsds.previewRedeem(totalAssets / RESERVE_FACTOR);
                limits[1] = totalAssets / RESERVE_FACTOR;
            }
            return limits;
        }

        // MKR <-> SKY
        if (poolId == bytes32(bytes20(address(mkrSkyConverter)))) {
            if (sellToken == address(mkr)) {
                limits[0] = mkr.totalSupply() / RESERVE_FACTOR;
                limits[1] = limits[0] * MKR_TO_SKY_RATE;
            } else {
                limits[0] = sky.totalSupply() / RESERVE_FACTOR;
                limits[1] = limits[0] / MKR_TO_SKY_RATE;
            }
            return limits;
        }
    }

    /// @inheritdoc ISwapAdapter
    function getCapabilities(bytes32 poolId, address, address)
        external
        view
        override
        returns (Capability[] memory capabilities)
    {

        if (poolId == bytes32(bytes20(address(sDai))) || 
            poolId == bytes32(bytes20(address(daiUsdsConverter))) || 
            poolId == bytes32(bytes20(address(sUsds))) ||
            poolId == bytes32(bytes20(address(mkrSkyConverter)))
        ) {
            capabilities = new Capability[](5);
            capabilities[0] = Capability.SellOrder;
            capabilities[1] = Capability.BuyOrder;
            capabilities[2] = Capability.PriceFunction;
            capabilities[3] = Capability.ConstantPrice;
            capabilities[4] = Capability.HardLimits;
            return capabilities;
        }

        if (poolId == bytes32(bytes20(address(daiLitePSM))) || poolId == bytes32(bytes20(address(usdsPsmWrapper)))) {
            capabilities = new Capability[](6);
            capabilities[0] = Capability.SellOrder;
            capabilities[1] = Capability.BuyOrder;
            capabilities[2] = Capability.PriceFunction;
            capabilities[3] = Capability.ConstantPrice;
            capabilities[4] = Capability.HardLimits;
            capabilities[5] = Capability.FeeOnTransfer;
            return capabilities;
        }

        revert Unavailable("SkySwapAdapter: Unsupported Pool");
    }

    /// @notice Returns all supported tokens for a pool
    function getTokens(bytes32 poolId)
        external
        view
        returns (address[] memory tokens)
    {
        Pair memory pair = pairs[poolId];
        if (pair.token0 == address(0)) {
            revert Unavailable("Sky: Invalid pool ID");
        }

        tokens = new address[](2);
        tokens[0] = pair.token0;
        tokens[1] = pair.token1;
    }

    /// @notice Returns all pool IDs
    function getPoolIds(uint256 offset, uint256 limit)
        external
        view
        returns (bytes32[] memory poolIds)
    {
        bytes32[] memory allPoolIds = new bytes32[](6);
        allPoolIds[0] = bytes32(bytes20(address(sDai)));
        allPoolIds[1] = bytes32(bytes20(address(daiLitePSM)));
        allPoolIds[2] = bytes32(bytes20(address(daiUsdsConverter)));
        allPoolIds[3] = bytes32(bytes20(address(usdsPsmWrapper)));
        allPoolIds[4] = bytes32(bytes20(address(sUsds)));
        allPoolIds[5] = bytes32(bytes20(address(mkrSkyConverter)));

        uint256 length = offset + limit > 6 ? 6 - offset : limit;
        poolIds = new bytes32[](length);
        for (uint256 i = 0; i < length; i++) {
            poolIds[i] = allPoolIds[i + offset];
        }
    }

    /**
     * @notice Executes a sell order on the contract.
     * @param poolId The pool ID.
     * @param sellToken The token being sold.
     * @param specifiedAmount The amount to be traded.
     * @return calculatedAmount The amount of tokens received.
     */
    function sell(bytes32 poolId, address sellToken, uint256 specifiedAmount)
        internal
        returns (uint256 calculatedAmount)
    {
        IERC20(sellToken).safeTransferFrom(
            msg.sender, address(this), specifiedAmount
        );
        // DAI <-> sDAI
        if (poolId == bytes32(bytes20(address(sDai)))) {
            if (address(sellToken) == address(dai)) {
                IERC20(sellToken).safeIncreaseAllowance(
                    address(sDai), specifiedAmount
                );
            }

            return address(sellToken) == address(dai)
                ? sDai.deposit(specifiedAmount, msg.sender)
                : sDai.redeem(specifiedAmount, msg.sender, address(this));
        }
        // DAI <-> USDC
        if (poolId == bytes32(bytes20(address(daiLitePSM)))) {
            IERC20(sellToken).safeIncreaseAllowance(
                address(daiLitePSM), specifiedAmount
            );

            // USDC-DAI
            if (address(sellToken) == address(usdc)) {
                return daiLitePSM.sellGem(msg.sender, specifiedAmount);
            } else {
                uint256 usdcAmount =
                    specifiedAmount / daiLitePSM.to18ConversionFactor();

                if (daiLitePSM.tout() > 0) {
                    uint256 fee =
                        (usdcAmount * daiLitePSM.tout()) / daiLitePSM.WAD();
                    usdcAmount = usdcAmount - fee;
                }

                daiLitePSM.buyGem(msg.sender, usdcAmount);
                return usdcAmount;
            }
        }
        // DAI <-> USDS
        if (poolId == bytes32(bytes20(address(daiUsdsConverter)))) {
            IERC20(sellToken).safeIncreaseAllowance(
                address(daiUsdsConverter), specifiedAmount
            );
            if (address(sellToken) == address(dai)) {
                daiUsdsConverter.daiToUsds(msg.sender, specifiedAmount);
                return specifiedAmount;
            } else {
                daiUsdsConverter.usdsToDai(msg.sender, specifiedAmount);
                return specifiedAmount;
            }
        }
        // USDS <-> USDC
        if (poolId == bytes32(bytes20(address(usdsPsmWrapper)))) {
            IERC20(sellToken).safeIncreaseAllowance(
                address(usdsPsmWrapper), specifiedAmount
            );

            if (address(sellToken) == address(usdc)) {
                return usdsPsmWrapper.sellGem(msg.sender, specifiedAmount);
            } else {
                uint256 usdcAmount =
                    specifiedAmount / usdsPsmWrapper.to18ConversionFactor();
                if (usdsPsmWrapper.tout() > 0) {
                    uint256 fee = (usdcAmount * usdsPsmWrapper.tout())
                        / usdsPsmWrapper.WAD();
                    usdcAmount = usdcAmount - fee;
                }
                usdsPsmWrapper.buyGem(msg.sender, usdcAmount);
                return usdcAmount;
            }
        }
        // USDS <-> sUSDS
        if (poolId == bytes32(bytes20(address(sUsds)))) {
            IERC20(sellToken).safeIncreaseAllowance(
                address(sUsds), specifiedAmount
            );

            return address(sellToken) == address(usds)
                ? sUsds.deposit(specifiedAmount, msg.sender)
                : sUsds.redeem(specifiedAmount, msg.sender, address(this));
        }
        // MKR <-> SKY
        if (poolId == bytes32(bytes20(address(mkrSkyConverter)))) {
            if (address(sellToken) == address(mkr)) {
                mkr.safeIncreaseAllowance(
                    address(mkrSkyConverter), specifiedAmount
                );
                mkrSkyConverter.mkrToSky(msg.sender, specifiedAmount);
                return specifiedAmount * MKR_TO_SKY_RATE;
            } else {
                sky.safeIncreaseAllowance(
                    address(mkrSkyConverter), specifiedAmount
                );
                mkrSkyConverter.skyToMkr(msg.sender, specifiedAmount);
                return specifiedAmount / MKR_TO_SKY_RATE;
            }
        }
    }

    /// @notice Executes a buy order on the contract.
    /// @param poolId The pool ID.
    /// @param sellToken The token being sold.
    /// @param specifiedAmount The amount of buyToken to receive.
    /// @return calculatedAmount The amount of sellToken sold.
    function buy(bytes32 poolId, address sellToken, uint256 specifiedAmount)
        internal
        returns (uint256 calculatedAmount)
    {
        // DAI <-> sDAI
        if (poolId == bytes32(bytes20(address(sDai)))) {
            if (address(sellToken) == address(dai)) {
                uint256 amountIn = sDai.previewMint(specifiedAmount);
                dai.safeTransferFrom(msg.sender, address(this), amountIn);
                dai.safeIncreaseAllowance(address(sDai), amountIn);
                sDai.mint(specifiedAmount, msg.sender);
                return amountIn;
            } else {
                uint256 amountIn = sDai.previewWithdraw(specifiedAmount);
                sDai.safeTransferFrom(msg.sender, address(this), amountIn);
                sDai.withdraw(specifiedAmount, msg.sender, address(this));
                return amountIn;
            }
        }
        // DAI <-> USDC
        if (poolId == bytes32(bytes20(address(daiLitePSM)))) {
            if (address(sellToken) == address(dai)) {
                uint256 amountIn =
                    specifiedAmount * daiLitePSM.to18ConversionFactor();
                if (daiLitePSM.tout() > 0) {
                    uint256 fee =
                        (amountIn * daiLitePSM.tout()) / daiLitePSM.WAD();
                    amountIn += fee;
                }
                dai.safeTransferFrom(msg.sender, address(this), amountIn);
                dai.safeIncreaseAllowance(address(daiLitePSM), amountIn);
                daiLitePSM.buyGem(msg.sender, specifiedAmount);
                return amountIn;
            } else {
                uint256 usdcAmount =
                    specifiedAmount / daiLitePSM.to18ConversionFactor();
                if (daiLitePSM.tin() > 0) {
                    uint256 fee =
                        (usdcAmount * daiLitePSM.tin()) / daiLitePSM.WAD();
                    usdcAmount += fee;
                }
                usdc.safeTransferFrom(msg.sender, address(this), usdcAmount);
                usdc.safeIncreaseAllowance(address(daiLitePSM), usdcAmount);
                daiLitePSM.sellGem(msg.sender, usdcAmount);
                return usdcAmount;
            }
        }
        // DAI <-> USDS
        if (poolId == bytes32(bytes20(address(daiUsdsConverter)))) {
            if (address(sellToken) == address(dai)) {
                dai.safeTransferFrom(msg.sender, address(this), specifiedAmount);
                dai.safeIncreaseAllowance(
                    address(daiUsdsConverter), specifiedAmount
                );
                daiUsdsConverter.daiToUsds(msg.sender, specifiedAmount);
                return specifiedAmount;
            } else {
                usds.safeTransferFrom(
                    msg.sender, address(this), specifiedAmount
                );
                usds.safeIncreaseAllowance(
                    address(daiUsdsConverter), specifiedAmount
                );
                daiUsdsConverter.usdsToDai(msg.sender, specifiedAmount);
                return specifiedAmount;
            }
        }
        // USDS <-> USDC
        if (poolId == bytes32(bytes20(address(usdsPsmWrapper)))) {
            if (address(sellToken) == address(usds)) {
                uint256 amountIn =
                    specifiedAmount * usdsPsmWrapper.to18ConversionFactor();
                if (usdsPsmWrapper.tout() > 0) {
                    uint256 fee = (amountIn * usdsPsmWrapper.tout())
                        / usdsPsmWrapper.WAD();
                    amountIn += fee;
                }
                usds.safeTransferFrom(msg.sender, address(this), amountIn);
                usds.safeIncreaseAllowance(address(usdsPsmWrapper), amountIn);
                usdsPsmWrapper.buyGem(msg.sender, specifiedAmount);
                return amountIn;
            } else {
                uint256 usdcAmount =
                    specifiedAmount / usdsPsmWrapper.to18ConversionFactor();
                if (usdsPsmWrapper.tin() > 0) {
                    uint256 fee = (usdcAmount * usdsPsmWrapper.tin())
                        / usdsPsmWrapper.WAD();
                    usdcAmount += fee;
                }
                usdc.safeTransferFrom(msg.sender, address(this), usdcAmount);
                usdc.safeIncreaseAllowance(address(usdsPsmWrapper), usdcAmount);
                usdsPsmWrapper.sellGem(msg.sender, usdcAmount);
                return usdcAmount;
            }
        }
        // USDS <-> sUSDS
        if (poolId == bytes32(bytes20(address(sUsds)))) {
            if (address(sellToken) == address(sUsds)) {
                uint256 amountIn = sUsds.previewMint(specifiedAmount);
                usds.safeTransferFrom(msg.sender, address(this), amountIn);
                usds.safeIncreaseAllowance(address(sUsds), amountIn);
                sUsds.mint(specifiedAmount, msg.sender);
                return amountIn;
            } else {
                uint256 amountIn = sUsds.previewWithdraw(specifiedAmount);
                IERC20(address(sUsds)).safeTransferFrom(
                    msg.sender, address(this), amountIn
                );
                IERC20(address(sUsds)).safeIncreaseAllowance(
                    address(sUsds), amountIn
                );
                sUsds.withdraw(specifiedAmount, msg.sender, address(this));
                return amountIn;
            }
        }
        // MKR <-> SKY
        if (poolId == bytes32(bytes20(address(mkrSkyConverter)))) {
            if (address(sellToken) == address(mkr)) {
                uint256 amountIn = specifiedAmount / MKR_TO_SKY_RATE;
                mkr.safeTransferFrom(msg.sender, address(this), amountIn);
                mkr.safeIncreaseAllowance(address(mkrSkyConverter), amountIn);
                mkrSkyConverter.mkrToSky(msg.sender, amountIn);
                return amountIn;
            } else {
                uint256 amountIn = specifiedAmount * MKR_TO_SKY_RATE;
                sky.safeTransferFrom(msg.sender, address(this), amountIn);
                sky.safeIncreaseAllowance(address(mkrSkyConverter), amountIn);
                mkrSkyConverter.skyToMkr(msg.sender, amountIn);
                return amountIn;
            }
        }
    }

    function getPriceAt(bytes32 poolId, address sellToken)
        internal
        view
        returns (Fraction memory)
    {
        // DAI <-> sDAI
        if (poolId == bytes32(bytes20(address(sDai)))) {
            if (sellToken == address(dai)) {
                return Fraction(sDai.previewDeposit(PRECISION), PRECISION);
            } else {
                return Fraction(sDai.previewRedeem(PRECISION), PRECISION);
            }
        }
        // DAI <-> USDC
        if (poolId == bytes32(bytes20(address(daiLitePSM)))) {
            if (sellToken == address(usdc)) {
                uint256 daiOutWad =
                    10 ** daiLitePSM.dec() * daiLitePSM.to18ConversionFactor();
                uint256 fee;
                if (daiLitePSM.tin() > 0) {
                    fee = (daiOutWad * daiLitePSM.tin()) / daiLitePSM.WAD();
                    unchecked {
                        daiOutWad -= fee;
                    }
                }
                return Fraction(daiOutWad, PRECISION);
            } else {
                uint256 daiInWad = IERC20Metadata(address(dai)).decimals()
                    * daiLitePSM.to18ConversionFactor();
                uint256 fee;
                if (daiLitePSM.tout() > 0) {
                    fee = (daiInWad * daiLitePSM.tout()) / daiLitePSM.WAD();
                    daiInWad += fee;
                }
                return Fraction(PRECISION, daiInWad);
            }
        }
        // DAI <-> USDS
        if (poolId == bytes32(bytes20(address(daiUsdsConverter)))) {
            return Fraction(PRECISION, PRECISION);
        }
        // USDS <-> USDC
        if (poolId == bytes32(bytes20(address(usdsPsmWrapper)))) {
            if (sellToken == address(usdc)) {
                uint256 usdsOutWad = 10 ** usdsPsmWrapper.dec()
                    * usdsPsmWrapper.to18ConversionFactor();
                uint256 fee;
                if (usdsPsmWrapper.tin() > 0) {
                    fee = (usdsOutWad * usdsPsmWrapper.tin())
                        / usdsPsmWrapper.WAD();
                    unchecked {
                        usdsOutWad -= fee;
                    }
                }
                return Fraction(usdsOutWad, PRECISION);
            } else {
                uint256 usdsInWad = IERC20Metadata(address(usds)).decimals()
                    * usdsPsmWrapper.to18ConversionFactor();
                uint256 fee;
                if (usdsPsmWrapper.tout() > 0) {
                    fee = (usdsInWad * usdsPsmWrapper.tout())
                        / usdsPsmWrapper.WAD();
                    usdsInWad += fee;
                }
                return Fraction(PRECISION, usdsInWad);
            }
        }
        // USDS <-> sUSDS
        if (poolId == bytes32(bytes20(address(sUsds)))) {
            if (sellToken == address(usds)) {
                return Fraction(sUsds.previewDeposit(PRECISION), PRECISION);
            } else {
                return Fraction(sUsds.previewRedeem(PRECISION), PRECISION);
            }
        }
        // MKR <-> SKY
        if (poolId == bytes32(bytes20(address(mkrSkyConverter)))) {
            if (sellToken == address(mkr)) {
                return Fraction(mkrSkyConverter.rate() * PRECISION, PRECISION);
            } else {
                return Fraction(PRECISION, mkrSkyConverter.rate() * PRECISION);
            }
        }
        return Fraction(0, 0);
    }
}

// INTERFACES

///////////////////////////////////// ISavingsDai
// ////////////////////////////////////////////////////////////

interface ISavingsDai is IERC20 {
    function asset() external view returns (address);

    function decimals() external view returns (uint8);

    function maxMint(address) external pure returns (uint256);

    function maxRedeem(address) external view returns (uint256);

    function previewMint(uint256 shares) external view returns (uint256);

    function previewWithdraw(uint256 assets) external view returns (uint256);

    function previewDeposit(uint256 assets) external view returns (uint256);

    function previewRedeem(uint256 shares) external view returns (uint256);

    function totalAssets() external view returns (uint256);

    function totalSupply() external pure returns (uint256);

    function deposit(uint256 assets, address receiver)
        external
        returns (uint256 shares);

    function mint(uint256 shares, address receiver)
        external
        returns (uint256 assets);

    function withdraw(uint256 assets, address receiver, address owner)
        external
        returns (uint256 shares);

    function redeem(uint256 shares, address receiver, address owner)
        external
        returns (uint256 assets);
}

///////////////////////////////////// IDssLitePSM
// ////////////////////////////////////////////////////////////

interface IDssLitePSM {
    /**
     * A lightweight PSM implementation.
     * @notice Swaps Dai for `gem` at a 1:1 exchange rate.
     * @notice Fees `tin` and `tout` might apply.
     * @dev `gem` balance is kept in `pocket` instead of this contract.
     * @dev A few assumptions are made:
     *      1. There are no other urns for the same `ilk`
     *      2. Stability fee is always zero for the `ilk`
     *      3. The `spot` price for gem is always 1 (`10**27`).
     *      4. The `spotter.par` (Dai parity) is always 1 (`10**27`).
     *      5. This contract can freely transfer `gem` on behalf of `pocket`.
     */
    function HALTED() external view returns (uint256);

    function ilk() external view returns (bytes32);

    function gem() external view returns (address);

    function daiJoin() external view returns (address);

    function vat() external view returns (address);

    function dai() external view returns (address);

    function pocket() external view returns (address);

    function to18ConversionFactor() external view returns (uint256);

    function WAD() external view returns (uint256);

    /// @notice Fee for selling gems.
    /// @dev `wad` precision. 1 * WAD means a 100% fee.
    function tin() external view returns (uint256);

    /// @notice Fee for buying gems.
    /// @dev `wad` precision. 1 * WAD means a 100% fee.
    function tout() external view returns (uint256);

    /// @notice Buffer for pre-minted Dai.
    /// @dev `wad` precision.
    function buf() external view returns (uint256);

    /**
     * @notice Function that swaps `gem` into Dai.
     * @dev Reverts if `tin` is set to `HALTED`.
     * @param usr The destination of the bought Dai.
     * @param gemAmt The amount of gem to sell. [`gem` precision].
     * @return daiOutWad The amount of Dai bought.
     */
    function sellGem(address usr, uint256 gemAmt)
        external
        returns (uint256 daiOutWad);

    /**
     * @notice Function that swaps Dai into `gem`.
     * @dev Reverts if `tout` is set to `HALTED`.
     * @param usr The destination of the bought gems.
     * @param gemAmt The amount of gem to buy. [`gem` precision].
     * @return daiInWad The amount of Dai required to sell.
     */
    function buyGem(address usr, uint256 gemAmt)
        external
        returns (uint256 daiInWad);

    /**
     * @notice Returns the number of decimals for `gem`.
     * @return The number of decimals for `gem`.
     */
    function dec() external view returns (uint256);

    /**
     * @notice Returns whether the contract is live or not.
     * @return Whether the contract is live or not.
     */
    function live() external view returns (uint256);
}

///////////////////////////////////// IDaiUsdsConverter
// ////////////////////////////////////////////////////////////

interface IDaiUsdsConverter {
    function dai() external view returns (address);

    function usds() external view returns (address);

    function daiToUsds(address usr, uint256 wad) external;

    function usdsToDai(address usr, uint256 wad) external;
}

///////////////////////////////////// IUsdsPsmWrapper
// ////////////////////////////////////////////////////////////

interface IUsdsPsmWrapper {
    function tin() external view returns (uint256);

    function tout() external view returns (uint256);

    function live() external view returns (uint256);

    function sellGem(address usr, uint256 gemAmt)
        external
        returns (uint256 usdsOutWad);

    function buyGem(address usr, uint256 gemAmt)
        external
        returns (uint256 usdsInWad);

    function WAD() external view returns (uint256);

    function HALTED() external view returns (uint256);

    function dec() external view returns (uint256);

    function to18ConversionFactor() external view returns (uint256);

    function usds() external view returns (address);

    function gem() external view returns (address);

    function psm() external view returns (address);

    function legacyDaiJoin() external view returns (address);

    function usdsJoin() external view returns (address);

    function vat() external view returns (address);

    function ilk() external view returns (bytes32);

    function pocket() external view returns (address);

    function legacyDai() external view returns (address);

    function buf() external view returns (uint256);
}

interface PsmLike {
    function gem() external view returns (address);

    function vat() external view returns (address);

    function daiJoin() external view returns (address);

    function pocket() external view returns (address);

    function tin() external view returns (uint256);

    function tout() external view returns (uint256);

    function buf() external view returns (uint256);

    function sellGem(address, uint256) external returns (uint256);

    function buyGem(address, uint256) external returns (uint256);

    function ilk() external view returns (bytes32);

    function vow() external view returns (address);
}

interface GemLike {
    function decimals() external view returns (uint8);

    function approve(address, uint256) external;

    function transferFrom(address, address, uint256) external;
}

interface DaiJoinLike {
    function dai() external view returns (address);

    function join(address, uint256) external;

    function exit(address, uint256) external;
}

interface UsdsJoinLike {
    function usds() external view returns (address);

    function join(address, uint256) external;

    function exit(address, uint256) external;
}

interface VatLike {
    function hope(address) external;

    function live() external view returns (uint256);
}

///////////////////////////////////// ISUsds
// ////////////////////////////////////////////////////////////

interface ISUsds is IERC4626 {
    // Savings yield
    function chi() external view returns (uint192); // The Rate Accumulator  [ray]

    function rho() external view returns (uint64); // Time of last drip     [unix

    // epoch time]

    function ssr() external view returns (uint256); // The USDS Savings Rate

    // [ray]

    function decimals() external view returns (uint8);

    function asset() external view returns (address);

    function totalAssets() external view returns (uint256);

    function convertToShares(uint256 assets) external view returns (uint256);

    function convertToAssets(uint256 shares) external view returns (uint256);

    function maxDeposit(address) external pure returns (uint256);

    function previewDeposit(uint256 assets) external view returns (uint256);

    function deposit(uint256 assets, address receiver)
        external
        returns (uint256 shares);

    function deposit(uint256 assets, address receiver, uint16 referral)
        external
        returns (uint256 shares);

    function maxMint(address) external pure returns (uint256);

    function previewMint(uint256 shares) external view returns (uint256);

    function mint(uint256 shares, address receiver)
        external
        returns (uint256 assets);

    function mint(uint256 shares, address receiver, uint16 referral)
        external
        returns (uint256 assets);

    function maxWithdraw(address owner) external view returns (uint256);

    function previewWithdraw(uint256 assets) external view returns (uint256);

    function withdraw(uint256 assets, address receiver, address owner)
        external
        returns (uint256 shares);

    function maxRedeem(address owner) external view returns (uint256);

    function previewRedeem(uint256 shares) external view returns (uint256);

    function redeem(uint256 shares, address receiver, address owner)
        external
        returns (uint256 assets);

    function permit(
        address owner,
        address spender,
        uint256 value,
        uint256 deadline,
        bytes memory signature
    ) external;
}

///////////////////////////////////// IMkrSkyConverter
// ////////////////////////////////////////////////////////////

interface IMkrSkyConverter {
    function mkr() external view returns (address);

    function sky() external view returns (address);

    function rate() external view returns (uint256);

    function mkrToSky(address usr, uint256 mkrAmt) external;

    function skyToMkr(address usr, uint256 skyAmt) external;
}
