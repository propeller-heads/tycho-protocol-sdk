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
    /// @dev FraxPoolV3 only supports FRAX<=>Token pairs
    /// @dev output amounts in case (buyToken == FRAX) are in TOKEN units(e.g. 10**6)
    function price(
        bytes32 _poolId,
        IERC20 _sellToken,
        IERC20 _buyToken,
        uint256[] memory _specifiedAmounts
    ) external view override returns (Fraction[] memory _prices) {
        _prices = new Fraction[](_specifiedAmounts.length);
        address sellTokenAddress = address(_sellToken);
        address buyTokenAddress = address(_buyToken);

        if(sellTokenAddress != address(FRAX) && buyTokenAddress != address(FRAX)) {
            revert Unavailable("Buy or Sell token should be FRAX");
        }

        for(uint256 i = 0; i < _specifiedAmounts.length; i++) {
            _prices[i] = getPriceAt(_specifiedAmounts[i], sellTokenAddress, buyTokenAddress);
        }
    }

    /// @inheritdoc ISwapAdapter
    function swap(
        bytes32 poolId,
        IERC20 sellToken,
        IERC20 buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external returns (Trade memory trade) {
        if(specifiedAmount == 0) {
            return trade;
        }

        address sellTokenAddress = address(sellToken);
        address buyTokenAddress = address(buyToken);
        address fraxAddress = address(FRAX);

        if(sellTokenAddress != fraxAddress && buyTokenAddress != fraxAddress) {
            revert Unavailable("Buy or Sell token should be FRAX");
        }

        uint256 gasBefore = gasleft();
        if (side == OrderSide.Sell) {
            if(sellTokenAddress == fraxAddress) {
                trade.calculatedAmount =
                    sellFraxForToken(buyTokenAddress, specifiedAmount);
            }
            else {
                trade.calculatedAmount =
                    sellTokenForFrax(sellTokenAddress, specifiedAmount);
            }
        } else {
            if(sellTokenAddress == fraxAddress) {
                trade.calculatedAmount =
                    buyTokenWithFrax(buyTokenAddress, specifiedAmount);
            }
            else {
                trade.calculatedAmount =
                    buyFraxWithToken(sellTokenAddress, specifiedAmount);
            }
        }
        trade.gasUsed = gasBefore - gasleft();
        trade.price = getPriceAt(specifiedAmount, sellTokenAddress, buyTokenAddress);
    }

    /// @inheritdoc ISwapAdapter
    /// @dev Only limit in FraxV3 is the one applied to the collateral(pool_ceiling); Selling FRAX(redeem) has no limits
    function getLimits(bytes32 poolId, IERC20 sellToken, IERC20 buyToken)
        external
        view
        override
        returns (uint256[] memory limits)
    {
        limits = new uint256[](2);
        uint256 collateralID;
        address sellTokenAddress = address(sellToken);

        if(sellTokenAddress == address(FRAX)) {
            collateralID = pool.collateralAddrToIdx(address(buyToken));
            limits[0] = type(uint256).max;
            limits[1] = pool.pool_ceilings(collateralID);
            return limits;
        }

        if(!pool.enabled_collaterals(sellTokenAddress)) {
            revert Unavailable("This sell token is not available");
        }
        collateralID = pool.collateralAddrToIdx(sellTokenAddress);
        limits[0] = pool.pool_ceilings(collateralID);
        limits[1] = type(uint256).max;
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

    /// @notice Buy FRAX with token
    /// @param sellTokenAddress Address of the token to sell
    /// @param receivedAmountFRAX Amount of FRAX tokens in(collateral) to receive
    function buyFraxWithToken(
        address sellTokenAddress,
        uint256 receivedAmountFRAX
    ) internal returns (uint256) {

        if(!pool.enabled_collaterals(sellTokenAddress)) {
            revert Unavailable("The input token is not available as buy method");
        }

        uint256 collateralID = pool.collateralAddrToIdx(sellTokenAddress);

        uint256 fraxAmountWithFeeAdded = getAmountWithFee(receivedAmountFRAX, false, false);
        uint256 collateralNeeded = pool.getFRAXInCollateral(collateralID, fraxAmountWithFeeAdded); // amountIn

        IERC20(sellTokenAddress).safeTransferFrom(msg.sender, address(this), collateralNeeded);
        IERC20(sellTokenAddress).approve(address(pool), collateralNeeded);

        pool.mintFrax(collateralID, fraxAmountWithFeeAdded, receivedAmountFRAX, true);

        return collateralNeeded;

    }

    /// @notice Buy token with FRAX
    /// @param receiveTokenAddress Address of the token to receive
    /// @param receivedAmountTokens Amount of tokens to receive
    function buyTokenWithFrax(
        address receiveTokenAddress,
        uint256 receivedAmountTokens
    ) internal returns (uint256) {

        if(!pool.enabled_collaterals(receiveTokenAddress)) {
            revert Unavailable("The input token is not available as buy method");
        }

        if(FRAX.global_collateral_ratio() < PRICE_PRECISION) {
            revert Unavailable("Cannot sell FRAX while global_collateral_ratio < PRICE_PRECISION");
        }

        uint256 collateralID = pool.collateralAddrToIdx(receiveTokenAddress);

        uint256 fraxToSpendWithoutFee =
            receivedAmountTokens *
            (10**pool.missing_decimals(collateralID)) /
            pool.collateral_prices(collateralID);
        uint256 fraxToSpend = getAmountWithFee(fraxToSpendWithoutFee, false, true);

        FRAX.approve(address(pool), fraxToSpend);
        IERC20(address(FRAX)).safeTransferFrom(msg.sender, address(this), fraxToSpend);

        pool.redeemFrax(collateralID, fraxToSpend, 0, 0);
        pool.collectRedemption(collateralID);

        return fraxToSpend;

    }

    /// @notice Sell(redeem) collateral FRAX token for Token
    /// @param receiveTokenAddress Address of the token to receive (to sell FRAX for)
    /// @param frax_amount Amount of FRAX tokens to spend
    function sellFraxForToken(
        address receiveTokenAddress,
        uint256 frax_amount
    ) internal returns (uint256) {

        if(!pool.enabled_collaterals(receiveTokenAddress)) {
            revert Unavailable("The input token is not available as buy method");
        }

        if(FRAX.global_collateral_ratio() < PRICE_PRECISION) {
            revert Unavailable("Cannot sell FRAX while global_collateral_ratio < PRICE_PRECISION");
        }

        uint256 collateralID = pool.collateralAddrToIdx(receiveTokenAddress);

        FRAX.approve(address(pool), frax_amount);
        IERC20(address(FRAX)).safeTransferFrom(msg.sender, address(this), frax_amount);

        pool.redeemFrax(collateralID, frax_amount, 0, 0);
        (, uint256 amountOut) = pool.collectRedemption(collateralID);

        return amountOut;

    }

    /// @notice Sell collateral(token) for FRAX token
    /// @param sellTokenAddress address of Token to spend
    /// @param collateralNeeded amount of Token to spend
    function sellTokenForFrax(
        address sellTokenAddress,
        uint256 collateralNeeded
    ) internal returns (uint256) {

        if(!pool.enabled_collaterals(sellTokenAddress)) {
            revert Unavailable("The input token is not available as sell method");
        }

        if(FRAX.global_collateral_ratio() < PRICE_PRECISION) {
            revert Unavailable("Cannot sell FRAX while global_collateral_ratio < PRICE_PRECISION");
        }

        uint256 collateralID = pool.collateralAddrToIdx(sellTokenAddress);
        uint256 fraxReceivedAmountWithoutFee = 
            collateralNeeded *
            (10**pool.missing_decimals(collateralID)) /
            pool.collateral_prices(collateralID);

        IERC20(sellTokenAddress).safeTransferFrom(msg.sender, address(this), collateralNeeded);
        IERC20(sellTokenAddress).approve(address(pool), collateralNeeded);

        (uint256 amountOut, ,) = pool.mintFrax(collateralID, fraxReceivedAmountWithoutFee, 0, true);

        return amountOut;

    }

    /// @notice Returns an amount (of tokens), with fee included
    /// @param amount amount of tokens(e.g. FRAX) to get add/remove the fee by
    /// @param isSubtraction determine if output amount should be the amount +(false) or -(true) the fee
    /// @param isRedemption determine if the fee to use is redemption_fee(true) or minting_fee(false)
    /// @return uint256 amount plus or minus the fee
    function getAmountWithFee(uint256 amount, bool isSubtraction, bool isRedemption) internal view returns (uint256) {

        uint256 fee = isRedemption ? FRAX.redemption_fee() : FRAX.minting_fee();
        if(isSubtraction) {
            return amount - (amount * (PRICE_PRECISION - fee) / PRICE_PRECISION);
        }
        else {
            return amount + (amount * (PRICE_PRECISION - fee) / PRICE_PRECISION);
        }

    }

    /// @notice Gets output price in FRAX or TOKENS, including fee
    /// @param specifiedAmount amount in input(to spend)
    /// @param sellTokenAddress token to sell (input)
    /// @param buyTokenAddress token to buy (output)
    /// @return uint256 output amount - fee
    function getPriceAt(
        uint256 specifiedAmount,
        address sellTokenAddress,
        address buyTokenAddress
    ) internal view returns (Fraction memory) {

        uint256 collateralID;
        if(sellTokenAddress == address(FRAX)) {
            if(!pool.enabled_collaterals(buyTokenAddress)) {
                revert Unavailable("This sell token is not available");
            }
            collateralID = pool.collateralAddrToIdx(buyTokenAddress);
            return Fraction(
                getAmountWithFee(
                    pool.getFRAXInCollateral(collateralID, specifiedAmount),
                    true,
                    true
                ),
                1
            );
        }
        else {
            if(!pool.enabled_collaterals(sellTokenAddress)) {
                revert Unavailable("This buy token is not available");
            }
            collateralID = pool.collateralAddrToIdx(sellTokenAddress);
            return Fraction(
                getAmountWithFee(
                    specifiedAmount * pool.collateral_prices(collateralID) / (10**(18 - pool.missing_decimals(collateralID))),
                    true,
                    false
                ),
                1
            );
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

    function getFRAXInCollateral(uint256 col_idx, uint256 frax_amount) external view returns (uint256);

    function allCollaterals() external view returns (address[] memory);

    function pool_ceilings(uint256 poolId) external view returns (uint256);

    function collateral_prices(uint256 col_idx) external view returns (uint256);

    function missing_decimals(uint256 col_idx) external view returns (uint256);

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
