// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {
    IERC20,
    SafeERC20
} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

struct PoolKey {
    address token0;
    address token1;
    uint128 fee;
    uint32 tickSpacing;
    address extension;
}

interface ILocker {
    function locked(uint256 id) external;
}

interface IPayer {
    function payCallback(uint256 id, address token) external;
}

address constant NATIVE_TOKEN_ADDRESS =
    address(0x0000000000000000000000000000eeEEee000000);

interface IFlashAccountant {
    function lock() external;

    function pay(address token) external;

    function withdraw(address token, address recipient, uint128 amount)
        external;

    receive() external payable;
}

interface IExposedStorage {
    function sload(bytes32 slot) external view returns (bytes32 result);
    function tload(bytes32 slot) external view returns (bytes32 result);
}

library ExposedStorageLib {
    function unsafeRead(IExposedStorage target, bytes32 slot)
        internal
        view
        returns (bytes32 result)
    {
        assembly ("memory-safe") {
            mstore(0, shl(224, 0xf4910a73))
            mstore(4, slot)

            pop(staticcall(gas(), target, 0, 36, 0, 32))

            result := mload(0)
        }
    }

    function unsafeReadTransient(IExposedStorage target, bytes32 slot)
        internal
        view
        returns (bytes32 result)
    {
        assembly ("memory-safe") {
            mstore(0, shl(224, 0xbd2e587d))
            mstore(4, slot)

            pop(staticcall(gas(), target, 0, 36, 0, 32))

            result := mload(0)
        }
    }
}

library CoreLib {
    using ExposedStorageLib for *;

    function poolPrice(ICore core, bytes32 poolId)
        internal
        view
        returns (uint192 sqrtRatio, int32 tick)
    {
        bytes32 key;
        assembly ("memory-safe") {
            mstore(0, poolId)
            mstore(32, 2)
            key := keccak256(0, 64)
        }

        bytes32 result = core.unsafeRead(key);

        assembly ("memory-safe") {
            sqrtRatio :=
                and(result, 0xffffffffffffffffffffffffffffffffffffffffffffffff)
            tick := shr(192, result)
        }
    }

    function poolLiquidity(ICore core, bytes32 poolId)
        internal
        view
        returns (uint128 liquidity)
    {
        bytes32 key;
        assembly ("memory-safe") {
            mstore(0, poolId)
            mstore(32, 3)
            key := keccak256(0, 64)
        }

        bytes32 result = core.unsafeRead(key);

        assembly ("memory-safe") {
            liquidity := and(result, 0xffffffffffffffffffffffffffffffff)
        }
    }
}

struct SwapParameters {
    int128 amount;
    bool isToken1;
    uint256 sqrtRatioLimit;
    uint256 skipAhead;
}

interface ICore is IFlashAccountant, IExposedStorage {
    // Loads from the saved balance of the contract to pay in the current lock
    // context.
    function load(address token, bytes32 salt, uint128 amount) external;

    // Saves an amount of a token to be used later.
    function save(address owner, address token, bytes32 salt, uint128 amount)
        external;

    function swap(PoolKey memory poolKey, SwapParameters memory params)
        external
        returns (int128 delta0, int128 delta1);
}

