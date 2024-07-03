// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";

uint256 constant TOKEN_DECIMALS = 10 ** 18;

/// @title InceptionEthSwapAdapter
contract InceptionEthSwapAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    IInceptionPool immutable pool;
    IInceptionToken immutable token;

    constructor(IInceptionPool _pool, IInceptionToken _token) {
        pool = IInceptionPool(_pool);
        token = IInceptionToken(_token);
    }

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32,
        address,
        address,
        uint256[] memory specifiedAmounts
    ) external override view returns (Fraction[] memory prices) {
        prices = new Fraction[](specifiedAmounts.length);
        uint256 ratio = token.ratio();

        for (uint256 i = 0; i < specifiedAmounts.length; i++) {
            prices[i] = Fraction((ratio / TOKEN_DECIMALS) * specifiedAmounts[i], 1);
        }
    }

    /// @inheritdoc ISwapAdapter
    function swap(
        bytes32,
        address,
        address,
        OrderSide,
        uint256 specifiedAmount
    ) external returns (Trade memory trade) {
        require(address(this).balance >= specifiedAmount, "Incorrect ETH amount sent");
        if (specifiedAmount == 0) {
            return trade;
        }

        uint256 gasBefore = gasleft();
        trade.calculatedAmount = sellETH(specifiedAmount);
        trade.gasUsed = gasBefore - gasleft();

        trade.price = Fraction(specifiedAmount * token.ratio(), TOKEN_DECIMALS);
    }

    /// @notice Executes a sell order (pool stake).
    /// @param amount The amount of ETH to be staked.
    /// @return uint256 The amount of tokens received.
    function sellETH(uint256 amount) internal returns (uint256) {
        pool.stake{value: amount}();
        uint256 shares = token.balanceOf(address(this));
        if (shares == 0) {
            revert Unavailable("Shares is zero!");
        }

        return shares;
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32, address, address)
        external
        pure
        override
        returns (uint256[] memory limits)
    {
        limits = new uint256[](2);
        limits[0] = 100;
        limits[1] = 1e25;
    }

    /// @inheritdoc ISwapAdapter
    function getCapabilities(
        bytes32,
        address,
        address
    ) external pure override returns (Capability[] memory capabilities) {
        capabilities = new Capability[](2);
        capabilities[0] = Capability.SellOrder;
        capabilities[1] = Capability.PriceFunction;
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(bytes32)
        external
        pure
        override
        returns (address[] memory)
    {
        revert NotImplemented("InceptionStEthSwapAdapter.getTokens");
    }

    /// @inheritdoc ISwapAdapter
    function getPoolIds(uint256, uint256)
        external
        pure
        override
        returns (bytes32[] memory)
    {
        revert NotImplemented("InceptionStEthSwapAdapter.getPoolIds");
    }

    function availableToStake() external view returns (uint256) {
        return pool.availableToStake();
    }

    /// @notice Fallback function to receive ETH
    receive() external payable {}
}

interface IInceptionPool {
    function getMinStake() external view returns (uint256);
    function stake() external payable;
    function availableToStake() external view returns (uint256);
}

interface IInceptionToken is IERC20 {
    function ratio() external view returns (uint256);
}
