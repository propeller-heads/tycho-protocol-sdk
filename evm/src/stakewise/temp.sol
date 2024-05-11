// SPDX-License-Identifier: BUSL-1.1

pragma solidity =0.8.22;

import {Initializable} from '@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol';
import {ReentrancyGuardUpgradeable} from '@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol';
import {Address} from '@openzeppelin/contracts/utils/Address.sol';
import {IEthValidatorsRegistry} from '../../interfaces/IEthValidatorsRegistry.sol';
import {IKeeperRewards} from '../../interfaces/IKeeperRewards.sol';
import {IVaultEthStaking} from '../../interfaces/IVaultEthStaking.sol';
import {Errors} from '../../libraries/Errors.sol';
import {VaultValidators} from './VaultValidators.sol';
import {VaultState} from './VaultState.sol';
import {VaultEnterExit} from './VaultEnterExit.sol';
import {VaultMev} from './VaultMev.sol';

/**
 * @title VaultEthStaking
 * @author StakeWise
 * @notice Defines the Ethereum staking functionality for the Vault
 */
abstract contract VaultEthStaking is
  Initializable,
  ReentrancyGuardUpgradeable,
  VaultState,
  VaultValidators,
  VaultEnterExit,
  VaultMev,
  IVaultEthStaking
{
  uint256 private constant _securityDeposit = 1e9;

  /// @inheritdoc IVaultEthStaking
  function deposit(
    address receiver,
    address referrer
  ) public payable virtual override returns (uint256 shares) {
    return _deposit(receiver, msg.value, referrer);
  }

  /// @inheritdoc IVaultEthStaking
  function updateStateAndDeposit(
    address receiver,
    address referrer,
    IKeeperRewards.HarvestParams calldata harvestParams
  ) public payable virtual override returns (uint256 shares) {
    updateState(harvestParams);
    return deposit(receiver, referrer);
  }

  /**
   * @dev Function for depositing using fallback function
   */
  receive() external payable virtual {
    _deposit(msg.sender, msg.value, address(0));
  }

  /// @inheritdoc IVaultEthStaking
  function receiveFromMevEscrow() external payable override {
    if (msg.sender != mevEscrow()) revert Errors.AccessDenied();
  }

  /// @inheritdoc VaultValidators
  function _registerSingleValidator(bytes calldata validator) internal virtual override {
    bytes calldata publicKey = validator[:48];
    IEthValidatorsRegistry(_validatorsRegistry).deposit{value: _validatorDeposit()}(
      publicKey,
      _withdrawalCredentials(),
      validator[48:144],
      bytes32(validator[144:_validatorLength])
    );

    emit ValidatorRegistered(publicKey);
  }

  /// @inheritdoc VaultValidators
  function _registerMultipleValidators(
    bytes calldata validators,
    uint256[] calldata indexes
  ) internal virtual override returns (bytes32[] memory leaves) {
    // SLOAD to memory
    uint256 currentValIndex = validatorIndex;

    uint256 startIndex;
    uint256 endIndex;
    bytes calldata validator;
    bytes calldata publicKey;
    uint256 validatorsCount = indexes.length;
    leaves = new bytes32[](validatorsCount);
    uint256 validatorDeposit = _validatorDeposit();
    bytes memory withdrawalCreds = _withdrawalCredentials();

    for (uint256 i = 0; i < validatorsCount; i++) {
      unchecked {
        // cannot realistically overflow
        endIndex += _validatorLength;
      }
      validator = validators[startIndex:endIndex];
      leaves[indexes[i]] = keccak256(
        bytes.concat(keccak256(abi.encode(validator, currentValIndex)))
      );
      publicKey = validator[:48];
      // slither-disable-next-line arbitrary-send-eth
      IEthValidatorsRegistry(_validatorsRegistry).deposit{value: validatorDeposit}(
        publicKey,
        withdrawalCreds,
        validator[48:144],
        bytes32(validator[144:_validatorLength])
      );
      startIndex = endIndex;
      unchecked {
        // cannot realistically overflow
        ++currentValIndex;
      }
      emit ValidatorRegistered(publicKey);
    }
  }

  /// @inheritdoc VaultState
  function _vaultAssets() internal view virtual override returns (uint256) {
    return address(this).balance;
  }

  /// @inheritdoc VaultEnterExit
  function _transferVaultAssets(
    address receiver,
    uint256 assets
  ) internal virtual override nonReentrant {
    return Address.sendValue(payable(receiver), assets);
  }

  /// @inheritdoc VaultValidators
  function _validatorDeposit() internal pure override returns (uint256) {
    return 32 ether;
  }

  /**
   * @dev Initializes the VaultEthStaking contract
   */
  function __VaultEthStaking_init() internal onlyInitializing {
    __ReentrancyGuard_init();

    // see https://github.com/OpenZeppelin/openzeppelin-contracts/issues/3706
    if (msg.value < _securityDeposit) revert Errors.InvalidSecurityDeposit();
    _deposit(address(this), msg.value, address(0));
  }

  /**
   * @dev This empty reserved space is put in place to allow future versions to add new
   * variables without shifting down storage in the inheritance chain.
   * See https://docs.openzeppelin.com/contracts/4.x/upgradeable#storage_gaps
   */
  uint256[50] private __gap;
}

