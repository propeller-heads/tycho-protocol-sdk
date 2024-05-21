// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {IERC20, SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {Math} from "openzeppelin-contracts/contracts/utils/math/Math.sol";
import {SafeCast} from 'openzeppelin-contracts/contracts/utils/math/SafeCast.sol';

/// @title StakeWise Adapter
/// @dev This Adapter supports ETH<->osETH swaps
contract StakeWiseAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    uint256 constant MAX_FEE_PERCENT = 10_000;

    IEthGenesisVault immutable vault;
    IERC20 constant osETH = IERC20(0xf1C9acDc66974dFB6dEcB12aA385b9cD01190E38);
    IOsTokenVaultController immutable osTokenVaultController;

    constructor(address _vault, address _osTokenVaultController) {
        vault = IEthGenesisVault(_vault);
        osTokenVaultController = IOsTokenVaultController(_osTokenVaultController);
    }

    /// @dev Check input tokens
    modifier checkInputTokens(address sellToken, address buyToken) {
        if(sellToken == address(0) && buyToken == address(osETH) || sellToken == address(osETH) && buyToken == address(0)) {   
        }
        else {
            revert Unavailable("This adapter only supports ETH<->osETH swaps");
        }
        _;
    }

    /// @dev enable receive to receive ETH
    receive() external payable {}

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        uint256[] memory specifiedAmounts
    ) external view override checkInputTokens(sellToken, buyToken) returns (Fraction[] memory _prices) {
        _prices = new Fraction[](specifiedAmounts.length);

        for (uint256 i = 0; i < specifiedAmounts.length; i++) {
            _prices[i] = getPriceAt(
                sellToken,
                buyToken,
                specifiedAmounts[i],
                true
            );
        }
    }

    /// @inheritdoc ISwapAdapter
    function swap(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external checkInputTokens(sellToken, buyToken) returns (Trade memory trade) {
        if (specifiedAmount == 0) {
            return trade;
        }

        uint256 gasBefore = gasleft();
        if (side == OrderSide.Sell) {
            // sell
            trade.calculatedAmount = sell(sellToken, specifiedAmount);
        } else {
            // buy
            trade.calculatedAmount = buy(buyToken, specifiedAmount);
        }
        trade.gasUsed = gasBefore - gasleft();
        trade.price = getPriceAt(
            sellToken,
            buyToken,
            10**18 / 100000, // PRECISE_UNIT: Safe amount that does not produce rounding errors or modifies supply
            false
        );
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(
        bytes32,
        address sellToken,
        address buyToken
    ) external view override checkInputTokens(sellToken, buyToken) returns (uint256[] memory limits) {
        limits = new uint256[](2);
        if (sellToken == address(osETH)) {
            limits[1] = vault.withdrawableAssets();
            limits[0] = vault.convertToShares(limits[1]);
        } else {
            limits[0] = osETH.totalSupply() - vault.totalAssets();
            limits[1] = vault.convertToShares(limits[0]);
        }
    }

    function getCapabilities(
        bytes32 poolId,
        address sellToken,
        address buyToken
    ) external returns (Capability[] memory capabilities) {
        revert NotImplemented("TemplateSwapAdapter.getCapabilities");
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(
        bytes32 poolId
    ) external returns (address[] memory tokens) {
        tokens = new address[](2);
        tokens[0] = address(0);
        tokens[1] = address(osETH);
    }

    function getPoolIds(
        uint256 offset,
        uint256 limit
    ) external returns (bytes32[] memory ids) {
        revert NotImplemented("TemplateSwapAdapter.getPoolIds");
    }

    /// @notice Get swap price
    /// @param sellToken token to sell
    /// @param buyToken token to buy
    /// @param amount amount to swap
    /// @param simulateTrade determine if trade should be simulated (used for price function)
    function getPriceAt(
        address sellToken,
        address buyToken,
        uint256 amount,
        bool simulateTrade
    ) internal view returns (Fraction memory) {
        uint256 numerator;
        uint256 amountToSwap = amount;
        if (!simulateTrade) {
            if (sellToken == address(osETH)) {
                // redeem, amount is osETH to spend
                return Fraction(vault.convertToAssets(amountToSwap), amountToSwap);
            } else {
                // mint, amount is ETH to spend
                return Fraction(vault.convertToShares(amountToSwap), amountToSwap);
            }
        }

        /// @dev PRECISE_UNIT: Safe amount that does not produce rounding errors or modifies supply
        amountToSwap = 10**18 / 100000;

        if (sellToken == address(osETH)) {
            // redeem, amount is osETH to spend
            uint256 sharesAfter = vault.totalShares() - amountToSwap;
            uint256 assetsAfter = vault.totalAssets() -
                vault.convertToAssets(amountToSwap);
            uint256 numerator = Math.mulDiv(amount, assetsAfter, sharesAfter);
            return Fraction(numerator, amountToSwap);
        } else {
            // mint, amount is ETH to spend
            uint256 assetsAfter = vault.totalAssets() + amountToSwap;
            uint256 totalSharesBefore = vault.totalShares();
            uint256 mintedShares = vault.convertToShares(amountToSwap);
            uint256 sharesAfter = totalSharesBefore +
                Math.mulDiv(
                    assetsAfter,
                    totalSharesBefore + mintedShares,
                    assetsAfter,
                    Math.Rounding.Ceil
                );
            uint256 numerator = Math.mulDiv(
                assetsAfter,
                sharesAfter,
                assetsAfter,
                Math.Rounding.Ceil
            );
            return Fraction(numerator, amountToSwap);
        }
    }

    /// @notice Executes a sell order on a given pool.
    /// @param sellToken The address of the token being sold.
    /// @param amount The amount to be traded.
    /// @return uint256 The amount of tokens received.
    function sell(
        address sellToken,
        uint256 amount
    ) internal returns (uint256) {
        if (sellToken == address(0)) {
            // ETH->osETH

            (bool sent_, ) = address(vault).call{value: amount}("");
            if(!sent_) { revert Unavailable("Ether transfer failed"); }

            /// @dev The fee is took 2 times (_mintShares on Vault and _mintOsTokenShares on osTokenVaultController), which in this case includes a rounding error(underestimate) of 2 basis points, so we use 2.198x
            uint256 effectiveFee = vault.feePercent();
            effectiveFee = MAX_FEE_PERCENT - (effectiveFee*2198/1000);

            uint256 amountOut = osTokenVaultController.convertToShares(amount) * effectiveFee / MAX_FEE_PERCENT;
            vault.mintOsToken(msg.sender, amountOut, address(0));
            return amountOut;
        } else {
            // osETH->ETH
            osETH.safeTransferFrom(msg.sender, address(this), amount);
            uint256 balBefore = address(this).balance;
            vault.redeemOsToken(amount, address(this), msg.sender);
            uint256 amountOut = address(this).balance - balBefore;

            (bool sent_, ) = address(msg.sender).call{value: amountOut}("");
            if(!sent_) { revert Unavailable("Ether transfer failed"); }

            return amountOut;
        }
    }

    /// @notice Executes a buy order on a given pool.
    /// @param buyToken The address of the token being bought.
    /// @param amountBought The amount of buyToken tokens to buy.
    /// @return uint256 The amount of tokens received.
    function buy(
        address buyToken,
        uint256 amountBought
    ) internal returns (uint256) {
        if (buyToken != address(0)) {
            // ETH->osETH
            uint256 amountIn = osTokenVaultController.convertToAssets(amountBought);

            /// @dev The fee is took 2 times (_mintShares on Vault and _mintOsTokenShares on osTokenVaultController), but in this case includes rounding errors so we overestimate to 2.22x
            uint256 effectiveFee = vault.feePercent();
            effectiveFee = MAX_FEE_PERCENT + (effectiveFee*2220/1000);

            amountIn = vault.convertToAssets(amountIn * effectiveFee / MAX_FEE_PERCENT);
            (bool sent_, ) = address(vault).call{value: amountIn}("");
            if(!sent_) { revert Unavailable("Ether transfer failed"); }

            vault.mintOsToken(msg.sender, amountBought, address(0));
            return amountIn;
        } else {
            // osETH->ETH
            uint256 amountIn = vault.convertToShares(amountBought);
            osETH.safeTransferFrom(msg.sender, address(this), amountIn);

            vault.redeemOsToken(amountIn, address(this), msg.sender);

            (bool sent_, ) = address(msg.sender).call{value: amountBought}("");
            if(!sent_) { revert Unavailable("Ether transfer failed"); }

            return amountIn;
        }
    }
}

interface IEthGenesisVault {
error AccessDenied();
  error InvalidShares();
  error InvalidAssets();
  error ZeroAddress();
  error InsufficientAssets();
  error CapacityExceeded();
  error InvalidCapacity();
  error InvalidSecurityDeposit();
  error InvalidFeeRecipient();
  error InvalidFeePercent();
  error NotHarvested();
  error NotCollateralized();
  error Collateralized();
  error InvalidProof();
  error LowLtv();
  error RedemptionExceeded();
  error InvalidPosition();
  error InvalidLtv();
  error InvalidHealthFactor();
  error InvalidReceivedAssets();
  error InvalidTokenMeta();
  error UpgradeFailed();
  error InvalidValidator();
  error InvalidValidators();
  error WhitelistAlreadyUpdated();
  error DeadlineExpired();
  error PermitInvalidSigner();
  error InvalidValidatorsRegistryRoot();
  error InvalidVault();
  error AlreadyAdded();
  error AlreadyRemoved();
  error InvalidOracles();
  error NotEnoughSignatures();
  error InvalidOracle();
  error TooEarlyUpdate();
  error InvalidAvgRewardPerSecond();
  error InvalidRewardsRoot();
  error HarvestFailed();
  error InvalidRedeemFromLtvPercent();
  error InvalidLiqThresholdPercent();
  error InvalidLiqBonusPercent();
  error InvalidLtvPercent();
  error InvalidCheckpointIndex();
  error InvalidCheckpointValue();
  error MaxOraclesExceeded();
  error ClaimTooEarly();
    function convertToShares(uint256 assets) external view returns (uint256);
    function convertToAssets(uint256 shares) external view returns (uint256);
    function getShares(address account) external view returns (uint256);
    function totalAssets() external view returns (uint256);
    function totalShares() external view returns (uint256);
    function withdrawableAssets() external view returns (uint256);
    function deposit(
        address receiver,
        address referrer
    ) external;
    function redeem(
        uint256 shares,
        address receiver
    ) external;
    function redeemOsToken(
        uint256 osTokenShares,
        address owner,
        address receiver
    ) external;
    function mintOsToken(
        address receiver,
        uint256 osTokenShares,
        address referrer
    ) external returns (uint256);
    function capacity() external view returns (uint256);
    function feePercent() external view returns (uint256);
}

interface IOsTokenVaultController {
  /**
   * @notice Event emitted on minting shares
   * @param vault The address of the Vault
   * @param receiver The address that received the shares
   * @param assets The number of assets collateralized
   * @param shares The number of tokens the owner received
   */
  event Mint(address indexed vault, address indexed receiver, uint256 assets, uint256 shares);

  /**
   * @notice Event emitted on burning shares
   * @param vault The address of the Vault
   * @param owner The address that owns the shares
   * @param assets The total number of assets withdrawn
   * @param shares The total number of shares burned
   */
  event Burn(address indexed vault, address indexed owner, uint256 assets, uint256 shares);

  /**
   * @notice Event emitted on state update
   * @param profitAccrued The profit accrued since the last update
   * @param treasuryShares The number of shares minted for the treasury
   * @param treasuryAssets The number of assets minted for the treasury
   */
  event StateUpdated(uint256 profitAccrued, uint256 treasuryShares, uint256 treasuryAssets);

  /**
   * @notice Event emitted on capacity update
   * @param capacity The amount after which the OsToken stops accepting deposits
   */
  event CapacityUpdated(uint256 capacity);

  /**
   * @notice Event emitted on treasury address update
   * @param treasury The new treasury address
   */
  event TreasuryUpdated(address indexed treasury);

  /**
   * @notice Event emitted on fee percent update
   * @param feePercent The new fee percent
   */
  event FeePercentUpdated(uint16 feePercent);

  /**
   * @notice Event emitted on average reward per second update
   * @param avgRewardPerSecond The new average reward per second
   */
  event AvgRewardPerSecondUpdated(uint256 avgRewardPerSecond);

  /**
   * @notice Event emitted on keeper address update
   * @param keeper The new keeper address
   */
  event KeeperUpdated(address keeper);

  /**
   * @notice The OsToken capacity
   * @return The amount after which the OsToken stops accepting deposits
   */
  function capacity() external view returns (uint256);

  /**
   * @notice The DAO treasury address that receives OsToken fees
   * @return The address of the treasury
   */
  function treasury() external view returns (address);

  /**
   * @notice The fee percent (multiplied by 100)
   * @return The fee percent applied by the OsToken on the rewards
   */
  function feePercent() external view returns (uint64);

  /**
   * @notice The address that can update avgRewardPerSecond
   * @return The address of the keeper contract
   */
  function keeper() external view returns (address);

  /**
   * @notice The average reward per second used to mint OsToken rewards
   * @return The average reward per second earned by the Vaults
   */
  function avgRewardPerSecond() external view returns (uint256);

  /**
   * @notice The fee per share used for calculating the fee for every position
   * @return The cumulative fee per share
   */
  function cumulativeFeePerShare() external view returns (uint256);

  /**
   * @notice The total number of shares controlled by the OsToken
   * @return The total number of shares
   */
  function totalShares() external view returns (uint256);

  /**
   * @notice Total assets controlled by the OsToken
   * @return The total amount of the underlying asset that is "managed" by OsToken
   */
  function totalAssets() external view returns (uint256);

  /**
   * @notice Converts shares to assets
   * @param assets The amount of assets to convert to shares
   * @return shares The amount of shares that the OsToken would exchange for the amount of assets provided
   */
  function convertToShares(uint256 assets) external view returns (uint256 shares);

  /**
   * @notice Converts assets to shares
   * @param shares The amount of shares to convert to assets
   * @return assets The amount of assets that the OsToken would exchange for the amount of shares provided
   */
  function convertToAssets(uint256 shares) external view returns (uint256 assets);

  /**
   * @notice Updates rewards and treasury fee checkpoint for the OsToken
   */
  function updateState() external;

  /**
   * @notice Mint OsToken shares. Can only be called by the registered vault.
   * @param receiver The address that will receive the shares
   * @param shares The amount of shares to mint
   * @return assets The amount of assets minted
   */
  function mintShares(address receiver, uint256 shares) external returns (uint256 assets);

  /**
   * @notice Burn shares for withdrawn assets. Can only be called by the registered vault.
   * @param owner The address that owns the shares
   * @param shares The amount of shares to burn
   * @return assets The amount of assets withdrawn
   */
  function burnShares(address owner, uint256 shares) external returns (uint256 assets);

  /**
   * @notice Update treasury address. Can only be called by the owner.
   * @param _treasury The new treasury address
   */
  function setTreasury(address _treasury) external;

  /**
   * @notice Update capacity. Can only be called by the owner.
   * @param _capacity The amount after which the OsToken stops accepting deposits
   */
  function setCapacity(uint256 _capacity) external;

  /**
   * @notice Update fee percent. Can only be called by the owner. Cannot be larger than 10 000 (100%).
   * @param _feePercent The new fee percent
   */
  function setFeePercent(uint16 _feePercent) external;

  /**
   * @notice Update keeper address. Can only be called by the owner.
   * @param _keeper The new keeper address
   */
  function setKeeper(address _keeper) external;

  /**
   * @notice Updates average reward per second. Can only be called by the keeper.
   * @param _avgRewardPerSecond The new average reward per second
   */
  function setAvgRewardPerSecond(uint256 _avgRewardPerSecond) external;
}
