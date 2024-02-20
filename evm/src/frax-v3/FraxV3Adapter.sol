// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {IERC20, ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {ERC20} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";

/// @title FraxV3Adapter
/// @dev Adapter for FraxV3 protocol, supports Frax --> sFrax and sFrax --> Frax
contract FraxV3Adapter is ISwapAdapter {

    ISFrax sFrax;
    IERC20 frax;

    constructor(ISFrax _sFrax) {
        sFrax = _sFrax;
        frax = IERC20(address(sFrax.asset()));
    }

    function price(
        bytes32 _poolId,
        IERC20 _sellToken,
        IERC20 _buyToken,
        uint256[] memory _specifiedAmounts
    ) external view override returns (Fraction[] memory _prices) {
        _prices = new Fraction[](_specifiedAmounts.length);
        if (address(_sellToken) == address(sFRAX) && address(_buyToken) == address(frax)) {

        }

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

    /// @inheritdoc ISwapAdapter
    /// @dev there is no hard capped limit 
    function getLimits(bytes32, IERC20 sellToken, IERC20 buyToken)
        external
        returns (uint256[] memory limits)
    {
        limits = new uint256[](2);

        if(address(sellToken) == address(frax)) { // Frax --> sFrax
            limits[0] = frax.totalSupply();
            limits[1] = sFrax.previewDeposit(limits[0]);
        } else {
            limits[0] = sFrax.totalSupply(); 
            limits[1] = sFrax.previewRedeem(limits[0]);
        }
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
        tokens = new IERC20[](2);

        tokens[0] = frax;
        tokens[1] = IERC20(address(sFRAX));
    }

    function getPoolIds(uint256, uint256)
        external
        returns (bytes32[] memory)
    {
        revert NotImplemented("FraxV3Adapter.getPoolIds");
    }


    /// @notice Get FRAX or SFRAX price
    /// @param sellToken token to sell(frax or sfrax)
    /// @param amountOut the amount of buyToken to buy
    /// @return amountIn of sellToken to spend
    function getAmountInForSfrax(address sellToken, uint256 amountOut) internal view returns (uint256) {

        if(sellToken == address(frax)) { // FRAX-SFRAX
            return sFrax.previewMint(amountOut);
        }
        else { // SFRAX-FRAX
            return sFrax.previewWithdraw(amountOut);
        }

    }

    /// @notice Get FRAX or SFRAX price
    /// @param sellToken token to sell(frax or sfrax)
    /// @param amountIn the amount sellToken to spend
    /// @return amountOut of buyToken to buy(received)
    function getAmountOutForSfrax(address sellToken, uint256 amountIn) internal view returns (uint256) {

        if(sellToken == address(frax)) { // FRAX-SFRAX
            return sFrax.previewDeposit(amountIn);
        }
        else { // SFRAX-FRAX
            return sFrax.previewRedeem(amountIn);
        }

    }

}

interface ISFrax {

    function previewDeposit(uint256 assets) external view returns (uint256);

    function previewMint(uint256 shares) external view returns (uint256);

    function previewRedeem(uint256 shares) external view returns (uint256);

    function previewWithdraw(uint256 assets) external view returns (uint256);

    function pricePerShare() external view returns (uint256);

    function asset() external view returns (ERC20); // FRAX

    function totalSupply() external view returns (uint256);

    function totalAssets() public view virtual returns (uint256);

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