// SPDX-License-Identifier: BUSL-1.1

pragma solidity =0.8.22;

import {Math} from '@openzeppelin/contracts/utils/math/Math.sol';
import {SafeCast} from '@openzeppelin/contracts/utils/math/SafeCast.sol';
import {IOsTokenVaultController} from '../../interfaces/IOsTokenVaultController.sol';
import {IOsTokenConfig} from '../../interfaces/IOsTokenConfig.sol';
import {IVaultOsToken} from '../../interfaces/IVaultOsToken.sol';
import {IVaultEnterExit} from '../../interfaces/IVaultEnterExit.sol';
import {Errors} from '../../libraries/Errors.sol';
import {VaultImmutables} from './VaultImmutables.sol';
import {VaultEnterExit} from './VaultEnterExit.sol';
import {VaultState} from './VaultState.sol';

/**
 * @title VaultOsToken
 * @author StakeWise
 * @notice Defines the functionality for minting OsToken
 */
abstract contract VaultOsToken is VaultImmutables, VaultState, VaultEnterExit, IVaultOsToken {
  uint256 private constant _wad = 1e18;
  uint256 private constant _hfLiqThreshold = 1e18;
  uint256 private constant _maxPercent = 10_000; // @dev 100.00 %

  /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
  IOsTokenVaultController private immutable _osTokenVaultController;

  /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
  IOsTokenConfig private immutable _osTokenConfig;

  mapping(address => OsTokenPosition) private _positions;

  /**
   * @dev Constructor
   * @dev Since the immutable variable value is stored in the bytecode,
   *      its value would be shared among all proxies pointing to a given contract instead of each proxy’s storage.
   * @param osTokenVaultController The address of the OsTokenVaultController contract
   * @param osTokenConfig The address of the OsTokenConfig contract
   */
  /// @custom:oz-upgrades-unsafe-allow constructor
  constructor(address osTokenVaultController, address osTokenConfig) {
    _osTokenVaultController = IOsTokenVaultController(osTokenVaultController);
    _osTokenConfig = IOsTokenConfig(osTokenConfig);
  }

  /// @inheritdoc IVaultOsToken
  function osTokenPositions(address user) external view override returns (uint128 shares) {
    OsTokenPosition memory position = _positions[user];
    if (position.shares != 0) _syncPositionFee(position);
    return position.shares;
  }

  /// @inheritdoc IVaultOsToken
  function mintOsToken(
    address receiver,
    uint256 osTokenShares,
    address referrer
  ) external override returns (uint256 assets) {
    _checkCollateralized();
    _checkHarvested();

    // mint osToken shares to the receiver
    assets = _osTokenVaultController.mintShares(receiver, osTokenShares);

    // fetch user position
    OsTokenPosition memory position = _positions[msg.sender];
    if (position.shares != 0) {
      _syncPositionFee(position);
    } else {
      position.cumulativeFeePerShare = SafeCast.toUint128(
        _osTokenVaultController.cumulativeFeePerShare()
      );
    }

    // add minted shares to the position
    position.shares += SafeCast.toUint128(osTokenShares);

    // calculate and validate LTV
    if (
      Math.mulDiv(
        convertToAssets(_balances[msg.sender]),
        _osTokenConfig.ltvPercent(),
        _maxPercent
      ) < _osTokenVaultController.convertToAssets(position.shares)
    ) {
      revert Errors.LowLtv();
    }

    // update state
    _positions[msg.sender] = position;

    // emit event
    emit OsTokenMinted(msg.sender, receiver, assets, osTokenShares, referrer);
  }

  /// @inheritdoc IVaultOsToken
  function burnOsToken(uint128 osTokenShares) external override returns (uint256 assets) {
    // burn osToken shares
    assets = _osTokenVaultController.burnShares(msg.sender, osTokenShares);

    // fetch user position
    OsTokenPosition memory position = _positions[msg.sender];
    if (position.shares == 0) revert Errors.InvalidPosition();
    _syncPositionFee(position);

    // update osToken position
    position.shares -= SafeCast.toUint128(osTokenShares);
    _positions[msg.sender] = position;

    // emit event
    emit OsTokenBurned(msg.sender, assets, osTokenShares);
  }

  /// @inheritdoc IVaultOsToken
  function liquidateOsToken(
    uint256 osTokenShares,
    address owner,
    address receiver
  ) external override {
    (uint256 burnedShares, uint256 receivedAssets) = _redeemOsToken(
      owner,
      receiver,
      osTokenShares,
      true
    );
    emit OsTokenLiquidated(
      msg.sender,
      owner,
      receiver,
      osTokenShares,
      burnedShares,
      receivedAssets
    );
  }

  /// @inheritdoc IVaultOsToken
  function redeemOsToken(uint256 osTokenShares, address owner, address receiver) external override {
    (uint256 burnedShares, uint256 receivedAssets) = _redeemOsToken(
      owner,
      receiver,
      osTokenShares,
      false
    );
    emit OsTokenRedeemed(msg.sender, owner, receiver, osTokenShares, burnedShares, receivedAssets);
  }

  /// @inheritdoc IVaultEnterExit
  function redeem(
    uint256 shares,
    address receiver
  ) public virtual override(IVaultEnterExit, VaultEnterExit) returns (uint256 assets) {
    assets = super.redeem(shares, receiver);
    _checkOsTokenPosition(msg.sender);
  }

  /// @inheritdoc IVaultEnterExit
  function enterExitQueue(
    uint256 shares,
    address receiver
  ) public virtual override(IVaultEnterExit, VaultEnterExit) returns (uint256 positionTicket) {
    positionTicket = super.enterExitQueue(shares, receiver);
    _checkOsTokenPosition(msg.sender);
  }

  /**
   * @dev Internal function for redeeming and liquidating osToken shares
   * @param owner The minter of the osToken shares
   * @param receiver The receiver of the assets
   * @param osTokenShares The amount of osToken shares to redeem or liquidate
   * @param isLiquidation Whether the liquidation or redemption is being performed
   * @return burnedShares The amount of shares burned
   * @return receivedAssets The amount of assets received
   */
  function _redeemOsToken(
    address owner,
    address receiver,
    uint256 osTokenShares,
    bool isLiquidation
  ) private returns (uint256 burnedShares, uint256 receivedAssets) {
    if (receiver == address(0)) revert Errors.ZeroAddress();
    _checkHarvested();

    // update osToken state for gas efficiency
    _osTokenVaultController.updateState();

    // fetch user position
    OsTokenPosition memory position = _positions[owner];
    if (position.shares == 0) revert Errors.InvalidPosition();
    _syncPositionFee(position);

    // SLOAD to memory
    (
      uint256 redeemFromLtvPercent,
      uint256 redeemToLtvPercent,
      uint256 liqThresholdPercent,
      uint256 liqBonusPercent,

    ) = _osTokenConfig.getConfig();

    // calculate received assets
    if (isLiquidation) {
      receivedAssets = Math.mulDiv(
        _osTokenVaultController.convertToAssets(osTokenShares),
        liqBonusPercent,
        _maxPercent
      );
    } else {
      receivedAssets = _osTokenVaultController.convertToAssets(osTokenShares);
    }

    {
      // check whether received assets are valid
      uint256 depositedAssets = convertToAssets(_balances[owner]);
      if (receivedAssets > depositedAssets || receivedAssets > withdrawableAssets()) {
        revert Errors.InvalidReceivedAssets();
      }

      uint256 mintedAssets = _osTokenVaultController.convertToAssets(position.shares);
      if (isLiquidation) {
        // check health factor violation in case of liquidation
        if (
          Math.mulDiv(depositedAssets * _wad, liqThresholdPercent, mintedAssets * _maxPercent) >=
          _hfLiqThreshold
        ) {
          revert Errors.InvalidHealthFactor();
        }
      } else if (
        // check ltv violation in case of redemption
        Math.mulDiv(depositedAssets, redeemFromLtvPercent, _maxPercent) > mintedAssets
      ) {
        revert Errors.InvalidLtv();
      }
    }

    // reduce osToken supply
    _osTokenVaultController.burnShares(msg.sender, osTokenShares);

    // update osToken position
    position.shares -= SafeCast.toUint128(osTokenShares);
    _positions[owner] = position;

    burnedShares = convertToShares(receivedAssets);

    // update total assets
    unchecked {
      _totalAssets -= SafeCast.toUint128(receivedAssets);
    }

    // burn owner shares
    _burnShares(owner, burnedShares);

    // check ltv violation in case of redemption
    if (
      !isLiquidation &&
      Math.mulDiv(convertToAssets(_balances[owner]), redeemToLtvPercent, _maxPercent) >
      _osTokenVaultController.convertToAssets(position.shares)
    ) {
      revert Errors.RedemptionExceeded();
    }

    // transfer assets to the receiver
    _transferVaultAssets(receiver, receivedAssets);
  }

  /**
   * @dev Internal function for syncing the osToken fee
   * @param position The position to sync the fee for
   */
  function _syncPositionFee(OsTokenPosition memory position) private view {
    // fetch current cumulative fee per share
    uint256 cumulativeFeePerShare = _osTokenVaultController.cumulativeFeePerShare();

    // check whether fee is already up to date
    if (cumulativeFeePerShare == position.cumulativeFeePerShare) return;

    // add treasury fee to the position
    position.shares = SafeCast.toUint128(
      Math.mulDiv(position.shares, cumulativeFeePerShare, position.cumulativeFeePerShare)
    );
    position.cumulativeFeePerShare = SafeCast.toUint128(cumulativeFeePerShare);
  }

  /**
   * @notice Internal function for checking position validity. Reverts if it is invalid.
   * @param user The address of the user
   */
  function _checkOsTokenPosition(address user) internal view {
    // fetch user position
    OsTokenPosition memory position = _positions[user];
    if (position.shares == 0) return;

    // check whether vault assets are up to date
    _checkHarvested();

    // sync fee
    _syncPositionFee(position);

    // calculate and validate position LTV
    if (
      Math.mulDiv(convertToAssets(_balances[user]), _osTokenConfig.ltvPercent(), _maxPercent) <
      _osTokenVaultController.convertToAssets(position.shares)
    ) {
      revert Errors.LowLtv();
    }
  }

  /**
   * @dev This empty reserved space is put in place to allow future versions to add new
   * variables without shifting down storage in the inheritance chain.
   * See https://docs.openzeppelin.com/contracts/4.x/upgradeable#storage_gaps
   */
  uint256[50] private __gap;
}

