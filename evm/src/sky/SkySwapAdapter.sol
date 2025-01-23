// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {IERC20Metadata} from
    "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {
    IERC20,
    SafeERC20
} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC4626} from "openzeppelin-contracts/contracts/interfaces/IERC4626.sol";


/// @title SkySwapAdapter

contract SkySwapAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;
    using SafeERC20 for ISavingsDai;

    uint256 constant PRECISION = 10 ** 18;
    uint256 constant MKR_TO_SKY_RATE = 24000;

    // DAI <-> sDAI
    ISavingsDai immutable savingsDai; // 0x83F20F44975D03b1b09e64809B757c47f942BEeA
    // DAI <-> USDC
    IDssLitePSM immutable daiLitePSM; // 0xf6e72Db5454dd049d0788e411b06CfAF16853042
    // DAI <-> USDS
    IDaiUsdsConverter immutable daiUsdsConverter; // 0x3225737a9Bbb6473CB4a45b7244ACa2BeFdB276A
    // USDS <-> USDC
    IUsdsPsmWrapper immutable usdsPsmWrapper; // 0xA188EEC8F81263234dA3622A406892F3D630f98c
    // USDS <-> sUSDS
    ISUsds immutable sUsds; // 0xa3931d71877C0E7a3148CB7Eb4463524FEc27fbD
    // MKR <-> SKY
    IMkrSkyConverter immutable mkrSkyConverter; // 0xBDcFCA946b6CDd965f99a839e4435Bcdc1bc470B


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
    ) 
    {
        savingsDai = ISavingsDai(savingsDai_);
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
    }

    // Token pair checks
    function isDaiSDaiPair(address sellToken, address buyToken) internal view returns (bool) {
        return (sellToken == address(dai) && buyToken == address(savingsDai)) ||
               (sellToken == address(savingsDai) && buyToken == address(dai));
    }

    function isDaiUsdcPair(address sellToken, address buyToken) internal view returns (bool) {
        return (sellToken == address(dai) && buyToken == address(usdc)) ||
               (sellToken == address(usdc) && buyToken == address(dai));
    }

    function isDaiUsdsPair(address sellToken, address buyToken) internal view returns (bool) {
        return (sellToken == address(dai) && buyToken == address(usds)) ||
               (sellToken == address(usds) && buyToken == address(dai));
    }

    function isUsdsUsdcPair(address sellToken, address buyToken) internal view returns (bool) {
        return (sellToken == address(usds) && buyToken == address(usdc)) ||
               (sellToken == address(usdc) && buyToken == address(usds));
    }

    function isUsdsSUsdsPair(address sellToken, address buyToken) internal view returns (bool) {
        return (sellToken == address(usds) && buyToken == address(sUsds)) ||
               (sellToken == address(sUsds) && buyToken == address(usds));
    }

    function isMkrSkyPair(address sellToken, address buyToken) internal view returns (bool) {
        return (sellToken == address(mkr) && buyToken == address(sky)) ||
               (sellToken == address(sky) && buyToken == address(mkr));
    }

    /// @dev Check if swap between provided sellToken and buyToken are supported
    /// by this adapter
    modifier checkInputTokens(address sellToken, address buyToken) {
        bool isValidPair = 
            isDaiSDaiPair(sellToken, buyToken) ||
            isDaiUsdcPair(sellToken, buyToken) ||
            isDaiUsdsPair(sellToken, buyToken) ||
            isUsdsUsdcPair(sellToken, buyToken) ||
            isUsdsSUsdsPair(sellToken, buyToken) ||
            isMkrSkyPair(sellToken, buyToken);

        if (!isValidPair) {
            revert Unavailable("Sky: Unsupported token pair");
        }

        _;
    }
    /// @inheritdoc ISwapAdapter
    function price(
        bytes32 _poolId,
        address _sellToken,
        address _buyToken,
        uint256[] memory _specifiedAmounts
    ) external view override returns (Fraction[] memory _prices) {
        revert NotImplemented("SkySwapAdapter.price");
    }

    /// @inheritdoc ISwapAdapter
    function swap(
        bytes32,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external override checkInputTokens(sellToken, buyToken) returns (Trade memory trade) {

        if (specifiedAmount == 0) {
            return trade;
        }

        uint256 gasBefore = gasleft();      
        if (side == OrderSide.Sell) {
            trade.calculatedAmount = sell(sellToken, buyToken, specifiedAmount);
        } else {
            trade.calculatedAmount = buy(sellToken, buyToken, specifiedAmount);
        }

        trade.gasUsed = gasBefore - gasleft();

        trade.price = getPriceAt(sellToken, buyToken);

    }

    /// @notice Executes a sell order on the contract.
    /// @param sellToken The token being sold.
    /// @param buyToken The token being bought.
    /// @param specifiedAmount The amount to be traded.
    /// @return calculatedAmount The amount of tokens received.
    function sell(address sellToken, address buyToken, uint256 specifiedAmount) internal returns (uint256 calculatedAmount) {
        
        IERC20(sellToken).safeTransferFrom(msg.sender, address(this), specifiedAmount);
        
        if (isDaiSDaiPair(sellToken, buyToken)) {

            if (address(sellToken) == address(dai)) {
                IERC20(sellToken).safeIncreaseAllowance(address(savingsDai), specifiedAmount);
            }

            return address(sellToken) == address(dai)
                ? savingsDai.deposit(specifiedAmount, msg.sender)
                    : savingsDai.redeem(specifiedAmount, msg.sender, address(this));

        }  else if (isDaiUsdcPair(sellToken, buyToken)) {
            IERC20(sellToken).safeIncreaseAllowance(address(daiLitePSM), specifiedAmount);

            // USDC-DAI
            if (address(sellToken) == address(usdc)) {
                return daiLitePSM.sellGem(msg.sender, specifiedAmount);

            // DAI-USDC
            } else {
                // Convert DAI (18 decimals) to USDC (6 decimals)
                uint256 usdcAmount = specifiedAmount / daiLitePSM.to18ConversionFactor();
                
                // Calculate fees if any
                if (daiLitePSM.tout() > 0) {
                    uint256 fee = usdcAmount * daiLitePSM.tout() / daiLitePSM.WAD();
                    usdcAmount = usdcAmount - fee;
                }
                
                return daiLitePSM.buyGem(msg.sender, usdcAmount);
            }
        } else if (isDaiUsdsPair(sellToken, buyToken)) {
            IERC20(sellToken).safeIncreaseAllowance(address(daiUsdsConverter), specifiedAmount);

            return address(sellToken) == address(dai)
                ? daiUsdsConverter.daiToUsds(msg.sender, specifiedAmount)
                : daiUsdsConverter.usdsToDai(msg.sender, specifiedAmount);

        } else if (isUsdsUsdcPair(sellToken, buyToken)) {
            IERC20(sellToken).safeIncreaseAllowance(address(usdsPsmWrapper), specifiedAmount);

            if (address(sellToken) == address(usdc)) {
                return usdsPsmWrapper.sellGem(msg.sender, specifiedAmount);
            } else {
                uint256 usdcAmount = specifiedAmount / usdsPsmWrapper.to18ConversionFactor();
                uint256 fee;
                if (usdsPsmWrapper.tin() > 0) {
                    fee = usdcAmount * usdsPsmWrapper.tin() / usdsPsmWrapper.WAD();
                    usdcAmount = usdcAmount - fee;
                }
                return usdsPsmWrapper.buyGem(msg.sender, usdcAmount);
            }
        } else if (isUsdsSUsdsPair(sellToken, buyToken)) {
            IERC20(sellToken).safeIncreaseAllowance(address(sUsds), specifiedAmount);

            return address(sellToken) == address(usds)
                ? sUsds.deposit(specifiedAmount, msg.sender)
                    : sUsds.redeem(specifiedAmount, msg.sender, address(this));

        } else if (isMkrSkyPair(sellToken, buyToken)) {
            IERC20(sellToken).safeIncreaseAllowance(address(mkrSkyConverter), specifiedAmount);

            if (address(sellToken) == address(mkr)) {
                return mkrSkyConverter.mkrToSky(msg.sender, specifiedAmount);
            } else {
                return mkrSkyConverter.skyToMkr(msg.sender, specifiedAmount);
            }
        }

        revert Unavailable("Sky: Unsupported token pair");
    }

    /// @notice Executes a buy order on the contract.
    /// @param buyToken The token being bought.
    /// @param specifiedAmount The amount of buyToken to receive.
    /// @return calculatedAmount The amount of sellToken sold.
    function buy(address sellToken, address buyToken, uint256 specifiedAmount)
        internal
        returns (uint256 calculatedAmount)
    {
        if (isDaiSDaiPair(sellToken, buyToken)) {
            if (address(buyToken) == address(savingsDai)) {
                // DAI->sDAI: Calculate DAI needed for specified sDAI amount
                uint256 amountIn = savingsDai.previewMint(specifiedAmount);
                dai.safeTransferFrom(msg.sender, address(this), amountIn);
                dai.safeIncreaseAllowance(address(savingsDai), amountIn);
                savingsDai.mint(specifiedAmount, msg.sender);
                return amountIn;
            } else {
                // sDAI->DAI: Calculate sDAI needed for specified DAI amount
                uint256 amountIn = savingsDai.previewWithdraw(specifiedAmount);
                savingsDai.safeTransferFrom(msg.sender, address(this), amountIn);
                savingsDai.withdraw(specifiedAmount, msg.sender, address(this));
                return amountIn;
            }
        } else if (isDaiUsdcPair(sellToken, buyToken)) {
            if (address(buyToken) == address(usdc)) {
                // DAI->USDC: Calculate DAI needed for specified USDC amount
                uint256 amountIn = specifiedAmount * daiLitePSM.to18ConversionFactor();
                if (daiLitePSM.tout() > 0) {
                    uint256 fee = amountIn * daiLitePSM.tout() / daiLitePSM.WAD();
                    amountIn += fee;
                }
                dai.safeTransferFrom(msg.sender, address(this), amountIn);
                dai.safeIncreaseAllowance(address(daiLitePSM), amountIn);
                daiLitePSM.buyGem(msg.sender, specifiedAmount);
                return amountIn;
            } else {
                // USDC->DAI: Calculate USDC needed for specified DAI amount
                uint256 usdcAmount = specifiedAmount / daiLitePSM.to18ConversionFactor();
                if (daiLitePSM.tin() > 0) {
                    uint256 fee = usdcAmount * daiLitePSM.tin() / daiLitePSM.WAD();
                    usdcAmount += fee;  // Add fee to input amount
                }
                usdc.safeTransferFrom(msg.sender, address(this), usdcAmount);
                usdc.safeIncreaseAllowance(address(daiLitePSM), usdcAmount);
                uint256 daiOut = daiLitePSM.sellGem(msg.sender, usdcAmount);
                return usdcAmount;
            }
        } else if (isDaiUsdsPair(sellToken, buyToken)) {
            if (address(buyToken) == address(usds)) {
                // DAI->USDS
                dai.safeTransferFrom(msg.sender, address(this), specifiedAmount);
                dai.safeIncreaseAllowance(address(daiUsdsConverter), specifiedAmount);
                daiUsdsConverter.daiToUsds(msg.sender, specifiedAmount);
                return specifiedAmount;
            } else {
                // USDS->DAI
                usds.safeTransferFrom(msg.sender, address(this), specifiedAmount);
                usds.safeIncreaseAllowance(address(daiUsdsConverter), specifiedAmount);
                daiUsdsConverter.usdsToDai(msg.sender, specifiedAmount);
                return specifiedAmount;
            }
        } else if (isUsdsUsdcPair(sellToken, buyToken)) {
            if (address(buyToken) == address(usdc)) {
                // USDS->USDC
                uint256 amountIn = specifiedAmount * usdsPsmWrapper.to18ConversionFactor();
                if (usdsPsmWrapper.tout() > 0) {
                    uint256 fee = amountIn * usdsPsmWrapper.tout() / usdsPsmWrapper.WAD();
                    amountIn += fee;
                }
                usds.safeTransferFrom(msg.sender, address(this), amountIn);
                usds.safeIncreaseAllowance(address(usdsPsmWrapper), amountIn);
                usdsPsmWrapper.buyGem(msg.sender, specifiedAmount);
                return amountIn;
            } else {
                // USDC->USDS
                uint256 amountIn = specifiedAmount / usdsPsmWrapper.to18ConversionFactor();
                usdc.safeTransferFrom(msg.sender, address(this), amountIn);
                usdc.safeIncreaseAllowance(address(usdsPsmWrapper), amountIn);
                usdsPsmWrapper.sellGem(msg.sender, amountIn);
                return amountIn;
            }
        } else if (isUsdsSUsdsPair(sellToken, buyToken)) {
            if (address(buyToken) == address(sUsds)) {
                // USDS->sUSDS
                uint256 amountIn = sUsds.previewMint(specifiedAmount);
                usds.safeTransferFrom(msg.sender, address(this), amountIn);
                usds.safeIncreaseAllowance(address(sUsds), amountIn);
                sUsds.mint(specifiedAmount, msg.sender);
                return amountIn;
            } else {
                // sUSDS->USDS
                uint256 amountIn = sUsds.previewWithdraw(specifiedAmount);
                IERC20(address(sUsds)).safeTransferFrom(msg.sender, address(this), amountIn);
                sUsds.withdraw(specifiedAmount, msg.sender, address(this));
                return amountIn;
            }
        } else if (isMkrSkyPair(sellToken, buyToken)) {
            if (address(buyToken) == address(sky)) {
                // MKR->SKY
                uint256 amountIn = specifiedAmount / MKR_TO_SKY_RATE;
                mkr.safeTransferFrom(msg.sender, address(this), amountIn);
                mkr.safeIncreaseAllowance(address(mkrSkyConverter), amountIn);
                mkrSkyConverter.mkrToSky(msg.sender, amountIn);
                return amountIn;
            } else {
                // SKY->MKR
                uint256 amountIn = specifiedAmount * MKR_TO_SKY_RATE;
                sky.safeTransferFrom(msg.sender, address(this), amountIn);
                sky.safeIncreaseAllowance(address(mkrSkyConverter), amountIn);
                mkrSkyConverter.skyToMkr(msg.sender, amountIn);
                return amountIn;
            }
        }

        revert Unavailable("Sky: Unsupported token pair");
    }

    function getPriceAt(address sellToken, address buyToken) internal view returns (Fraction memory) {
        if (isDaiSDaiPair(sellToken, buyToken)) {
            if (sellToken == address(dai)) {
                return
                    Fraction(savingsDai.previewDeposit(PRECISION), PRECISION);
            } else {
                return
                    Fraction(savingsDai.previewRedeem(PRECISION), PRECISION);
            }
        } else if (isDaiUsdcPair(sellToken, buyToken)) {
            if (sellToken == address(usdc)) {
                // gem is USDC, daiLitePSM.dec() returns gem decimals = 6
                // To get 1 unit of gem (USDC), we need to multiply by 10^6
                // to18ConversionFactor = 10 ** (18 - gem.decimals())
                uint256 daiOutWad = 10**daiLitePSM.dec() * daiLitePSM.to18ConversionFactor();
                uint256 fee;
                if (daiLitePSM.tin() > 0) {
                    fee = daiOutWad * daiLitePSM.tin() / daiLitePSM.WAD();
                    unchecked {
                        daiOutWad -= fee;
                    }
                }
                return Fraction(daiOutWad, PRECISION);
            } else {
                uint256 daiInWad = IERC20Metadata(address(dai)).decimals() * daiLitePSM.to18ConversionFactor();
                uint256 fee;
                if (daiLitePSM.tout() > 0) {
                    fee = daiInWad * daiLitePSM.tout() / daiLitePSM.WAD();
                    daiInWad += fee;
                }
                return Fraction(PRECISION, daiInWad);
            }
        } else if (isDaiUsdsPair(sellToken, buyToken)) {
            return Fraction(PRECISION, PRECISION);

        } else if (isUsdsUsdcPair(sellToken, buyToken)) {
            // PsmLike psm = PsmLike(usdsPsmWrapper.psm());
            if (sellToken == address(usdc)) {
                uint256 usdsOutWad = 10**usdsPsmWrapper.dec() * usdsPsmWrapper.to18ConversionFactor();
                uint256 fee;
                if (usdsPsmWrapper.tin() > 0) {
                    fee = usdsOutWad * usdsPsmWrapper.tin() / usdsPsmWrapper.WAD();
                    unchecked {
                        usdsOutWad -= fee;
                    }
                }
                return Fraction(usdsOutWad, PRECISION);
            } else {
                uint256 usdsInWad = IERC20Metadata(address(usds)).decimals() * usdsPsmWrapper.to18ConversionFactor();
                uint256 fee;
                if (usdsPsmWrapper.tout() > 0) {
                    fee = usdsInWad * usdsPsmWrapper.tout() / usdsPsmWrapper.WAD();
                    usdsInWad += fee;
                }
                return Fraction(PRECISION, usdsInWad);
            }

        } else if (isUsdsSUsdsPair(sellToken, buyToken)) {
            if (sellToken == address(usds)) {
                return
                    Fraction(sUsds.previewDeposit(PRECISION), PRECISION);
            } else {
                return
                    Fraction(sUsds.previewRedeem(PRECISION), PRECISION);
            }

        } else if (isMkrSkyPair(sellToken, buyToken)) {
            if (sellToken == address(mkr)) {
                return
                    Fraction(mkrSkyConverter.rate()*PRECISION, PRECISION);
            } else {
                return
                    Fraction(PRECISION, mkrSkyConverter.rate()*PRECISION);
            }
        }
        return Fraction(0, 0); // Default return for unreachable path
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32, address sellToken, address buyToken)
        external
        view
        override
        returns (uint256[] memory limits)
    {
        limits = new uint256[](2);

        // DAI <-> sDAI
        if (isDaiSDaiPair(sellToken, buyToken)) {

            if (sellToken == address(dai)) {
                limits[0] = 3 * (10 ** 24);
                limits[1] = limits[0];
            } else {
                uint256 totalAssets = savingsDai.totalAssets();
                limits[0] = savingsDai.previewWithdraw(totalAssets);
                limits[1] = totalAssets;
            }
            return limits;
        }

        // DAI <-> USDC
        // DAI is in DssLitePsm
        // USDC is in DssLitePsm.pocket()
        if (isDaiUsdcPair(sellToken, buyToken)) {

            if (sellToken == address(usdc)) {
                // When selling USDC, we need DAI in the PSM to cover it
                // Convert DAI balance to USDC terms (accounting for decimals)
                limits[0] = dai.balanceOf(address(daiLitePSM)) / (daiLitePSM.to18ConversionFactor()*10);
                limits[1] = dai.balanceOf(address(daiLitePSM))/10;
            } else {
                // When selling DAI, we need USDC in the PSM's pocket to cover it
                uint256 usdcBalance = usdc.balanceOf(daiLitePSM.pocket());
                limits[0] = (usdcBalance * daiLitePSM.to18ConversionFactor())/10;  // Convert to DAI decimals
                limits[1] = usdc.balanceOf(daiLitePSM.pocket())/10;
            }


            return limits;
        }

        // DAI <-> USDS
        if (isDaiUsdsPair(sellToken, buyToken)) {

            limits[0] = 3 * (10 ** 24);
            limits[1] = limits[0];

            return limits;
        }

        // USDS <-> USDC 
        if (isUsdsUsdcPair(sellToken, buyToken)) {

            limits[0] = 3 * (10 ** 24);
            limits[1] = limits[0];

            return limits;
        }

        // USDS <-> sUSDS
        if (isUsdsSUsdsPair(sellToken, buyToken)) {

            if (sellToken == address(usds)) {
                limits[0] = 3 * (10 ** 24);
                limits[1] = limits[0];
            } else {
                uint256 totalAssets = sUsds.totalAssets();
                limits[0] = sUsds.previewWithdraw(totalAssets);
                limits[1] = totalAssets;
            }
            return limits;
        }

        // MKR <-> SKY
        if (isMkrSkyPair(sellToken, buyToken)) {

            if (sellToken == address(mkr)) {
                limits[0] = mkr.totalSupply();
                limits[1] = limits[0] * MKR_TO_SKY_RATE;
            } else {
                limits[0] = sky.totalSupply();
                limits[1] = limits[0] / MKR_TO_SKY_RATE;
            }
            return limits;
        }

        revert Unavailable("Sky: Invalid token pair");
    }

    function getCapabilities(
        bytes32 poolId,
        address sellToken,
        address buyToken
    ) external 
      view 
      returns (Capability[] memory capabilities) 
    {
        
        revert NotImplemented("SkySwapAdapter.getCapabilities");
    }

    function getTokens(bytes32 poolId)
        external
        view
        returns (address[] memory tokens)
    {
        revert NotImplemented("SkySwapAdapter.getTokens");
    }

    function getPoolIds(uint256 offset, uint256 limit)
        external
        view
        returns (bytes32[] memory ids)
    {
        revert NotImplemented("SkySwapAdapter.getPoolIds");
    }
}


