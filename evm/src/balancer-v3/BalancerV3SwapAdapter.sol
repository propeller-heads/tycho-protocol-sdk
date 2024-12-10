// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.26;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {IERC20, SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC4626} from "openzeppelin-contracts/contracts/interfaces/IERC4626.sol";

/**
 * @title Balancer V3 Swap Adapter
 */
contract BalancerV3SwapAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    // Balancer V3 constants
    uint256 constant RESERVE_LIMIT_FACTOR = 3; // 0.3 as being divided by 10
    uint256 constant SWAP_DEADLINE_SEC = 1000;

    // Balancer V3 contracts
    IVault immutable vault;
    IBatchRouter immutable router;

    // ETH and Wrapped ETH addresses, using ETH as address(0)
    address constant WETH_ADDRESS = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
    address constant ETH_ADDRESS = address(0);

    constructor(address payable vault_, address _router) {
        vault = IVault(vault_);
        router = IBatchRouter(_router);
    }

    /// @dev Enable ETH receiving
    receive() external payable {}

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32 _poolId,
        address _sellToken,
        address _buyToken,
        uint256[] memory _specifiedAmounts
    ) external override returns (Fraction[] memory _prices) {
        address pool = address(bytes20(_poolId));

        _prices = new Fraction[](_specifiedAmounts.length);

        IERC20 sellToken = IERC20(_sellToken);
        IERC20 buyToken = IERC20(_buyToken);

        for (uint256 i = 0; i < _specifiedAmounts.length; i++) {
            _prices[i] = getPriceAt(
                pool,
                sellToken,
                buyToken,
                _specifiedAmounts[i]
            );
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

        address pool = address(bytes20(poolId));

        // perform swap
        if (side == OrderSide.Sell) {
            trade.calculatedAmount = sellERC20(
                pool,
                IERC20(sellToken),
                IERC20(buyToken),
                specifiedAmount
            );
        } else {
            trade.calculatedAmount = buyERC20(
                pool,
                IERC20(sellToken),
                IERC20(buyToken),
                specifiedAmount
            );
        }

        uint256 gasBefore = gasleft();
        trade.gasUsed = gasBefore - gasleft();
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(
        bytes32 poolId,
        address sellToken,
        address buyToken
    ) external view override returns (uint256[] memory limits) {
        limits = new uint256[](2);
        address pool = address(bytes20(poolId));
        (IERC20 sellTokenERC, IERC20 buyTokenERC) = (
            IERC20(sellToken),
            IERC20(buyToken)
        );

        (IERC20[] memory tokens, , uint256[] memory balancesRaw, ) = vault
            .getPoolTokenInfo(pool);

        for (uint256 i = 0; i < tokens.length; i++) {
            if (tokens[i] == sellTokenERC) {
                limits[0] = (balancesRaw[i] * RESERVE_LIMIT_FACTOR) / 10;
            }
            if (tokens[i] == buyTokenERC) {
                limits[1] = (balancesRaw[i] * RESERVE_LIMIT_FACTOR) / 10;
            }
        }
    }

    /// @inheritdoc ISwapAdapter
    function getCapabilities(
        bytes32,
        address,
        address
    ) external pure override returns (Capability[] memory capabilities) {
        capabilities = new Capability[](4);
        capabilities[0] = Capability.SellOrder;
        capabilities[1] = Capability.BuyOrder;
        capabilities[2] = Capability.PriceFunction;
        capabilities[3] = Capability.HardLimits;
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(
        bytes32 poolId
    ) external view override returns (address[] memory tokens) {
        address poolAddress = address(bytes20(poolId));
        IERC20[] memory tokens_ = vault.getPoolTokens(poolAddress);
        tokens = new address[](tokens_.length);

        for (uint256 i = 0; i < tokens_.length; i++) {
            tokens[i] = address(tokens_[i]);
        }
    }

    function getPoolIds(
        uint256,
        uint256
    ) external pure override returns (bytes32[] memory) {
        revert NotImplemented("BalancerV3SwapAdapter.getPoolIds");
    }

    /**
     * @dev Returns the price of the swap
     * @dev The price is not scaled by the token decimals
     * @param pool The ID of the trading pool.
     * @param sellToken The token being sold.
     * @param buyToken The token being bought.
     * @param specifiedAmount The amount to be traded.
     */
    function getPriceAt(
        address pool,
        IERC20 sellToken,
        IERC20 buyToken,
        uint256 specifiedAmount
    ) internal returns (Fraction memory calculatedPrice) {
        calculatedPrice = Fraction(
            getAmountOut(pool, sellToken, buyToken, specifiedAmount),
            specifiedAmount
        );
    }

    /**
     * @dev Returns the amount of sellToken tokens to spend to get 'specifiedAmount' buyToken tokens
     * @param pool The address of the pool to trade in.
     * @param sellToken The token being sold.
     * @param buyToken The token being bought.
     * @param specifiedAmount The amount to be traded.
     * @return amountIn The amount of tokens to spend.
     */
    function getAmountIn(
        address pool,
        IERC20 sellToken,
        IERC20 buyToken,
        uint256 specifiedAmount
    ) internal returns (uint256 amountIn) {
        bytes memory userData; // empty bytes

        IBatchRouter.SwapPathStep memory step = IBatchRouter.SwapPathStep({
            pool: pool,
            tokenOut: buyToken,
            isBuffer: false
        });
        IBatchRouter.SwapPathStep[]
            memory steps = new IBatchRouter.SwapPathStep[](1);
        steps[0] = step;

        IBatchRouter.SwapPathExactAmountOut memory path = IBatchRouter
            .SwapPathExactAmountOut({
                tokenIn: sellToken,
                steps: steps,
                maxAmountIn: type(uint256).max,
                exactAmountOut: specifiedAmount
            });

        IBatchRouter.SwapPathExactAmountOut[]
            memory paths = new IBatchRouter.SwapPathExactAmountOut[](1);
        paths[0] = path;

        (, , uint256[] memory amountsIn) = router.querySwapExactOut(
            paths,
            address(0),
            userData
        );

        amountIn = amountsIn[0];
    }

    /**
     * @dev Returns the amount of buyToken tokens received by spending 'specifiedAmount' sellToken tokens
     * @param pool The address of the pool to trade in.
     * @param sellToken The token being sold.
     * @param buyToken The token being bought.
     * @param specifiedAmount The amount to be traded.
     * @return amountOut The amount of tokens to receive.
     */
    function getAmountOut(
        address pool,
        IERC20 sellToken,
        IERC20 buyToken,
        uint256 specifiedAmount
    ) internal returns (uint256 amountOut) {
        bytes memory userData; // empty bytes

        IBatchRouter.SwapPathStep memory step = IBatchRouter.SwapPathStep({
            pool: pool,
            tokenOut: buyToken,
            isBuffer: false
        });
        IBatchRouter.SwapPathStep[]
            memory steps = new IBatchRouter.SwapPathStep[](1);
        steps[0] = step;

        IBatchRouter.SwapPathExactAmountIn memory path = IBatchRouter
            .SwapPathExactAmountIn({
                tokenIn: sellToken,
                steps: steps,
                exactAmountIn: specifiedAmount,
                minAmountOut: 1
            });

        IBatchRouter.SwapPathExactAmountIn[]
            memory paths = new IBatchRouter.SwapPathExactAmountIn[](1);
        paths[0] = path;

        (, , uint256[] memory amountsOut) = router.querySwapExactIn(
            paths,
            address(0),
            userData
        );

        amountOut = amountsOut[0];
    }

    /**
     * @dev Perform a sell order for ERC20 tokens
     * @param pool The address of the pool to trade in.
     * @param sellToken The token being sold.
     * @param buyToken The token being bought.
     * @param specifiedAmount The amount to be traded.
     * @return calculatedAmount The amount of tokens received.
     */
    function sellERC20(
        address pool,
        IERC20 sellToken,
        IERC20 buyToken,
        uint256 specifiedAmount
    ) internal returns (uint256 calculatedAmount) {
        // prepare constants
        bytes memory userData;
        bool isETHSell = address(sellToken) == address(0);
        bool isETHBuy = address(sellToken) == address(0);

        // prepare steps
        IBatchRouter.SwapPathStep memory step = IBatchRouter.SwapPathStep({
            pool: pool,
            tokenOut: buyToken,
            isBuffer: false
        });
        IBatchRouter.SwapPathStep[]
            memory steps = new IBatchRouter.SwapPathStep[](1);
        steps[0] = step;

        // prepare params
        IBatchRouter.SwapPathExactAmountIn memory path = IBatchRouter
            .SwapPathExactAmountIn({
                tokenIn: sellToken,
                steps: steps,
                exactAmountIn: specifiedAmount,
                minAmountOut: 1
            });
        IBatchRouter.SwapPathExactAmountIn[]
            memory paths = new IBatchRouter.SwapPathExactAmountIn[](1);
        paths[0] = path;

        // prepare swap
        uint256[] memory amountsOut;
        if (isETHSell) {
            paths[0].tokenIn = IERC20(WETH_ADDRESS);
        } else if (isETHBuy) {
            paths[0].steps[0].tokenOut = IERC20(WETH_ADDRESS);
        } else {
            // Approve and Transfer ERC20 token
            sellToken.safeTransferFrom(
                msg.sender,
                address(this),
                specifiedAmount
            );
            sellToken.safeIncreaseAllowance(address(router), specifiedAmount);
        }

        // Swap (incl. WETH)
        (, , amountsOut) = router.swapExactIn(
            paths,
            type(uint256).max,
            isETHSell || isETHBuy,
            userData
        );

        // return amount
        calculatedAmount = amountsOut[0];
    }

    /**
     * @dev Perform a sell order for ERC20 tokens
     * @param pool The address of the pool to trade in.
     * @param sellToken The token being sold.
     * @param buyToken The token being bought.
     * @param specifiedAmount The amount to be traded.
     * @return calculatedAmount The amount of tokens received.
     */
    function buyERC20(
        address pool,
        IERC20 sellToken,
        IERC20 buyToken,
        uint256 specifiedAmount
    ) internal returns (uint256 calculatedAmount) {
        // prepare constants
        bytes memory userData;
        bool isETHSell = address(sellToken) == address(0);
        bool isETHBuy = address(sellToken) == address(0);

        // prepare steps
        IBatchRouter.SwapPathStep memory step = IBatchRouter.SwapPathStep({
            pool: pool,
            tokenOut: buyToken,
            isBuffer: false
        });
        IBatchRouter.SwapPathStep[]
            memory steps = new IBatchRouter.SwapPathStep[](1);
        steps[0] = step;

        // prepare params
        IBatchRouter.SwapPathExactAmountIn memory path = IBatchRouter
            .SwapPathExactAmountIn({
                tokenIn: sellToken,
                steps: steps,
                exactAmountIn: specifiedAmount,
                minAmountOut: 1
            });
        IBatchRouter.SwapPathExactAmountIn[]
            memory paths = new IBatchRouter.SwapPathExactAmountIn[](1);
        paths[0] = path;

        // prepare swap
        uint256[] memory amountsOut;
        if (isETHSell) {
            // Set token in as WETH
            paths[0].tokenIn = IERC20(WETH_ADDRESS);
        } else if (isETHBuy) {
            // Set token out as WETH
            paths[0].steps[0].tokenOut = IERC20(WETH_ADDRESS);
        } else {
            // Get amountIn
            uint256 amountIn = getAmountIn(
                pool,
                sellToken,
                buyToken,
                specifiedAmount
            );

            // Approve and Transfer ERC20 token
            sellToken.safeTransferFrom(
                msg.sender,
                address(this),
                specifiedAmount
            );
            buyToken.safeIncreaseAllowance(address(router), specifiedAmount);
        }

        // perform swap
        (, , amountsOut) = router.swapExactIn(
            paths,
            type(uint256).max,
            isETHSell || isETHBuy,
            userData
        );

        // return amount
        calculatedAmount = amountsOut[0];
    }
}

interface IVault {
    type PoolConfigBits is bytes32;

    enum SwapKind {
        EXACT_IN,
        EXACT_OUT
    }

    enum TokenType {
        STANDARD,
        WITH_RATE
    }

    enum WrappingDirection {
        WRAP,
        UNWRAP
    }

    struct VaultSwapParams {
        SwapKind kind;
        address pool;
        IERC20 tokenIn;
        IERC20 tokenOut;
        uint256 amountGivenRaw;
        uint256 limitRaw;
        bytes userData;
    }

    struct BufferWrapOrUnwrapParams {
        SwapKind kind;
        WrappingDirection direction;
        IERC4626 wrappedToken;
        uint256 amountGivenRaw;
        uint256 limitRaw;
    }

    struct PoolData {
        PoolConfigBits poolConfigBits;
        IERC20[] tokens;
        TokenInfo[] tokenInfo;
        uint256[] balancesRaw;
        uint256[] balancesLiveScaled18;
        uint256[] tokenRates;
        uint256[] decimalScalingFactors;
    }

    struct TokenInfo {
        TokenType tokenType;
        IRateProvider rateProvider;
        bool paysYieldFees;
    }

    function swap(
        VaultSwapParams memory vaultSwapParams
    )
        external
        returns (
            uint256 amountCalculatedRaw,
            uint256 amountInRaw,
            uint256 amountOutRaw
        );

    function getPoolTokenCountAndIndexOfToken(
        address pool,
        IERC20 token
    ) external view returns (uint256 tokenCount, uint256 index);

    function erc4626BufferWrapOrUnwrap(
        BufferWrapOrUnwrapParams memory params
    )
        external
        returns (
            uint256 amountCalculatedRaw,
            uint256 amountInRaw,
            uint256 amountOutRaw
        );

    function getPoolData(address pool) external view returns (PoolData memory);

    function getPoolTokenInfo(
        address pool
    )
        external
        view
        returns (
            IERC20[] memory tokens,
            TokenInfo[] memory tokenInfo,
            uint256[] memory balancesRaw,
            uint256[] memory lastBalancesLiveScaled18
        );

    function getPoolTokens(
        address pool
    ) external view returns (IERC20[] memory tokens);
}

interface IRateProvider {
    /**
     * @dev Returns an 18 decimal fixed point number that is the exchange rate of the token to some other underlying
     * token. The meaning of this rate depends on the context.
     */
    function getRate() external view returns (uint256);
}

interface IBatchRouter {
    struct SwapPathStep {
        address pool;
        IERC20 tokenOut;
        // If true, the "pool" is an ERC4626 Buffer. Used to wrap/unwrap tokens if pool doesn't have enough liquidity.
        bool isBuffer;
    }

    struct SwapPathExactAmountIn {
        IERC20 tokenIn;
        // For each step:
        // If tokenIn == pool, use removeLiquidity SINGLE_TOKEN_EXACT_IN.
        // If tokenOut == pool, use addLiquidity UNBALANCED.
        SwapPathStep[] steps;
        uint256 exactAmountIn;
        uint256 minAmountOut;
    }

    struct SwapPathExactAmountOut {
        IERC20 tokenIn;
        // for each step:
        // If tokenIn == pool, use removeLiquidity SINGLE_TOKEN_EXACT_OUT.
        // If tokenOut == pool, use addLiquidity SINGLE_TOKEN_EXACT_OUT.
        SwapPathStep[] steps;
        uint256 maxAmountIn;
        uint256 exactAmountOut;
    }

    function querySwapExactIn(
        SwapPathExactAmountIn[] memory paths,
        address sender,
        bytes calldata userData
    )
        external
        returns (
            uint256[] memory pathAmountsOut,
            address[] memory tokensOut,
            uint256[] memory amountsOut
        );

    function querySwapExactOut(
        SwapPathExactAmountOut[] memory paths,
        address sender,
        bytes calldata userData
    )
        external
        returns (
            uint256[] memory pathAmountsIn,
            address[] memory tokensIn,
            uint256[] memory amountsIn
        );

    function swapExactIn(
        SwapPathExactAmountIn[] memory paths,
        uint256 deadline,
        bool wethIsEth,
        bytes calldata userData
    )
        external
        payable
        returns (
            uint256[] memory pathAmountsOut,
            address[] memory tokensOut,
            uint256[] memory amountsOut
        );

    function swapExactOut(
        SwapPathExactAmountOut[] memory paths,
        uint256 deadline,
        bool wethIsEth,
        bytes calldata userData
    )
        external
        payable
        returns (
            uint256[] memory pathAmountsIn,
            address[] memory tokensIn,
            uint256[] memory amountsIn
        );
}
