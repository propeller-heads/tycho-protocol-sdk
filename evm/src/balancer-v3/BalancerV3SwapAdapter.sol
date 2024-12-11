// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.26;

import "./lib/BalancerSwapHelpers.sol";

/**
 * @title Balancer V3 Swap Adapter
 */
contract BalancerV3SwapAdapter is BalancerSwapHelpers {

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

        // perform swap (forward to middleware)
        trade.calculatedAmount = swapMiddleware(
            poolId,
            sellToken,
            buyToken,
            side,
            specifiedAmount
        );

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

}

