// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {IERC20, ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";

/// @title Ankr BNBStakingPool Adapter
/// Adapter for Ankr staking pools which implement ankrBNB and BNBStakingPool functions
contract AnkrBNBStakingPoolAdapter is ISwapAdapter {

    IAnkrBNBStakingPool pool;

    constructor(IAnkrBNBStakingPool _pool) {
        pool = _pool;
    }

    function price(
        bytes32 _poolId,
        IERC20 _sellToken,
        IERC20 _buyToken,
        uint256[] memory _specifiedAmounts
    ) external view override returns (Fraction[] memory _prices) {
        revert NotImplemented("AnkrBNBStakingPoolAdapter.price");
    }

    function swap(
        bytes32 poolId,
        IERC20 sellToken,
        IERC20 buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external returns (Trade memory trade) {
        revert NotImplemented("AnkrBNBStakingPoolAdapter.swap");
    }

    function getLimits(bytes32 poolId, IERC20 sellToken, IERC20 buyToken)
        external
        returns (uint256[] memory limits)
    {
        revert NotImplemented("AnkrBNBStakingPoolAdapter.getLimits");
    }

    function getCapabilities(bytes32 poolId, IERC20 sellToken, IERC20 buyToken)
        external
        returns (Capability[] memory capabilities)
    {
        revert NotImplemented("AnkrBNBStakingPoolAdapter.getCapabilities");
    }

    function getTokens(bytes32 poolId)
        external
        returns (IERC20[] memory tokens)
    {
        revert NotImplemented("AnkrBNBStakingPoolAdapter.getTokens");
    }

    function getPoolIds(uint256 offset, uint256 limit)
        external
        returns (bytes32[] memory ids)
    {
        revert NotImplemented("AnkrBNBStakingPoolAdapter.getPoolIds");
    }
}

interface ILiquidTokenStakingPool {
    event BearingTokenChanged(address prevValue, address newValue);

    event CertificateTokenChanged(address prevValue, address newValue);

    event MinimumStakeChanged(uint256 prevValue, uint256 newValue);

    event MinimumUnstakeChanged(uint256 prevValue, uint256 newValue);

    event Staked(
        address indexed staker,
        uint256 amount,
        uint256 shares,
        bool indexed isRebasing
    );

    event Unstaked(
        address indexed ownerAddress,
        address indexed receiverAddress,
        uint256 amount,
        uint256 shares,
        bool indexed isRebasing
    );

    event Received(address indexed from, uint256 value);

    function setBearingToken(address newValue) external;

    function setCertificateToken(address newValue) external;

    function setMinimumStake(uint256 newValue) external;

    function setMinimumUnstake(uint256 newValue) external;

    function stakeBonds() external payable;

    function stakeCerts() external payable;

    function getFreeBalance() external view returns (uint256);

    function getMinStake() external view returns (uint256);

    function getMinUnstake() external view returns (uint256);
}

interface ICertificateToken is IERC20 {

    function sharesToBonds(uint256 amount) external view returns (uint256);

    function bondsToShares(uint256 amount) external view returns (uint256);

    function ratio() external view returns (uint256);

    function isRebasing() external pure returns (bool);

    function mint(address account, uint256 amount) external;

    function burn(address account, uint256 amount) external;
}

/// @notice Custom wrapped interface containing additional functions to ILiquidTokenStakingPool not included in the interface
/// but implemented and required by the pool contract
interface IAnkrBNBStakingPool is ILiquidTokenStakingPool {

    function swap(uint256 shares, address receiverAddress) external;

    function getTokens() external view returns (address, address);

}