// SPDX-License-Identifier: BUSL-1.1

pragma solidity =0.8.22;

import {Initializable} from '@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol';
import {SafeCast} from '@openzeppelin/contracts/utils/math/SafeCast.sol';
import {Math} from '@openzeppelin/contracts/utils/math/Math.sol';
import {IVaultState} from '../../interfaces/IVaultState.sol';
import {IKeeperRewards} from '../../interfaces/IKeeperRewards.sol';
import {ExitQueue} from '../../libraries/ExitQueue.sol';
import {Errors} from '../../libraries/Errors.sol';
import {VaultImmutables} from './VaultImmutables.sol';
import {VaultFee} from './VaultFee.sol';

/**
 * @title VaultState
 * @author StakeWise
 * @notice Defines Vault's state manipulation
 */
abstract contract VaultState is VaultImmutables, Initializable, VaultFee, IVaultState {
  using ExitQueue for ExitQueue.History;

  uint128 internal _totalShares;
  uint128 internal _totalAssets;

  /// @inheritdoc IVaultState
  uint128 public override queuedShares;
  uint128 internal _unclaimedAssets;

  ExitQueue.History internal _exitQueue;
  mapping(bytes32 => uint256) internal _exitRequests;
  mapping(address => uint256) internal _balances;

  uint256 private _capacity;

  /// @inheritdoc IVaultState
  function totalShares() external view override returns (uint256) {
    return _totalShares;
  }

  /// @inheritdoc IVaultState
  function totalAssets() external view override returns (uint256) {
    return _totalAssets;
  }

  /// @inheritdoc IVaultState
  function getShares(address account) external view override returns (uint256) {
    return _balances[account];
  }

  /// @inheritdoc IVaultState
  function convertToShares(uint256 assets) public view override returns (uint256 shares) {
    return _convertToShares(assets, Math.Rounding.Floor);
  }

  /// @inheritdoc IVaultState
  function convertToAssets(uint256 shares) public view override returns (uint256 assets) {
    uint256 totalShares_ = _totalShares;
    return (totalShares_ == 0) ? shares : Math.mulDiv(shares, _totalAssets, totalShares_);
  }

  /// @inheritdoc IVaultState
  function capacity() public view override returns (uint256) {
    // SLOAD to memory
    uint256 capacity_ = _capacity;

    // if capacity is not set, it is unlimited
    return capacity_ == 0 ? type(uint256).max : capacity_;
  }

  /// @inheritdoc IVaultState
  function withdrawableAssets() public view override returns (uint256) {
    uint256 vaultAssets = _vaultAssets();
    unchecked {
      // calculate assets that are reserved by users who queued for exit
      // cannot overflow as it is capped with underlying asset total supply
      uint256 reservedAssets = convertToAssets(queuedShares) + _unclaimedAssets;
      return vaultAssets > reservedAssets ? vaultAssets - reservedAssets : 0;
    }
  }

  /// @inheritdoc IVaultState
  function isStateUpdateRequired() external view override returns (bool) {
    return IKeeperRewards(_keeper).isHarvestRequired(address(this));
  }

  /// @inheritdoc IVaultState
  function updateState(
    IKeeperRewards.HarvestParams calldata harvestParams
  ) public virtual override {
    // process total assets delta  since last update
    (int256 totalAssetsDelta, bool harvested) = _harvestAssets(harvestParams);

    // process total assets delta if it has changed
    if (totalAssetsDelta != 0) _processTotalAssetsDelta(totalAssetsDelta);

    // update exit queue every time new update is harvested
    if (harvested) _updateExitQueue();
  }

  /**
   * @dev Internal function for processing rewards and penalties
   * @param totalAssetsDelta The number of assets earned or lost
   */
  function _processTotalAssetsDelta(int256 totalAssetsDelta) internal {
    // SLOAD to memory
    uint256 newTotalAssets = _totalAssets;
    if (totalAssetsDelta < 0) {
      // add penalty to total assets
      newTotalAssets -= uint256(-totalAssetsDelta);

      // update state
      _totalAssets = SafeCast.toUint128(newTotalAssets);
      return;
    }

    // convert assets delta as it is positive
    uint256 profitAssets = uint256(totalAssetsDelta);
    newTotalAssets += profitAssets;

    // update state
    _totalAssets = SafeCast.toUint128(newTotalAssets);

    // calculate admin fee recipient assets
    uint256 feeRecipientAssets = Math.mulDiv(profitAssets, feePercent, _maxFeePercent);
    if (feeRecipientAssets == 0) return;

    // SLOAD to memory
    uint256 totalShares_ = _totalShares;

    // calculate fee recipient's shares
    uint256 feeRecipientShares;
    if (totalShares_ == 0) {
      feeRecipientShares = feeRecipientAssets;
    } else {
      unchecked {
        feeRecipientShares = Math.mulDiv(
          feeRecipientAssets,
          totalShares_,
          newTotalAssets - feeRecipientAssets
        );
      }
    }

    // SLOAD to memory
    address _feeRecipient = feeRecipient;
    // mint shares to the fee recipient
    _mintShares(_feeRecipient, feeRecipientShares);
    emit FeeSharesMinted(_feeRecipient, feeRecipientShares, feeRecipientAssets);
  }

  /**
	 * @dev Internal function that must be used to process exit queue
   * @dev Make sure that sufficient time passed between exit queue updates (at least 1 day).
          Currently it's restricted by the keeper's harvest interval
   * @return burnedShares The total amount of burned shares
   */
  function _updateExitQueue() internal virtual returns (uint256 burnedShares) {
    // SLOAD to memory
    uint256 _queuedShares = queuedShares;
    if (_queuedShares == 0) return 0;

    // calculate the amount of assets that can be exited
    uint256 unclaimedAssets = _unclaimedAssets;
    uint256 exitedAssets = Math.min(
      _vaultAssets() - unclaimedAssets,
      convertToAssets(_queuedShares)
    );
    if (exitedAssets == 0) return 0;

    // calculate the amount of shares that can be burned
    burnedShares = convertToShares(exitedAssets);
    if (burnedShares == 0) return 0;

    // update queued shares and unclaimed assets
    queuedShares = SafeCast.toUint128(_queuedShares - burnedShares);
    _unclaimedAssets = SafeCast.toUint128(unclaimedAssets + exitedAssets);

    // push checkpoint so that exited assets could be claimed
    _exitQueue.push(burnedShares, exitedAssets);
    emit CheckpointCreated(burnedShares, exitedAssets);

    // update state
    _totalShares -= SafeCast.toUint128(burnedShares);
    _totalAssets -= SafeCast.toUint128(exitedAssets);
  }

  /**
   * @dev Internal function for minting shares
   * @param owner The address of the owner to mint shares to
   * @param shares The number of shares to mint
   */
  function _mintShares(address owner, uint256 shares) internal virtual {
    // update total shares
    _totalShares += SafeCast.toUint128(shares);

    // mint shares
    unchecked {
      // cannot overflow because the sum of all user
      // balances can't exceed the max uint256 value
      _balances[owner] += shares;
    }
  }

  /**
   * @dev Internal function for burning shares
   * @param owner The address of the owner to burn shares for
   * @param shares The number of shares to burn
   */
  function _burnShares(address owner, uint256 shares) internal virtual {
    // burn shares
    _balances[owner] -= shares;

    // update total shares
    unchecked {
      // cannot underflow because the sum of all shares can't exceed the _totalShares
      _totalShares -= SafeCast.toUint128(shares);
    }
  }

  /**
   * @dev Internal conversion function (from assets to shares) with support for rounding direction.
   */
  function _convertToShares(
    uint256 assets,
    Math.Rounding rounding
  ) internal view returns (uint256 shares) {
    uint256 totalShares_ = _totalShares;
    // Will revert if assets > 0, totalShares > 0 and _totalAssets = 0.
    // That corresponds to a case where any asset would represent an infinite amount of shares.
    return
      (assets == 0 || totalShares_ == 0)
        ? assets
        : Math.mulDiv(assets, totalShares_, _totalAssets, rounding);
  }

  /**
   * @dev Internal function for harvesting Vaults' new assets
   * @return The total assets delta after harvest
   * @return `true` when the rewards were harvested, `false` otherwise
   */
  function _harvestAssets(
    IKeeperRewards.HarvestParams calldata harvestParams
  ) internal virtual returns (int256, bool);

  /**
	 * @dev Internal function for retrieving the total assets stored in the Vault.
          NB! Assets can be forcibly sent to the vault, the returned value must be used with caution
   * @return The total amount of assets stored in the Vault
   */
  function _vaultAssets() internal view virtual returns (uint256);

  /**
   * @dev Initializes the VaultState contract
   * @param capacity_ The amount after which the Vault stops accepting deposits
   */
  function __VaultState_init(uint256 capacity_) internal onlyInitializing {
    if (capacity_ == 0) revert Errors.InvalidCapacity();
    // skip setting capacity if it is unlimited
    if (capacity_ != type(uint256).max) _capacity = capacity_;
  }

  /**
   * @dev This empty reserved space is put in place to allow future versions to add new
   * variables without shifting down storage in the inheritance chain.
   * See https://docs.openzeppelin.com/contracts/4.x/upgradeable#storage_gaps
   */
  uint256[50] private __gap;
}