// INTERFACES

///////////////////////////////////// ISavingsDai ////////////////////////////////////////////////////////////

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

///////////////////////////////////// IDssLitePSM ////////////////////////////////////////////////////////////

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
    function sellGem(address usr, uint256 gemAmt) external returns (uint256 daiOutWad);

    /**
     * @notice Function that swaps Dai into `gem`.
     * @dev Reverts if `tout` is set to `HALTED`.
     * @param usr The destination of the bought gems.
     * @param gemAmt The amount of gem to buy. [`gem` precision].
     * @return daiInWad The amount of Dai required to sell.
     */
    function buyGem(address usr, uint256 gemAmt) external returns (uint256 daiInWad);

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

///////////////////////////////////// IDaiUsdsConverter ////////////////////////////////////////////////////////////

interface IDaiUsdsConverter {
    function dai() external view returns (address);
    function usds() external view returns (address);

    function daiToUsds(address usr, uint256 wad) external returns (uint256 usdsOutWad);
    function usdsToDai(address usr, uint256 wad) external returns (uint256 daiInWad);

}

///////////////////////////////////// IUsdsPsmWrapper ////////////////////////////////////////////////////////////

interface IUsdsPsmWrapper {

    function tin() external view returns (uint256);
    function tout() external view returns (uint256);
    function live() external view returns (uint256);
    function sellGem(address usr, uint256 gemAmt) external returns (uint256 usdsOutWad);
    function buyGem(address usr, uint256 gemAmt) external returns (uint256 usdsInWad);
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

///////////////////////////////////// ISUsds ////////////////////////////////////////////////////////////

interface ISUsds is IERC4626 {

