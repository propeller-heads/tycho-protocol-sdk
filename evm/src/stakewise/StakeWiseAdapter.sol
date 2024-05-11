// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {
    IERC20,
    SafeERC20
} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
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

    function price(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        uint256[] memory _specifiedAmounts
    ) external view override returns (Fraction[] memory _prices) {
        revert NotImplemented("TemplateSwapAdapter.price");
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

    function getLimits(bytes32 poolId, address sellToken, address buyToken)
        external
        returns (uint256[] memory limits)
    {
        revert NotImplemented("TemplateSwapAdapter.getLimits");
    }

    function getCapabilities(bytes32 poolId, address sellToken, address buyToken)
        external
        returns (Capability[] memory capabilities)
    {
        revert NotImplemented("TemplateSwapAdapter.getCapabilities");
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(bytes32 poolId)
        external
        returns (address[] memory tokens)
    {
        tokens = new address[](2);
        tokens[0] = address(0);
        tokens[1] = address(osETH);
    }

    function getPoolIds(uint256 offset, uint256 limit)
        external
        returns (bytes32[] memory ids)
    {
        revert NotImplemented("TemplateSwapAdapter.getPoolIds");
    }
}

interface IEthGenesisVault {
    function convertToShares(uint256 shares) external view returns (uint256);
    function convertToAssets(uint256 assets) external view returns (uint256);
    function getShares(address account) external view returns (uint256);
    function totalAssets() external view returns (uint256);
    function totalShares() external view returns (uint256);
    function withdrawableAssets() external view returns (uint256);
    function capacity() external view returns (uint256);
    function deposit(address receiver, address referrer) external view returns (uint256);
    function redeem(uint256 shares, address receiver) external view returns (uint256);
    function redeemOsToken(uint256 osTokenShares, address owner, address receiver) external view returns (uint256);
}
