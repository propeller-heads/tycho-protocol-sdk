// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {ERC20} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import {
    IERC20,
    SafeERC20
} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title KyberSwap Elastic Adapter
contract KyberSwapElasticAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    /// @dev custom limit factor for limits/reserves
    uint256 RESERVE_LIMIT_FACTOR = 10;

    IElasticFactory elasticFactory;
    IPoolOracle poolOracle;

    constructor(address _elasticFactory) {
        elasticFactory = IElasticFactory(_elasticFactory);
        poolOracle = IPoolOracle(elasticFactory.poolOracle());
    }

    function price(
        bytes32 _poolId,
        address _sellToken,
        address _buyToken,
        uint256[] memory _specifiedAmounts
    ) external view override returns (Fraction[] memory _prices) {
        address poolAddress = address(bytes20(_poolId));
        _prices = new Fraction[](_specifiedAmounts.length);
        Fraction memory uniformPrice = getPriceAt(poolAddress, _sellToken, _buyToken);

        for (uint256 i = 0; i < _specifiedAmounts.length; i++) {
            _prices[i] = uniformPrice;
        }
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

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32 poolId, address sellToken, address buyToken)
        external
        returns (uint256[] memory limits)
    {
        address poolAddress = address(bytes20(poolId));
        limits = new uint256[](2);
        limits[0] = IERC20(sellToken).balanceOf(poolAddress) / RESERVE_LIMIT_FACTOR;
        limits[1] = IERC20(buyToken).balanceOf(poolAddress) / RESERVE_LIMIT_FACTOR;
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

    /// @notice Get price for a given pair
    /// @dev Since KyberSwapElastic uses an Oracle, prices are always independant of the amount
    /// @param poolAddress address of the pool to swap in
    /// @param sellToken address of the token to sell
    /// @param buyToken address of the token to buy
    function getPriceAt(address poolAddress, address sellToken, address buyToken) internal view returns (Fraction memory) {
        uint32[] memory secondsAgos = new uint32[](1);
        secondsAgos[0] = 0;
        int56[] memory prices = poolOracle.observeFromPool(poolAddress, secondsAgos);
        if(sellToken == IElasticPool(poolAddress).token0()) {
            return Fraction(
                ERC20(buyToken).decimals(), // 1 token
                uint256(uint56(prices[0]))
            );
        }
        else {
            return Fraction(
                uint256(uint56(prices[0])),
                ERC20(sellToken).decimals() // 1 token
            );
        }
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

interface IElasticPool {
    function token0() external view returns (address);
    function token1() external view returns (address);
}
