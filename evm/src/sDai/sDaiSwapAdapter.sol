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

    constructor(address savingsDai_) {
        savingsDai = ISavingsDai(savingsDai_);
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

    function getLimits(bytes32 poolId, address sellToken, address buyToken)
        external
        returns (uint256[] memory limits)
    {
        revert NotImplemented("TemplateSwapAdapter.getLimits");
    }

    function getCapabilities(
        bytes32 poolId,
        address sellToken,
        address buyToken
    ) external returns (Capability[] memory capabilities) {
        revert NotImplemented("TemplateSwapAdapter.getCapabilities");
    }

    function getTokens(bytes32 poolId)
        external
        returns (address[] memory tokens)
    {
        revert NotImplemented("TemplateSwapAdapter.getTokens");
    }

    function getPoolIds(uint256 offset, uint256 limit)
        external
        returns (bytes32[] memory ids)
    {
        revert NotImplemented("TemplateSwapAdapter.getPoolIds");
    }


    ///// TEST FUNCTIONS /////

    function getAssetAddress() external view returns (address) {

        return savingsDai.asset();
    }
}

interface ISavingsDai {


    function asset() external view returns (address);

    function decimals() external view returns (uint8);

}