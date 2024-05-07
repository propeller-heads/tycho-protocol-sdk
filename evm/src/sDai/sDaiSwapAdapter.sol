// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {IERC20Metadata} from
    "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {
    IERC20,
    SafeERC20
} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title sDaiSwapAdapter

contract sDaiSwapAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    ISavingsDai immutable savingsDai;
    IDai immutable dai;

    constructor(address savingsDai_) {
        savingsDai = ISavingsDai(savingsDai_);
        dai = IDai(savingsDai.asset());
    }

    function price(
        bytes32 _poolId,
        address _sellToken,
        address _buyToken,
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

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32, address sellToken, address buyToken)
        external
        view
        override
        returns (uint256[] memory limits)
    {
        limits = new uint256[](2);
        /// @dev Limits are underestimated to 90% of totalSupply
        
        if (sellToken == savingsDai.asset()) {
            limits[0] = dai.totalSupply() * 90/100;
            limits[1] = savingsDai.previewDeposit(limits[0]);
        } else {
            limits[0] = savingsDai.totalSupply() * 90/100;
            limits[1] = savingsDai.previewRedeem(limits[0]);
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
    function getTokens(bytes32)
        external
        view
        override
        returns (address[] memory tokens)
    {
        tokens = new address[](2);
        tokens[0] = savingsDai.asset();
        tokens[1] = address(savingsDai);
    }

    /// @inheritdoc ISwapAdapter
    function getPoolIds(uint256, uint256)
        external
        view
        override
        returns (bytes32[] memory ids)
    {
        ids = new bytes32[](1);
        ids[0] = bytes20(address(savingsDai));
    }


    ///// TEST FUNCTIONS /////

    function getAssetAddress() external view returns (address) {

        return savingsDai.asset();
    }
}

interface ISavingsDai {


    function asset() external view returns (address);

    function decimals() external view returns (uint8);

    function maxMint(address) external pure returns (uint256);

    function maxRedeem(address) external view returns (uint256);

    function previewDeposit(uint256 assets) external view returns (uint256);

    function previewRedeem(uint256 shares) external view returns (uint256);

    function totalSupply() external pure returns (uint256);

}

interface IDai {

    function totalSupply() external pure returns (uint256);

}