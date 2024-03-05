// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {IERC20, ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";

contract BancorV3SwapAdapter is ISwapAdapter {
    IBancorV3BancorNetwork immutable bancorNetwork;

    constructor (address bancorNetwork_) {
        bancorNetwork = IBancorV3BancorNetwork(bancorNetwork_);
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

    /// @inheritdoc ISwapAdapter
    function getPoolIds(uint256 offset, uint256 limit)
        external
        view
        override
        returns (bytes32[] memory ids)
    {
        uint256 endIdx = offset + limit;
        Token[] memory tokenPools = bancorNetwork.liquidityPools(); 
        if (endIdx > tokenPools.length) {
            endIdx = tokenPools.length;
        }
        ids = new bytes32[](endIdx - offset);
        for (uint256 i = 0; i < ids.length; i++) {
            ids[i] = bytes20(address(tokenPools[offset + i]));
        }
    }


}


interface IBancorV3BancorNetwork {

    //function poolCollections() external view returns (IPoolCollection[] memory);

    /// @dev returns the set of all liquidity pools
    function liquidityPools() external view returns (Token[] memory);

    /// @dev returns the respective pool collection for the provided pool
    // function collectionByPool(Token pool) external view returns (IPoolCollection);

    /**
     * @dev performs a trade by providing the input source amount, sends the proceeds to the optional beneficiary (or
     * to the address of the caller, in case it's not supplied), and returns the trade target amount
     *
     * requirements:
     *
     * - the caller must have approved the network to transfer the source tokens on its behalf (except for in the
     *   native token case)
     */
    function tradeBySourceAmount(
        Token sourceToken,
        Token targetToken,
        uint256 sourceAmount,
        uint256 minReturnAmount,
        uint256 deadline,
        address beneficiary
    ) external payable returns (uint256);

    /**
     * @dev performs a trade by providing the output target amount, sends the proceeds to the optional beneficiary (or
     * to the address of the caller, in case it's not supplied), and returns the trade source amount
     *
     * requirements:
     *
     * - the caller must have approved the network to transfer the source tokens on its behalf (except for in the
     *   native token case)
     */
    function tradeByTargetAmount(
        Token sourceToken,
        Token targetToken,
        uint256 targetAmount,
        uint256 maxSourceAmount,
        uint256 deadline,
        address beneficiary
    ) external payable returns (uint256);





}

interface Token {

}