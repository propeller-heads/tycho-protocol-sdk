// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {
    IERC20,
    SafeERC20
} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title Camelot V3 Adapter
contract CamelotV3Adapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    IQuoter immutable quoter;
    IAlgebraFactory immutable factory;

    constructor (address _quoter) {
        quoter = IQuoter(_quoter);
        factory = IAlgebraFactory(quoter.factory());
    }

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32,
        address sellToken,
        address buyToken,
        uint256[] memory specifiedAmounts
    ) external view override returns (Fraction[] memory prices) {
        prices = new Fraction[](_specifiedAmounts.length);

        for (uint256 i = 0; i < specifiedAmounts.length; i++) {
            prices[i] = getPriceAt(sellToken, buyToken, specifiedAmounts[i]);
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
    function getLimits(bytes32 poolId, address sellToken, address buyToken)
        external
        returns (uint256[] memory limits)
    {
        address poolAddress = address(bytes20(poolId));
        IAlgebraPool pool = IAlgebraPool(poolAddress);
        limits = new uint256[](2);
        limits[0] = IERC20(sellToken).balanceOf(poolAddress);
        limits[1] = IERC20(buyToken).balanceOf(poolAddress);
    }

    function getCapabilities(
        bytes32 poolId,
        address sellToken,
        address buyToken
    ) external returns (Capability[] memory capabilities) {
        revert NotImplemented("TemplateSwapAdapter.getCapabilities");
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(bytes32 poolId)
        external
        view
        returns (address[] memory tokens)
    {
        IAlgebraPool pool = IAlgebraPool(address(bytes20(poolId)));
        tokens = new address[](2);
        tokens[0] = pool.token0();
        tokens[1] = pool.token1();
    }

    function getPoolIds(uint256, uint256)
        external
        pure
        returns (bytes32[] memory)
    {
        revert NotImplemented("CamelotV3Adapter.getPoolIds");
    }

    /// @notice Get swap price
    /// @param sellToken The token to sell
    /// @param buyToken The token to buy
    /// @param specifiedAmount The amount to Swap
    function getPriceAt(address sellToken, address buyToken, uint256 specifiedAmount) internal returns (Fraction memory) {
        uint256 amountOut = quoter.quoteExactInputSingle(sellToken, buyToken, specifiedAmount, 0);
        return Fraction(
            amountOut,
            specifiedAmount
        );
    }
}

interface IQuoter {
    function quoteExactInput(bytes memory path, uint256 amountIn)
        external
        returns (uint256 amountOut, uint16[] memory fees);

    function quoteExactInputSingle(
        address tokenIn,
        address tokenOut,
        uint256 amountIn,
        uint160 limitSqrtPrice
    ) external returns (uint256 amountOut, uint16 fee);

    function quoteExactOutput(bytes memory path, uint256 amountOut)
        external
        returns (uint256 amountIn, uint16[] memory fees);

    function quoteExactOutputSingle(
        address tokenIn,
        address tokenOut,
        uint256 amountOut,
        uint160 limitSqrtPrice
    ) external returns (uint256 amountIn, uint16 fee);

    function factory() external view returns (address);
}

interface IAlgebraFactory {

  function poolDeployer() external view returns (address);

  function farmingAddress() external view returns (address);

  function vaultAddress() external view returns (address);

  function poolByPair(address tokenA, address tokenB) external view returns (address pool);
}

interface IAlgebraPool {
    function token0() external view returns (address);

    function token1() external view returns (address);

    function swap(address recipient, bool zero2one, int256 amountRequired, uint160 limitSqrtPrice, bytes calldata data) external returns (int256 amount0, int256 amount1);
}
