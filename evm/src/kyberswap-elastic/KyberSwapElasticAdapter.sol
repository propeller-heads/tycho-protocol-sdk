// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {
    IERC20,
    SafeERC20
} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title KyberSwap Elastic Adapter
contract KyberSwapElasticAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    IElasticFactory elasticFactory;
    IPoolOracle poolOracle;

    constructor(address _elasticFactory) {
        elasticFactory = IElasticFactory(_elasticFactory);
        poolOracle = elasticFactory.poolOracle();
    }

    function price(
        bytes32 _poolId,
        address _sellToken,
        address _buyToken,
        uint256[] memory _specifiedAmounts
    ) external view override returns (Fraction[] memory _prices) {
        revert NotImplemented("KyberSwapElasticAdapter.price");
    }

    function swap(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external returns (Trade memory trade) {
        revert NotImplemented("KyberSwapElasticAdapter.swap");
    }

    function getLimits(bytes32 poolId, address sellToken, address buyToken)
        external
        returns (uint256[] memory limits)
    {
        revert NotImplemented("KyberSwapElasticAdapter.getLimits");
    }

    function getCapabilities(
        bytes32 poolId,
        address sellToken,
        address buyToken
    ) external returns (Capability[] memory capabilities) {
        revert NotImplemented("KyberSwapElasticAdapter.getCapabilities");
    }

    function getTokens(bytes32 poolId)
        external
        returns (address[] memory tokens)
    {
        revert NotImplemented("KyberSwapElasticAdapter.getTokens");
    }

    function getPoolIds(uint256 offset, uint256 limit)
        external
        returns (bytes32[] memory ids)
    {
        revert NotImplemented("KyberSwapElasticAdapter.getPoolIds");
    }
}

interface IPoolOracle {
  function observeFromPool(
    address pool,
    uint32[] memory secondsAgos
  )
    external view
    returns (int56[] memory tickCumulatives);
}

interface IElasticFactory {
    function poolOracle() external view returns (address);
    
    function getPool(address token0, address token1, uint24 swapFee) external view returns (address);
}
