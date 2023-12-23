// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {IERC20, ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

address constant FRAX_ADDRESS = 0x853d955aCEf822Db058eb8505911ED77F175b99e;
uint256 constant PRICE_PRECISION = 1e6;

/// @title FraxPoolV3Adapter
/// Adapter of FraxPoolV3, which is used to mint/redeem FRAX token
contract FraxPoolV3Adapter is ISwapAdapter {

    using SafeERC20 for IERC20;

    IFraxPoolV3 immutable pool;
    IFrax immutable FRAX;

    constructor(IFraxPoolV3 _pool) {
        pool = _pool;
        FRAX = IFrax(FRAX_ADDRESS);
    }

    /// @inheritdoc ISwapAdapter
    /// @dev output amounts are in FRAX units(10**6)
    function price(
        bytes32 _poolId,
        IERC20 _sellToken,
        IERC20 _buyToken,
        uint256[] memory _specifiedAmounts
    ) external view override returns (Fraction[] memory _prices) {
        _prices = new Fraction[](_specifiedAmounts.length);
        address sellTokenAddress = address(_sellToken);

        if(!pool.enabled_collaterals(sellTokenAddress) || !pool.enabled_collaterals(address(_buyToken))) {
            revert Unavailable("This sell or buy token is not available");
        }

        uint256 collateralID = pool.collateralAddrToIdx(sellTokenAddress);

        if(sellTokenAddress != address(FRAX)) {
            for(uint256 i = 0; i < _specifiedAmounts.length; i++) {
                _prices[i] = Fraction(pool.getFRAXInCollateral(collateralID, _specifiedAmounts[i]), 1);
            }
        }
        else {
            for(uint256 i = 0; i < _specifiedAmounts.length; i++) {
                _prices[i] = Fraction(pool.collateral_prices(collateralID), 1);
            }
        }
    }

    function swap(
        bytes32 poolId,
        IERC20 sellToken,
        IERC20 buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external returns (Trade memory trade) {
        revert NotImplemented("FraxPoolV3Adapter.swap");
    }

    function getLimits(bytes32 poolId, IERC20 sellToken, IERC20 buyToken)
        external
        returns (uint256[] memory limits)
    {
        revert NotImplemented("FraxPoolV3Adapter.getLimits");
    }

    /// @inheritdoc ISwapAdapter
    function getCapabilities(bytes32, IERC20, IERC20)
        external
        pure
        override
        returns (Capability[] memory capabilities)
    {
        capabilities = new Capability[](3);
        capabilities[0] = Capability.SellOrder;
        capabilities[1] = Capability.BuyOrder;
        capabilities[2] = Capability.PriceFunction;
    }

    /// @inheritdoc ISwapAdapter
    /// @dev Since FraxV3 has only one pool, and the tokens are therefore collaterals, this function returns available tokens addresses.
    /// @return tokens available tokens in the FraxPoolV3 contract
    function getTokens(bytes32 poolId)
        external
        view
        override
        returns (IERC20[] memory tokens)
    {
        address[] memory collateralAddresses = pool.allCollaterals();
        tokens = new IERC20[](collateralAddresses.length);

        for(uint256 i = 0; i < collateralAddresses.length; i++) {
            tokens[i] = IERC20(collateralAddresses[i]);
        }
    }

    /// @inheritdoc ISwapAdapter
    /// @dev Since FraxV3 has only one pool, and the tokens are therefore collaterals, this function returns available collaterals ids.
    /// @return ids IDs of the collaterals available in the FraxPoolV3 contract
    function getPoolIds(uint256 offset, uint256 limit)
        external
        view
        override
        returns (bytes32[] memory ids)
    {
        uint256 endIdx = offset + limit;
        address[] memory allCollaterals = pool.allCollaterals();
        if (endIdx > allCollaterals.length) {
            endIdx = allCollaterals.length;
        }
        ids = new bytes32[](endIdx - offset);
        for(uint256 i = offset; i < ids.length; i++) {
            ids[i] = bytes32(pool.collateralAddrToIdx(allCollaterals[offset + i]));
        }
    }

    /// @notice Buy(Mint) FRAX token
    /// @param sellTokenAddress Address of the token to sell
    /// @param receivedAmountFRAX Amount of FRAX tokens in(collateral) to receive
    function buyFrax(
        address sellTokenAddress,
        uint256 receivedAmountFRAX
    ) internal {

        if(!pool.enabled_collaterals(sellTokenAddress)) {
            revert Unavailable("The input token is not available as buy method");
        }

        uint256 collateralID = pool.collateralAddrToIdx(sellTokenAddress);

        uint256 fraxAmountWithFeeAdded = getAmountWithFee(receivedAmountFRAX, collateralID, true);
        uint256 collateralNeeded = pool.getFRAXInCollateral(collateralID, fraxAmountWithFeeAdded);

        IERC20(sellTokenAddress).safeTransferFrom(msg.sender, address(this), collateralNeeded);
        IERC20(sellTokenAddress).approve(address(pool), collateralNeeded);

        pool.mintFrax(collateralID, fraxAmountWithFeeAdded, receivedAmountFRAX, true);

    }

    /// @notice Sell(redeem) collateral FRAX token
    /// @param receiveToken Address of the token to receive (to sell FRAX for)
    /// @param amount Amount of FRAX tokens to spend
    function sellFrax(
        address receiveToken,
        uint256 amount
    ) internal {

        if(!pool.enabled_collaterals(receiveToken)) {
            revert Unavailable("The input token is not available as buy method");
        }

        if(FRAX.global_collateral_ratio() < PRICE_PRECISION) {
            revert Unavailable("Cannot sell FRAX while global_collateral_ratio < PRICE_PRECISION");
        }

        uint256 collateralID = pool.collateralAddrToIdx(receiveToken);

        FRAX.approve(address(pool), amount);
        IERC20(address(FRAX)).safeTransferFrom(msg.sender, address(this), amount);

        pool.redeemFrax(collateralID, amount, 0, 0);
        pool.collectRedemption(collateralID);

    }

    /// @notice Returns an amount (of tokens), with fee included
    /// @param amount amount of tokens(e.g. FRAX) to get add/remove the fee by
    /// @param collateralID ID of the collateral to use to get fee
    /// @param isSubtraction determine if output amount should be the amount +(false) or -(true) the fee
    /// @return uint256 amount plus or minus the fee
    function getAmountWithFee(uint256 amount, uint256 collateralID, bool isSubtraction) internal view returns (uint256) {

        uint256 fee = FRAX.minting_fee();
        if(isSubtraction) {
            return amount - (amount * fee / PRICE_PRECISION);
        }
        else {
            return amount + (amount * fee / PRICE_PRECISION);
        } 

    }

}

/// @notice Wrapped (simplified) FraxPoolV3 interface
/// @dev as Frax does not use interfaces, copying the whole code would result in a 1000+ lines long file
/// therefore we can use a custom interface to interact with the only methods and entities we need
interface IFraxPoolV3 {

    function mintFrax(
        uint256 col_idx, 
        uint256 frax_amt,
        uint256 frax_out_min,
        bool one_to_one_override
    ) external returns (
        uint256 total_frax_mint, 
        uint256 collat_needed, 
        uint256 fxs_needed
    );

    function redeemFrax(
        uint256 col_idx, 
        uint256 frax_amount, 
        uint256 fxs_out_min, 
        uint256 col_out_min
    ) external returns (
        uint256 collat_out, 
        uint256 fxs_out
    );

    function collectRedemption(uint256 col_idx) external returns (uint256 fxs_amount, uint256 collateral_amount);

    function enabled_collaterals(
        address collateralAddress
    ) external view returns (bool);

    function collateralAddrToIdx(
        address collateralAddress
    ) external view returns (uint256);

    function collateral_prices(
        uint256 collateralID
    ) external view returns (uint256);

    function getFRAXInCollateral(uint256 col_idx, uint256 frax_amount) external view returns (uint256);

    function allCollaterals() external view returns (address[] memory);

}

interface IFrax {
  function COLLATERAL_RATIO_PAUSER() external view returns (bytes32);
  function DEFAULT_ADMIN_ADDRESS() external view returns (address);
  function DEFAULT_ADMIN_ROLE() external view returns (bytes32);
  function addPool(address pool_address ) external;
  function allowance(address owner, address spender ) external view returns (uint256);
  function approve(address spender, uint256 amount ) external returns (bool);
  function balanceOf(address account ) external view returns (uint256);
  function burn(uint256 amount ) external;
  function burnFrom(address account, uint256 amount ) external;
  function collateral_ratio_paused() external view returns (bool);
  function controller_address() external view returns (address);
  function creator_address() external view returns (address);
  function decimals() external view returns (uint8);
  function decreaseAllowance(address spender, uint256 subtractedValue ) external returns (bool);
  function eth_usd_consumer_address() external view returns (address);
  function eth_usd_price() external view returns (uint256);
  function frax_eth_oracle_address() external view returns (address);
  function frax_info() external view returns (uint256, uint256, uint256, uint256, uint256, uint256, uint256, uint256);
  function frax_pools(address ) external view returns (bool);
  function frax_pools_array(uint256 ) external view returns (address);
  function frax_price() external view returns (uint256);
  function frax_step() external view returns (uint256);
  function fxs_address() external view returns (address);
  function fxs_eth_oracle_address() external view returns (address);
  function fxs_price() external view returns (uint256);
  function genesis_supply() external view returns (uint256);
  function getRoleAdmin(bytes32 role ) external view returns (bytes32);
  function getRoleMember(bytes32 role, uint256 index ) external view returns (address);
  function getRoleMemberCount(bytes32 role ) external view returns (uint256);
  function globalCollateralValue() external view returns (uint256);
  function global_collateral_ratio() external view returns (uint256);
  function grantRole(bytes32 role, address account ) external;
  function hasRole(bytes32 role, address account ) external view returns (bool);
  function increaseAllowance(address spender, uint256 addedValue ) external returns (bool);
  function last_call_time() external view returns (uint256);
  function minting_fee() external view returns (uint256);
  function name() external view returns (string memory);
  function owner_address() external view returns (address);
  function pool_burn_from(address b_address, uint256 b_amount ) external;
  function pool_mint(address m_address, uint256 m_amount ) external;
  function price_band() external view returns (uint256);
  function price_target() external view returns (uint256);
  function redemption_fee() external view returns (uint256);
  function refreshCollateralRatio() external;
  function refresh_cooldown() external view returns (uint256);
  function removePool(address pool_address ) external;
  function renounceRole(bytes32 role, address account ) external;
  function revokeRole(bytes32 role, address account ) external;
  function setController(address _controller_address ) external;
  function setETHUSDOracle(address _eth_usd_consumer_address ) external;
  function setFRAXEthOracle(address _frax_oracle_addr, address _weth_address ) external;
  function setFXSAddress(address _fxs_address ) external;
  function setFXSEthOracle(address _fxs_oracle_addr, address _weth_address ) external;
  function setFraxStep(uint256 _new_step ) external;
  function setMintingFee(uint256 min_fee ) external;
  function setOwner(address _owner_address ) external;
  function setPriceBand(uint256 _price_band ) external;
  function setPriceTarget(uint256 _new_price_target ) external;
  function setRedemptionFee(uint256 red_fee ) external;
  function setRefreshCooldown(uint256 _new_cooldown ) external;
  function setTimelock(address new_timelock ) external;
  function symbol() external view returns (string memory);
  function timelock_address() external view returns (address);
  function toggleCollateralRatio() external;
  function totalSupply() external view returns (uint256);
  function transfer(address recipient, uint256 amount ) external returns (bool);
  function transferFrom(address sender, address recipient, uint256 amount ) external returns (bool);
  function weth_address() external view returns (address);
}