    // Savings yield
    function chi() external view returns (uint192); // The Rate Accumulator  [ray]
    function rho() external view returns (uint64); // Time of last drip     [unix epoch time]
    function ssr() external view returns (uint256); // The USDS Savings Rate [ray]
    function decimals() external view returns (uint8);

    function asset() external view returns (address);

    function totalAssets() external view returns (uint256);

    function convertToShares(uint256 assets) external view returns (uint256);

    function convertToAssets(uint256 shares) external view returns (uint256);

    function maxDeposit(address) external pure returns (uint256);

    function previewDeposit(uint256 assets) external view returns (uint256);

    function deposit(uint256 assets, address receiver) external returns (uint256 shares);

    function deposit(uint256 assets, address receiver, uint16 referral) external returns (uint256 shares);

    function maxMint(address) external pure returns (uint256);

    function previewMint(uint256 shares) external view returns (uint256);

    function mint(uint256 shares, address receiver) external returns (uint256 assets);

    function mint(uint256 shares, address receiver, uint16 referral) external returns (uint256 assets);

    function maxWithdraw(address owner) external view returns (uint256);

    function previewWithdraw(uint256 assets) external view returns (uint256);

    function withdraw(uint256 assets, address receiver, address owner) external returns (uint256 shares);

    function maxRedeem(address owner) external view returns (uint256);

    function previewRedeem(uint256 shares) external view returns (uint256);

    function redeem(uint256 shares, address receiver, address owner) external returns (uint256 assets);

    function permit(address owner, address spender, uint256 value, uint256 deadline, bytes memory signature) external;

}

///////////////////////////////////// IMkrSkyConverter ////////////////////////////////////////////////////////////

interface IMkrSkyConverter {
    function mkr() external view returns (address);
    function sky() external view returns (address);
    function rate() external view returns (uint256);
    function mkrToSky(address usr, uint256 mkrAmt) external returns (uint256 skyAmt);
    function skyToMkr(address usr, uint256 skyAmt) external returns (uint256 mkrAmt);
}