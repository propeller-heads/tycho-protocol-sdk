// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {IERC20, SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {Math} from "openzeppelin-contracts/contracts/utils/Math/Math.sol";

/// @title StakeWise Adapter
/// @dev This Adapter supports ETH<->osETH swaps
contract StakeWiseAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    IEthGenesisVault immutable vault;
    IERC20 constant osETH = IERC20(0xf1C9acDc66974dFB6dEcB12aA385b9cD01190E38);

    constructor(address _vault) {
        vault = IEthGenesisVault(_vault);
    }

    /// @dev enable receive to receive ETH
    receive() external payable {}

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        uint256[] memory _specifiedAmounts
    ) external view override returns (Fraction[] memory _prices) {
        _prices = new Fraction[](_specifiedAmounts.length);

        for (uint256 i = 0; i < _specifiedAmounts.length; i++) {
            _prices[i] = getPriceAt(
                _sellToken,
                _buyToken,
                _specifiedAmounts[i],
                true
            );
        }
    }

    function swap(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external returns (Trade memory trade) {
        revert NotImplemented("TemplateSwapAdapter.swap");
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(
        bytes32,
        address sellToken,
        address buyToken
    ) external returns (uint256[] memory limits) {
        if (sellToken == address(osETH)) {
            limits[0] = vault.convertToShares(vault.withdrawableAssets());
            limits[1] = vault.withdrawableAssets();
        } else {
            limits[0] = vault.capacity() - vault.totalAssets();
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
        if (sellToken == address(osETH)) {
            
            return Fraction(vault.convertToAssets(amount), amount);
        } else {
            return Fraction(vault.convertToShares(amount), amount);
        }
    }
}

interface IEthGenesisVault {
    function convertToShares(uint256 shares) external view returns (uint256);
    function convertToAssets(uint256 assets) external view returns (uint256);
    function getShares(address account) external view returns (uint256);
    function totalAssets() external view returns (uint256);
    function totalShares() external view returns (uint256);
    function withdrawableAssets() external view returns (uint256);
    function deposit(
        address receiver,
        address referrer
    ) external view returns (uint256);
    function redeem(
        uint256 shares,
        address receiver
    ) external view returns (uint256);
    function redeemOsToken(
        uint256 osTokenShares,
        address owner,
        address receiver
    ) external view returns (uint256);
    function capacity() external view returns (uint256);
}