// SPDX-License-Identifier: BUSL-1.1

pragma solidity =0.8.22;

import {Initializable} from '@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol';
import {SafeCast} from '@openzeppelin/contracts/utils/math/SafeCast.sol';
import {Math} from '@openzeppelin/contracts/utils/math/Math.sol';
import {IKeeperRewards} from '../../interfaces/IKeeperRewards.sol';
import {IVaultEnterExit} from '../../interfaces/IVaultEnterExit.sol';
import {ExitQueue} from '../../libraries/ExitQueue.sol';
import {Errors} from '../../libraries/Errors.sol';
import {VaultImmutables} from './VaultImmutables.sol';
import {VaultState} from './VaultState.sol';

/**
 * @title VaultEnterExit
 * @author StakeWise
 * @notice Defines the functionality for entering and exiting the Vault
 */
abstract contract VaultEnterExit is VaultImmutables, Initializable, VaultState, IVaultEnterExit {
  using ExitQueue for ExitQueue.History;

  /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
  uint256 private immutable _exitingAssetsClaimDelay;

  /**
   * @dev Constructor
   * @dev Since the immutable variable value is stored in the bytecode,
   *      its value would be shared among all proxies pointing to a given contract instead of each proxy’s storage.
   * @param exitingAssetsClaimDelay The minimum delay after which the assets can be claimed after joining the exit queue
   */
  /// @custom:oz-upgrades-unsafe-allow constructor
  constructor(uint256 exitingAssetsClaimDelay) {
    _exitingAssetsClaimDelay = exitingAssetsClaimDelay;
  }

  /// @inheritdoc IVaultEnterExit
  function getExitQueueIndex(uint256 positionTicket) external view override returns (int256) {
    uint256 checkpointIdx = _exitQueue.getCheckpointIndex(positionTicket);
    return checkpointIdx < _exitQueue.checkpoints.length ? int256(checkpointIdx) : -1;
  }

  /// @inheritdoc IVaultEnterExit
  function redeem(
    uint256 shares,
    address receiver
  ) public virtual override returns (uint256 assets) {
    _checkNotCollateralized();
    if (shares == 0) revert Errors.InvalidShares();
    if (receiver == address(0)) revert Errors.ZeroAddress();

    // calculate amount of assets to burn
    assets = convertToAssets(shares);

    // reverts in case there are not enough withdrawable assets
    if (assets > withdrawableAssets()) revert Errors.InsufficientAssets();

    // update total assets
    _totalAssets -= SafeCast.toUint128(assets);

    // burn owner shares
    _burnShares(msg.sender, shares);

    // transfer assets to the receiver
    _transferVaultAssets(receiver, assets);

    emit Redeemed(msg.sender, receiver, assets, shares);
  }

  /// @inheritdoc IVaultEnterExit
  function enterExitQueue(
    uint256 shares,
    address receiver
  ) public virtual override returns (uint256 positionTicket) {
    _checkCollateralized();
    if (shares == 0) revert Errors.InvalidShares();
    if (receiver == address(0)) revert Errors.ZeroAddress();

    // SLOAD to memory
    uint256 _queuedShares = queuedShares;

    // calculate position ticket
    positionTicket = _exitQueue.getLatestTotalTickets() + _queuedShares;

    // add to the exit requests
    _exitRequests[keccak256(abi.encode(receiver, block.timestamp, positionTicket))] = shares;

    // reverts if owner does not have enough shares
    _balances[msg.sender] -= shares;

    unchecked {
      // cannot overflow as it is capped with _totalShares
      queuedShares = SafeCast.toUint128(_queuedShares + shares);
    }

    emit ExitQueueEntered(msg.sender, receiver, positionTicket, shares);
  }

  /// @inheritdoc IVaultEnterExit
  function calculateExitedAssets(
    address receiver,
    uint256 positionTicket,
    uint256 timestamp,
    uint256 exitQueueIndex
  )
    public
    view
    override
    returns (uint256 leftShares, uint256 claimedShares, uint256 claimedAssets)
  {
    uint256 requestedShares = _exitRequests[
      keccak256(abi.encode(receiver, timestamp, positionTicket))
    ];

    // calculate exited shares and assets
    (claimedShares, claimedAssets) = _exitQueue.calculateExitedAssets(
      exitQueueIndex,
      positionTicket,
      requestedShares
    );
    leftShares = requestedShares - claimedShares;
  }

  /// @inheritdoc IVaultEnterExit
  function claimExitedAssets(
    uint256 positionTicket,
    uint256 timestamp,
    uint256 exitQueueIndex
  )
    external
    override
    returns (uint256 newPositionTicket, uint256 claimedShares, uint256 claimedAssets)
  {
    if (block.timestamp < timestamp + _exitingAssetsClaimDelay) revert Errors.ClaimTooEarly();
    bytes32 queueId = keccak256(abi.encode(msg.sender, timestamp, positionTicket));

    // calculate exited shares and assets
    uint256 leftShares;
    (leftShares, claimedShares, claimedAssets) = calculateExitedAssets(
      msg.sender,
      positionTicket,
      timestamp,
      exitQueueIndex
    );
    // nothing to claim
    if (claimedShares == 0) return (positionTicket, claimedShares, claimedAssets);

    // clean up current exit request
    delete _exitRequests[queueId];

    // skip creating new position for the shares rounding error
    if (leftShares > 1) {
      // update user's queue position
      newPositionTicket = positionTicket + claimedShares;
      _exitRequests[keccak256(abi.encode(msg.sender, timestamp, newPositionTicket))] = leftShares;
    }

    // transfer assets to the receiver
    _unclaimedAssets -= SafeCast.toUint128(claimedAssets);
    _transferVaultAssets(msg.sender, claimedAssets);
    emit ExitedAssetsClaimed(msg.sender, positionTicket, newPositionTicket, claimedAssets);
  }

  /**
   * @dev Internal function that must be used to process user deposits
   * @param to The address to mint shares to
   * @param assets The number of assets deposited
   * @param referrer The address of the referrer. Set to zero address if not used.
   * @return shares The total amount of shares minted
   */
  function _deposit(
    address to,
    uint256 assets,
    address referrer
  ) internal virtual returns (uint256 shares) {
    _checkHarvested();
    if (to == address(0)) revert Errors.ZeroAddress();
    if (assets == 0) revert Errors.InvalidAssets();

    uint256 totalAssetsAfter;
    unchecked {
      // cannot overflow as it is capped with underlying asset total supply
      totalAssetsAfter = _totalAssets + assets;
    }
    if (totalAssetsAfter > capacity()) revert Errors.CapacityExceeded();

    // calculate amount of shares to mint
    shares = _convertToShares(assets, Math.Rounding.Ceil);

    // update state
    _totalAssets = SafeCast.toUint128(totalAssetsAfter);
    _mintShares(to, shares);

    emit Deposited(msg.sender, to, assets, shares, referrer);
  }

  /**
   * @dev Internal function for transferring assets from the Vault to the receiver
   * @dev IMPORTANT: because control is transferred to the receiver, care must be
   *    taken to not create reentrancy vulnerabilities. The Vault must follow the checks-effects-interactions pattern:
   *    https://docs.soliditylang.org/en/v0.8.22/security-considerations.html#use-the-checks-effects-interactions-pattern
   * @param receiver The address that will receive the assets
   * @param assets The number of assets to transfer
   */
  function _transferVaultAssets(address receiver, uint256 assets) internal virtual;

  /**
   * @dev This empty reserved space is put in place to allow future versions to add new
   * variables without shifting down storage in the inheritance chain.
   * See https://docs.openzeppelin.com/contracts/4.x/upgradeable#storage_gaps
   */
  uint256[50] private __gap;
}