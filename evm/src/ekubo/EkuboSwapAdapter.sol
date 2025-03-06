// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {ISwapAdapterV2} from "src/interfaces/ISwapAdapterV2.sol";
import {
    IERC20,
    SafeERC20
} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {SqrtRatioFloat, MIN_SQRT_RATIO, MAX_SQRT_RATIO} from "./SqrtRatio.sol";

struct PoolKey {
    address token0;
    address token1;
    bytes32 config;
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

function isPriceIncreasing(int128 amount, bool isToken1) pure returns (bool increasing) {
    assembly ("memory-safe") {
        increasing := xor(isToken1, slt(amount, 0))
    }
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
        returns (SqrtRatioFloat sqrtRatio)
    {
        bytes32 key;
        assembly ("memory-safe") {
            mstore(0, poolId)
            mstore(32, 2)
            key := keccak256(0, 64)
        }

        bytes32 p = core.unsafeRead(key);

        assembly ("memory-safe") {
            sqrtRatio := and(p, 0xffffffffffffffffffffffff)
        }
    }
}

struct SwapParameters {
    int128 amount;
    bool isToken1;
    uint96 sqrtRatioLimit;
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

contract EkuboSwapAdapter is ISwapAdapterV2, BaseLocker {
    using SafeERC20 for IERC20;
    using CoreLib for ICore;

    uint256 private constant TWO_POW_127 = 1 << 127;
    uint256 private constant INT128_MAX = TWO_POW_127 - 1;

    ICore immutable core;

    constructor(ICore core_) BaseLocker(core_) {
        core = core_;
    }

    /// @inheritdoc ISwapAdapterV2
    function price(
        bytes memory /*poolId*/,
        address /*sellToken*/,
        address /*buyToken*/,
        uint256[] memory /*specifiedAmounts*/,
        bytes memory /*data*/
    ) external pure override returns (Fraction[] memory /*prices*/) {
        revert NotImplemented("not supported with view");
    }

    /// @inheritdoc ISwapAdapterV2
    function swap(
        bytes memory poolId,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount,
        bytes memory data
    ) external override returns (Trade memory trade) {
        PoolKey memory poolKey = decodePoolKey(poolId);
        uint256 skipAhead = abi.decode(data, (uint256));

        int128 amount;
        bool isToken1;
        SqrtRatioFloat sqrtRatioLimit;

        if (side == OrderSide.Sell) {
            if (specifiedAmount > INT128_MAX) {
                revert AmountOverflow();
            }

            amount = int128(uint128(specifiedAmount));
            isToken1 = poolKey.token1 == sellToken;
            sqrtRatioLimit = isToken1 ? MAX_SQRT_RATIO : MIN_SQRT_RATIO;
        } else {
            if (specifiedAmount > TWO_POW_127) {
                revert AmountOverflow();
            }

            amount = int128(-int256(specifiedAmount));
            isToken1 = poolKey.token0 == buyToken;
            sqrtRatioLimit = isToken1 ? MIN_SQRT_RATIO : MAX_SQRT_RATIO;
        }

        bytes memory res = lock(abi.encode(
            msg.sender,
            poolKey,
            SwapParameters(
                amount,
                isToken1,
                SqrtRatioFloat.unwrap(sqrtRatioLimit),
                skipAhead
            )
        ));

        (uint256 gasUsed, uint256 calculatedAmount) = abi.decode(res, (uint256, uint256));

        SqrtRatioFloat sqrtRatioAfter = core.poolPrice(computePoolId(poolKey));

        trade = Trade(
            calculatedAmount,
            gasUsed,
            sqrtRatioAfter.toFixed().toRational()
        );
    }

    /// @inheritdoc ISwapAdapterV2
    function getLimits(bytes memory /*poolId*/, address /*sellToken*/, address /*buyToken*/, bytes memory /*data*/)
        external
        pure
        override
        returns (uint256[] memory /*limits*/)
    {
        revert NotImplemented("not supported with view");
    }

    /// @inheritdoc ISwapAdapterV2
    function getCapabilities(bytes memory, address, address)
        external
        pure
        override
        returns (Capability[] memory capabilities)
    {
        capabilities = new Capability[](4);
        capabilities[0] = Capability.SellOrder;
        capabilities[1] = Capability.BuyOrder;
        capabilities[2] = Capability.MarginalPrice;
        capabilities[3] = Capability.FeeOnTransfer;
    }

    /// @inheritdoc ISwapAdapterV2
    function getTokens(bytes memory poolId)
        external
        pure
        override
        returns (address[] memory tokens)
    {
        PoolKey memory poolKey = decodePoolKey(poolId);

        tokens = new address[](2);
        tokens[0] = poolKey.token0;
        tokens[1] = poolKey.token1;
    }

    /// @inheritdoc ISwapAdapterV2
    function getPoolIds(uint256, uint256)
        external
        pure
        override
        returns (bytes32[] memory)
    {
        revert NotImplemented("infinite possible pools");
    }

    function handleLockData(uint256, bytes memory data)
        internal
        override
        returns (bytes memory res)
    {
        (
            address swapper,
            PoolKey memory poolKey,
            SwapParameters memory swapParameters
        ) = abi.decode(data, (address, PoolKey, SwapParameters));

        uint256 gasLeftBefore = gasleft();
        (int128 delta0, int128 delta1) = core.swap(poolKey, swapParameters);
        uint256 swapGasUsed = gasLeftBefore - gasleft();

        if (isPriceIncreasing(swapParameters.amount, swapParameters.isToken1)) {
            withdraw(poolKey.token0, uint128(-delta0), swapper);
            pay(swapper, poolKey.token1, uint128(delta1));
        } else {
            withdraw(poolKey.token1, uint128(-delta1), swapper);
            pay(swapper, poolKey.token0, uint128(delta0));
        }

        res = abi.encode(swapGasUsed);
    }

    function decodePoolKey(bytes memory enc) internal pure returns (PoolKey memory poolKey) {
        poolKey = abi.decode(enc, (PoolKey));
    }

    function computePoolId(PoolKey memory poolKey) internal pure returns (bytes32 poolId) {
        poolId = keccak256(
            abi.encode(
                poolKey.token0,
                poolKey.token1,
                poolKey.config
            )
        );
    }

    error AmountOverflow();
}
