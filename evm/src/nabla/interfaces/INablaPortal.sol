// SPDX-License-Identifier: MIT

pragma solidity ^0.8.13;

interface INablaPortal {
    /**
     * @notice Swap ETH for tokens using the NablaRouter
     * @param _amountIn         The amount of input ETH to swap
     * @param _amountOutMin     The minimum amount of output token that the user will accept
     * @param _tokenPath        Array of tokens to swap along the route (first token must be WETH)
     * @param _routerPath       Array of routers to use
     * @param _to               The recipient of the output tokens
     * @param _deadline         Unix timestamp after which the transaction will revert
     * @param _priceUpdateData  Array of price update data
     * @return amountOut_         Output amount of tokens
     * @dev By calling this function the price feed gets be updated (IPriceOracleAdapter.updatePriceFeeds)
     */
    function swapEthForExactTokens(
        uint256 _amountIn,
        uint256 _amountOutMin,
        address[] calldata _tokenPath,
        address[] calldata _routerPath,
        address _to,
        uint256 _deadline,
        bytes[] calldata _priceUpdateData
    ) external payable returns (uint256 amountOut_);

    /**
     * @notice Swap tokens using the NablaRouter and receive ETH
     * @param _amountIn         The amount of input tokens to swap
     * @param _amountOutMin     The minimum amount of ETH that the user will accept
     * @param _tokenPath        Array of tokens to swap along the route (last token must be WETH)
     * @param _routerPath       Array of routers to use
     * @param _to               The recipient of ETH
     * @param _deadline         Unix timestamp after which the transaction will revert
     * @param _priceUpdateData  Array of price update data
     * @return amountOut_         Output amount of ETH
     * @dev By calling this function the price feed gets be updated (IPriceOracleAdapter.updatePriceFeeds)
     */
    function swapExactTokensForEth(
        uint256 _amountIn,
        uint256 _amountOutMin,
        address[] calldata _tokenPath,
        address[] calldata _routerPath,
        address _to,
        uint256 _deadline,
        bytes[] calldata _priceUpdateData
    ) external payable returns (uint256 amountOut_);

    /**
     * @notice Swap tokens using the NablaRouter
     * @param _amountIn         The amount of input tokens to swap
     * @param _amountOutMin     The minimum amount of output token that the user will accept
     * @param _tokenPath        Array of tokens to swap along the route
     * @param _routerPath       Array of routers to use
     * @param _to               The recipient of the output tokens
     * @param _deadline         Unix timestamp after which the transaction will revert
     * @param _priceUpdateData  Array of price update data
     * @return amountOut_         Output amount of tokens
     * @dev By calling this function the price feed gets be updated (IPriceOracleAdapter.updatePriceFeeds)
     */
    function swapExactTokensForTokens(
        uint256 _amountIn,
        uint256 _amountOutMin,
        address[] calldata _tokenPath,
        address[] calldata _routerPath,
        address _to,
        uint256 _deadline,
        bytes[] calldata _priceUpdateData
    ) external payable returns (uint256 amountOut_);

    /**
     * @notice Get a quote for how many `_toToken` tokens `_amountIn` many `tokenIn`
     *         tokens can currently be swapped for.
     * @param _amountIn     The amount of input tokens to swap
     * @param _tokenPath    Array of tokens to swap along the route
     * @param _routerPath   Array of routers to use
     * @param _tokenPrices  Array of token prices fetched off-chain
     * @return amountOut_    Number of `_toToken` tokens that such a swap would yield right now
     */
    function quoteSwapExactTokensForTokens(
        uint256 _amountIn,
        address[] calldata _tokenPath,
        address[] calldata _routerPath,
        uint256[] calldata _tokenPrices
    ) external returns (uint256 amountOut_);

    /**
     * @notice Retrieves the list of routers.
     * @return routers_ An array of addresses representing the routers.
     */
    function getRouters() external view returns (address[] memory routers_);

    /**
     * @notice Retrieves the list of assets associated with a specific router.
     * @param _router The address of the router for which to retrieve the assets.
     * @return routerAssets_ An array of addresses representing the assets associated with the specified router.
     */
    function getRouterAssets(
        address _router
    ) external view returns (address[] memory routerAssets_);

    /**
     * @notice Sets the address of the guard oracle.
     * @param _guardOracleAddress The address of the new guard oracle.
     * @return success_ Confirmation of success
     */
    function setGuardOracle(
        address _guardOracleAddress
    ) external returns (bool success_);

    /**
     * @notice Activates the EVGO validation.
     * @param _router The address of the router to activate the guard for.
     */
    function enGarde(address _router) external;

    /**
     * @notice Dectivates the EVGO validation.
     * @param _router The address of the router to deactivate the guard for.
     */
    function standDown(address _router) external;
}
