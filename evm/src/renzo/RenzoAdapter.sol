// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {IERC20, ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {SafeERC20} from
    "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title Renzo Protocol Adapter
contract RenzoAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    IRestakeManager immutable restakeManager;
    IRenzoOracle immutable renzoOracle;
    IERC20 immutable ezETH;

    constructor(address _restakeManager) {
        restakeManager = IRestakeManager(_restakeManager);
        renzoOracle = restakeManager.renzoOracle();
        ezETH = IERC20(address(restakeManager.ezETH()));
    }

    /// @dev check if buyToken is supported(only ezETH is available as buyToken); the restakeManager reverts internally if sellToken is not a supported collateral
    modifier checkBuyToken(address _buyToken) {
        if(_buyToken != address(ezETH)) {
            revert Unavailable("This adapter only supports token -> ezETH swaps");
        }
        _;
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

    function getCapabilities(bytes32 poolId, IERC20 sellToken, IERC20 buyToken)
        external
        returns (Capability[] memory capabilities)
    {
        revert NotImplemented("TemplateSwapAdapter.getCapabilities");
    }

    function getTokens(bytes32 poolId)
        external
        returns (IERC20[] memory tokens)
    {
        revert NotImplemented("TemplateSwapAdapter.getTokens");
    }

    function getPoolIds(uint256 offset, uint256 limit)
        external
        returns (bytes32[] memory ids)
    {
        revert NotImplemented("TemplateSwapAdapter.getPoolIds");
    }

}

interface IEzEthToken {}

interface IRestakeManager {

    function renzoOracle() external view returns (IRenzoOracle);

    function deposit(
        IERC20 _collateralToken,
        uint256 _amount
    ) external;

    function ezETH() external view returns (IEzEthToken);

}

interface IRenzoOracle {        
    function lookupTokenValue(IERC20 _token, uint256 _balance) external view returns (uint256);
    function lookupTokenAmountFromValue(IERC20 _token, uint256 _value) external view returns (uint256);
    function lookupTokenValues(IERC20[] memory _tokens, uint256[] memory _balances) external view returns (uint256);
    function calculateMintAmount(uint256 _currentValueInProtocol, uint256 _newValueAdded, uint256 _existingEzETHSupply) external pure returns (uint256);
    function calculateRedeemAmount(uint256 _ezETHBeingBurned, uint256 _existingEzETHSupply, uint256 _currentValueInProtocol) external pure returns (uint256) ;
}
