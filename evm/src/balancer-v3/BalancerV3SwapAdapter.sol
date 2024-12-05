// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {IERC20, SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC4626} from "openzeppelin-contracts/contracts/interfaces/IERC4626.sol";


contract BalancerV3SwapAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    // Balancer V3 constants
    uint256 constant RESERVE_LIMIT_FACTOR = 3;
    uint256 constant SWAP_DEADLINE_SEC = 1000;

    IVault immutable vault;

    constructor(address payable vault_) {
        vault = IVault(vault_);
    }

    function price(
        bytes32 _poolId,
        address _sellToken,
        address _buyToken,
        uint256[] memory _specifiedAmounts
    ) external view override returns (Fraction[] memory _prices) {
        revert NotImplemented("BalancerV3SwapAdapter.price");
    }

    function swap(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external returns (Trade memory trade) {
        revert NotImplemented("BalancerV3SwapAdapter.swap");
    }

    function getLimits(
        bytes32 poolId,
        address sellToken,
        address buyToken
    ) external returns (uint256[] memory limits) {
        revert NotImplemented("BalancerV3SwapAdapter.getLimits");
    }

    function getCapabilities(
        bytes32 poolId,
        address sellToken,
        address buyToken
    ) external returns (Capability[] memory capabilities) {
        revert NotImplemented("BalancerV3SwapAdapter.getCapabilities");
    }

    function getTokens(
        bytes32 poolId
    ) external returns (address[] memory tokens) {
        revert NotImplemented("BalancerV3SwapAdapter.getTokens");
    }

    function getPoolIds(
        uint256 offset,
        uint256 limit
    ) external returns (bytes32[] memory ids) {
        revert NotImplemented("BalancerV3SwapAdapter.getPoolIds");
    }
}

interface IVault {
    enum SwapKind {
        EXACT_IN,
        EXACT_OUT
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
}
