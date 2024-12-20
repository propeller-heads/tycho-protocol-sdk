// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {IERC20, SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

contract NablaPortalSwapAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    address immutable oracleAdapter;
    address immutable guardOracle;
    address immutable weth;

    constructor(address _oracleAdapter, address _guardOracle, address _weth) {
        oracleAdapter = _oracleAdapter;
        guardOracle = _guardOracle;
        weth = _weth;
    }

    function price(
        bytes32 _poolId,
        address _sellToken,
        address _buyToken,
        uint256[] memory _specifiedAmounts
    ) external override returns (Fraction[] memory _prices) {
        _prices = new Fraction[](_specifiedAmounts.length);
        revert NotImplemented("NablaPortalSwapAdapter.price");
    }

    function swap(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external override returns (Trade memory trade) {
        revert NotImplemented("NablaPortalSwapAdapter.swap");
    }

    function getLimits(
        bytes32 poolId,
        address sellToken,
        address buyToken
    ) external view override returns (uint256[] memory limits) {
        revert NotImplemented("NablaPortalSwapAdapter.getLimits");
    }

    function getCapabilities(
        bytes32 /*poolId*/,
        address /*sellToken*/,
        address /*buyToken*/
    ) external pure override returns (Capability[] memory capabilities) {
        capabilities = new Capability[](4);
        capabilities[0] = Capability.SellOrder;
        capabilities[1] = Capability.BuyOrder;
        capabilities[2] = Capability.PriceFunction;
        capabilities[3] = Capability.HardLimits;
    }

    /// @dev Optional
    function getTokens(
        bytes32 /*poolId*/
    ) external pure override returns (address[] memory /*tokens*/) {
        revert NotImplemented("NablaPortalSwapAdapter.getTokens");
    }

    /// @dev Optional
    function getPoolIds(
        uint256 /*offset*/,
        uint256 /*limit*/
    ) external pure override returns (bytes32[] memory /*ids*/) {
        revert NotImplemented("NablaPortalSwapAdapter.getPoolIds");
    }
}
