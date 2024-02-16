// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {IERC20, ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";

/// @title FraxV3 Adapter
/// @dev Adapter for FraxV3 protocol, supports sFRAX<->FRAX and FRAX<->FXBs
contract FraxV3Adapter is ISwapAdapter {

    ISFrax sFrax;
    IFxbAmo fxbAmo;

    constructor(ISFrax _sFrax, IFxbAmo _fxbAmo) {
        sFrax = _sFrax;
        fxbAmo = _fxbAmo;
    }

    function price(
        bytes32 _poolId,
        IERC20 _sellToken,
        IERC20 _buyToken,
        uint256[] memory _specifiedAmounts
    ) external view override returns (Fraction[] memory _prices) {
        revert NotImplemented("TemplateSwapAdapter.price");
    }

    function swap(
        bytes32 poolId,
        IERC20 sellToken,
        IERC20 buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external returns (Trade memory trade) {
        revert NotImplemented("TemplateSwapAdapter.swap");
    }

    function getLimits(bytes32 poolId, IERC20 sellToken, IERC20 buyToken)
        external
        returns (uint256[] memory limits)
    {
        revert NotImplemented("TemplateSwapAdapter.getLimits");
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
    function getTokens(bytes32)
        external
        returns (IERC20[] memory tokens)
    {
        FXBFactory factory = fxbAmo.iFxbFactory();
        uint256 fxbsLength = factory.fxbsLength();

        tokens = IERC20[](2 + fxbsLength);

        for(uint256 i = 0; i < fxbsLength; i++) {
            tokens[i] = IERC20(factory.fxbs(i));
        }

        tokens[tokens.length - 2] = IERC20(address(0));
        tokens[tokens.length - 1] = IERC20(address(sFRAX));
    }

    function getPoolIds(uint256 offset, uint256 limit)
        external
        returns (bytes32[] memory ids)
    {
        revert NotImplemented("TemplateSwapAdapter.getPoolIds");
    }
}

interface ISFrax {

    function previewDeposit(uint256 assets) external view returns (uint256);

    function previewMint(uint256 shares) external view returns (uint256);

    function previewRedeem(uint256 shares) external view returns (uint256);

    function previewWithdraw(uint256 assets) external view returns (uint256);

    function pricePerShare() external view returns (uint256);

    function deposit(uint256 assets, address receiver) external returns (uint256 shares);

    function mint(uint256 shares, address receiver) external returns (uint256 assets);

    function withdraw(
        uint256 assets,
        address receiver,
        address owner
    ) external returns (uint256 shares);

    function redeem(
        uint256 shares,
        address receiver,
        address owner
    ) external returns (uint256 assets);

}

interface IFxbAmo {

    function mintBonds(address _fxb, uint256 _amount) external;

    function redeemBonds(address _fxb, address _recipient, uint256 _amount) external;

    function withdrawFrax(address _recipient, uint256 _amount) external;

    function withdrawBonds(address _fxb, address _recipient, uint256 _amount) external;

    function iFxbFactory() external view returns (FXBFactory);

}

interface FXBFactory {

    function fxbs(uint256 i) external view returns (address);

    function fxbsLength() external view returns (uint256);

}
