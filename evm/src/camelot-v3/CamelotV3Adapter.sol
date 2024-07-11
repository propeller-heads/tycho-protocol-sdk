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

    uint160 constant MIN_SQRT_RATIO = 4295128739;
    uint160 constant MAX_SQRT_RATIO = 1461446703485210103287273052203988822378723970342;

    IQuoter immutable quoter;
    IAlgebraFactory immutable factory;

    constructor (address _quoter) {
        quoter = IQuoter(_quoter);
        factory = IAlgebraFactory(quoter.factory());
    }

    /// @notice Callback used by pool when executing a swap
    /// address(pool), tokenToTransfer, deltaPosition
    /// @dev data structure:
    /// {
    ///     address sender, // address of the user executing swap(initial
    /// msg.sender)
    ///     address pool, // address of the pool
    ///     address tokenToTransfer, // address of token to transfer to the pool
    ///     uint8 deltaPosition // quantity to transfer is deltaQty0(0) or
    /// deltaQty1(1)
    /// }
    function swapCallback(
        int256 deltaQty0,
        int256 deltaQty1,
        bytes calldata data
    ) external {
        (
            address msgSender,
            address pool,
            address tokenToTransfer,
            uint8 deltaPosition
        ) = abi.decode(data, (address, address, address, uint8));
        IERC20(tokenToTransfer).safeTransferFrom(
            msgSender,
            pool,
            deltaPosition == 0 ? uint256(deltaQty0) : uint256(deltaQty1)
        );
    }

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32,
        address sellToken,
        address buyToken,
        uint256[] memory specifiedAmounts
    ) external override returns (Fraction[] memory prices) {
        prices = new Fraction[](specifiedAmounts.length);

        for (uint256 i = 0; i < specifiedAmounts.length; i++) {
            prices[i] = getPriceAt(sellToken, buyToken, specifiedAmounts[i]);
        }
    }

    /// @inheritdoc ISwapAdapter
    function swap(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external override returns (Trade memory trade) {
        if (specifiedAmount == 0) {
            return trade;
        }

        IAlgebraPool pool = IAlgebraPool(address(bytes20(poolId)));
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32 poolId, address sellToken, address buyToken)
        external
        view
        override
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
    ) external override pure returns (Capability[] memory capabilities) {
        revert NotImplemented("TemplateSwapAdapter.getCapabilities");
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(bytes32 poolId)
        external
        view
        override
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
        override
        returns (bytes32[] memory)
    {
        revert NotImplemented("CamelotV3Adapter.getPoolIds");
    }

    /// @notice Get swap price
    /// @param sellToken The token to sell
    /// @param buyToken The token to buy
    /// @param specifiedAmount The amount to Swap
    function getPriceAt(address sellToken, address buyToken, uint256 specifiedAmount) internal returns (Fraction memory) {
        (uint256 amountOut,) = quoter.quoteExactInputSingle(sellToken, buyToken, specifiedAmount, 0);
        return Fraction(
            amountOut,
            specifiedAmount
        );
    }

    /// @notice Execute a sell order on a given pool
    /// @param pool pool to swap in
    /// @param sellToken token to sell
    /// @param buyToken token to buy
    /// @param specifiedAmount amount of sellToken to sell
    /// @return (uint256) buyToken amount received
    function sell(IAlgebraPool pool, address sellToken, address buyToken, uint256 specifiedAmount) internal returns (uint256) {
        bool sellTokenIsToken0 = pool.token0() == sellToken;
        IERC20 buyTokenContract = IERC20(buyToken);

        // callback data for swapCallback called by pool
        bytes memory data = abi.encode(
            msg.sender, address(pool), sellToken, sellTokenIsToken0 ? 0 : 1
        );

        uint160 limitSqrtP =
            sellTokenIsToken0 ? MIN_SQRT_RATIO + 1 : MAX_SQRT_RATIO - 1;
        uint256 balBefore = buyTokenContract.balanceOf(msg.sender);
        pool.swap(
            msg.sender,
            sellTokenIsToken0,
            int256(specifiedAmount),
            limitSqrtP,
            data
        );
        return buyTokenContract.balanceOf(msg.sender) - balBefore;
    }

    /// @notice Execute a buy order on a given pool
    /// @param pool pool to swap in
    /// @param sellToken token to sell
    /// @param buyToken token to buy
    /// @param specifiedAmount amount of buyToken to buy
    /// @return (uint256) sellToken amount spent
    function buy(
        IAlgebraPool pool,
        address sellToken,
        address buyToken,
        uint256 specifiedAmount
    ) internal returns (uint256) {
        bool buyTokenIsToken0 = pool.token0() == buyToken;
        IERC20 sellTokenContract = IERC20(sellToken);

        // callback data for swapCallback called by pool
        bytes memory data = abi.encode(
            msg.sender, address(pool), sellToken, buyTokenIsToken0 ? 1 : 0
        );

        uint160 limitSqrtP =
            buyTokenIsToken0 ? MIN_SQRT_RATIO + 1 : MAX_SQRT_RATIO - 1;
        uint256 balBefore = sellTokenContract.balanceOf(address(this));
        pool.swap(
            msg.sender,
            buyTokenIsToken0,
            -int256(specifiedAmount),
            limitSqrtP,
            data
        );
        return balBefore - sellTokenContract.balanceOf(address(this));
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