abstract contract BaseLocker is ILocker, IPayer {
    using SafeERC20 for IERC20;

    error BaseLockerAccountantOnly();

    IFlashAccountant internal immutable accountant;

    constructor(IFlashAccountant _accountant) {
        accountant = _accountant;
    }

    /// CALLBACK HANDLERS

    function locked(uint256 id) external {
        if (msg.sender != address(accountant)) {
            revert BaseLockerAccountantOnly();
        }

        bytes memory data = msg.data[36:];

        bytes memory result = handleLockData(id, data);

        assembly ("memory-safe") {
            // raw return whatever the handler sent
            return(add(result, 32), mload(result))
        }
    }

    function payCallback(uint256, address token) external {
        if (msg.sender != address(accountant)) {
            revert BaseLockerAccountantOnly();
        }

        address from;
        uint256 amount;
        assembly ("memory-safe") {
            from := calldataload(68)
            amount := calldataload(100)
        }

        if (from != address(this)) {
            IERC20(token).safeTransferFrom(from, address(accountant), amount);
        } else {
            IERC20(token).safeTransfer(address(accountant), amount);
        }
    }

    /// INTERNAL FUNCTIONS

    function lock(bytes memory data) internal returns (bytes memory result) {
        address target = address(accountant);

        assembly ("memory-safe") {
            // We will store result where the free memory pointer is now, ...
            result := mload(0x40)

            // But first use it to store the calldata

            // Selector of lock()
            mstore(result, shl(224, 0xf83d08ba))

            // We only copy the data, not the length, because the length is read
            // from the calldata size
            let len := mload(data)
            mcopy(add(result, 4), add(data, 32), len)

            // If the call failed, pass through the revert
            if iszero(call(gas(), target, 0, result, add(len, 36), 0, 0)) {
                returndatacopy(0, 0, returndatasize())
                revert(0, returndatasize())
            }

            // Copy the entire return data into the space where the result is
            // pointing
            mstore(result, returndatasize())
            returndatacopy(add(result, 32), 0, returndatasize())

            // Update the free memory pointer to be after the end of the data,
            // aligned to the next 32 byte word
            mstore(
                0x40,
                and(add(add(result, add(32, returndatasize())), 31), not(31))
            )
        }
    }

    error ExpectedRevertWithinLock();

    function lockAndExpectRevert(bytes memory data)
        internal
        returns (bytes memory result)
    {
        address target = address(accountant);

        assembly ("memory-safe") {
            // We will store result where the free memory pointer is now, ...
            result := mload(0x40)

            // But first use it to store the calldata

            // Selector of lock()
            mstore(result, shl(224, 0xf83d08ba))

            // We only copy the data, not the length, because the length is read
            // from the calldata size
            let len := mload(data)
            mcopy(add(result, 4), add(data, 32), len)

            // If the call succeeded, revert with
            // ExpectedRevertWithinLock.selector
            if call(gas(), target, 0, result, add(len, 36), 0, 0) {
                mstore(0, shl(224, 0x4c816e2b))
                revert(0, 0)
            }

            // Copy the entire revert data into the space where the result is
            // pointing
            mstore(result, returndatasize())
            returndatacopy(add(result, 32), 0, returndatasize())

            // Update the free memory pointer to be after the end of the data,
            // aligned to the next 32 byte word
            mstore(
                0x40,
                and(add(add(result, add(32, returndatasize())), 31), not(31))
            )
        }
    }

    function pay(address from, address token, uint256 amount) internal {
        address target = address(accountant);

        if (amount > 0) {
            if (token == NATIVE_TOKEN_ADDRESS) {
                address(accountant).call{value: amount}("");
            } else {
                assembly ("memory-safe") {
                    let free := mload(0x40)
                    // selector of pay(address)
                    mstore(free, shl(224, 0x0c11dedd))
                    mstore(add(free, 4), token)
                    mstore(add(free, 36), from)
                    mstore(add(free, 68), amount)

                    // if it failed, pass through revert
                    if iszero(call(gas(), target, 0, free, 100, 0, 0)) {
                        returndatacopy(0, 0, returndatasize())
                        revert(0, returndatasize())
                    }
                }
            }
        }
    }

    function withdraw(address token, uint128 amount, address recipient)
        internal
    {
        if (amount > 0) {
            accountant.withdraw(token, recipient, amount);
        }
    }

    function handleLockData(uint256 id, bytes memory data)
        internal
        virtual
        returns (bytes memory result);
}

contract EkuboSwapAdapter is ISwapAdapter, BaseLocker {
    using SafeERC20 for IERC20;

    ICore immutable core;

    constructor(ICore core_) BaseLocker(core_) {
        core = core_;
    }

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        uint256[] memory specifiedAmounts
    ) external view override returns (Fraction[] memory prices) {
        revert NotImplemented("todo");
    }

    /// @inheritdoc ISwapAdapter
    function swap(
        bytes32 poolId,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external override returns (Trade memory trade) {
        revert NotImplemented("todo");
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32 poolId, address sellToken, address buyToken)
        external
        view
        override
        returns (uint256[] memory limits)
    {
        revert NotImplemented("todo");
    }

    /// @inheritdoc ISwapAdapter
    function getCapabilities(bytes32, address, address)
        external
        pure
        override
        returns (Capability[] memory capabilities)
    {
        capabilities = new Capability[](5);
        capabilities[0] = Capability.SellOrder;
        capabilities[1] = Capability.BuyOrder;
        capabilities[2] = Capability.PriceFunction;
        capabilities[3] = Capability.MarginalPrice;
        capabilities[4] = Capability.FeeOnTransfer;
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(bytes32 poolId)
        external
        view
        override
        returns (address[] memory tokens)
    {
        revert NotImplemented("todo");
    }

    /// @inheritdoc ISwapAdapter
    function getPoolIds(uint256 offset, uint256 limit)
        external
        view
        override
        returns (bytes32[] memory ids)
    {
        revert NotImplemented("todo");
    }

    function handleLockData(uint256 id, bytes memory data)
        internal
        override
        returns (bytes memory)
    {
        revert NotImplemented("todo");
    }
}
