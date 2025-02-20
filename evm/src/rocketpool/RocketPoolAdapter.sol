// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

uint256 constant PRECISION = 10 ** 18;

/// @title RocketPool Adapter
contract RocketPoolAdapter is ISwapAdapter {
    RocketStorageInterface private immutable rocketStorage;
    RocketTokenRETHInterface private immutable rocketETH;
    RocketDepositPoolInterface private immutable rocketPool;
    RocketDAOProtocolSettingsDepositInterface private immutable
        rocketDaoSettings;
    RocketMinipoolQueueInterface private immutable rocketMinipoolQueue;

    constructor(RocketStorageInterface rocketStorageAddress) {
        rocketStorage = RocketStorageInterface(rocketStorageAddress);
        rocketETH = RocketTokenRETHInterface(
            rocketStorage.getAddress(
                keccak256(
                    abi.encodePacked("contract.address", "rocketTokenRETH")
                )
            )
        );
        rocketPool = RocketDepositPoolInterface(
            rocketStorage.getAddress(
                keccak256(
                    abi.encodePacked("contract.address", "rocketDepositPool")
                )
            )
        );
        rocketDaoSettings = RocketDAOProtocolSettingsDepositInterface(
            rocketStorage.getAddress(
                keccak256(
                    abi.encodePacked(
                        "contract.address", "rocketDAOProtocolSettingsDeposit"
                    )
                )
            )
        );
        rocketMinipoolQueue = RocketMinipoolQueueInterface(
            rocketStorage.getAddress(
                keccak256(
                    abi.encodePacked("contract.address", "rocketMinipoolQueue")
                )
            )
        );
    }

    /// @notice Internal check for input and output tokens
    /// @dev This contract only supports swaps between rETH<=>ETH
    /// We also check that input or output token is address(0) as ETH, to assure
    /// no wrong path prices or swaps can be executed
    modifier checkInputTokens(address sellToken, address buyToken) {
        if (sellToken != address(rocketETH) && buyToken != address(rocketETH)) {
            revert Unavailable(
                "This contract only supports rETH<=>ETH(address(0)) swaps"
            );
        }
        if (sellToken != address(0) && buyToken != address(0)) {
            revert Unavailable(
                "This contract only supports rETH<=>ETH(address(0)) swaps"
            );
        }
        _;
    }

    /// @dev enable receive to fill the contract with ether for payable swaps
    receive() external payable {}

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32,
        address sellToken,
        address buyToken,
        uint256[] memory specifiedAmounts
    )
        external
        view
        override
        checkInputTokens(sellToken, buyToken)
        returns (Fraction[] memory prices)
    {
        prices = new Fraction[](specifiedAmounts.length);

        for (uint256 i = 0; i < specifiedAmounts.length; i++) {
            prices[i] = getPriceAt(specifiedAmounts[i], sellToken);
        }
    }

    /// @inheritdoc ISwapAdapter
    function swap(
        bytes32,
        address sellToken,
        address buyToken,
        OrderSide side,
        uint256 specifiedAmount
    )
        external
        checkInputTokens(sellToken, buyToken)
        returns (Trade memory trade)
    {
        if (specifiedAmount == 0) {
            return trade;
        }

        uint256 gasBefore = gasleft();

        if (sellToken != address(0)) {
            if (side == OrderSide.Buy) {
                uint256 amountToSpend = rocketETH.getRethValue(specifiedAmount);
                rocketETH.transferFrom(msg.sender, address(this), amountToSpend);
                rocketETH.burn(amountToSpend);
            } else {
                rocketETH.transferFrom(
                    msg.sender, address(this), specifiedAmount
                );
                rocketETH.burn(specifiedAmount);
            }
        } else {
            if (side == OrderSide.Buy) {
                uint256 amountIn = rocketETH.getEthValue(
                    specifiedAmount
                        + (
                            (specifiedAmount * rocketDaoSettings.getDepositFee())
                                / PRECISION
                        )
                );
                rocketPool.deposit{value: amountIn}();
            } else {
                rocketPool.deposit{value: specifiedAmount}();
            }
        }

        trade.gasUsed = gasBefore - gasleft();
        trade.price = getPriceAt(specifiedAmount, sellToken);
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32, address sellToken, address buyToken)
        external
        view
        override
        checkInputTokens(sellToken, buyToken)
        returns (uint256[] memory limits)
    {
        uint256 ETHLimit = rocketPool.getMaximumDepositAmount();
        if (rocketDaoSettings.getAssignDepositsEnabled()) {
            ETHLimit = ETHLimit + rocketMinipoolQueue.getEffectiveCapacity();
        }
        uint256 rETHLimit = rocketETH.getRethValue(ETHLimit);

        limits = new uint256[](2);
        /**
         * @dev About MAX limits:
         * ETH Deposit limit is:
         * depositPoolMaxCapacity - depositPoolBalance (ref:
         * RocketDepositPool.sol:173),
         * But if rocketDao.getAssignDepositsEnabled() is true, an additional
         * limit of:
         * rocketMiniPoolQueue.getEffectiveCapacity()
         * is added. (ref: RocketDepositPool.sol:138)
         * rETH Deposit Limit is rETH.totalCollateral(), but we getRethValue(ETH
         * deposit limit) to make a better
         * estimate, as the swap would revert in (Side = Buy, sellToken = ETH)
         * if exceeding the ETH maxDepositAmount.
         *
         * @dev About MIN limits:
         * minLimits for rocketPool can be get via the function
         * rocketDao.getMinimumDeposit(),
         * it returns the min. amount in ETH;
         * to get minimum rETH amount, use rocketETH.getRethValue(min. amount in
         * ETH);
         *
         */
        if (sellToken == address(0)) {
            limits[0] = ETHLimit;
            limits[1] = rETHLimit;
        } else {
            limits[0] = rETHLimit;
            limits[1] = ETHLimit;
        }
    }

    /// @inheritdoc ISwapAdapter
    function getCapabilities(bytes32, address, address)
        external
        pure
        override
        returns (Capability[] memory capabilities)
    {
        capabilities = new Capability[](3);
        capabilities[0] = Capability.SellOrder;
        capabilities[1] = Capability.BuyOrder;
        capabilities[2] = Capability.PriceFunction;
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(bytes32)
        external
        view
        override
        returns (address[] memory tokens)
    {
        tokens = new address[](2);
        tokens[0] = address(0);
        tokens[1] = address(rocketETH);
    }

    function getPoolIds(uint256, uint256)
        external
        pure
        override
        returns (bytes32[] memory)
    {
        revert NotImplemented("RocketPoolAdapter.getPoolIds");
    }

    /// @notice Get swap price including fee
    /// @dev RocketPool only supports rETH<=>ETH swaps, thus we can check just
    /// one token to retrieve the price
    /// @param sellToken token to sell
    function getPriceAt(uint256 specifiedAmount, address sellToken)
        internal
        view
        returns (Fraction memory)
    {
        if (sellToken == address(0)) {
            uint256 depositFee = (
                specifiedAmount * rocketDaoSettings.getDepositFee()
            ) / PRECISION;
            uint256 amountReth =
                rocketETH.getRethValue(specifiedAmount - depositFee);
            return Fraction(amountReth, rocketETH.getEthValue(amountReth));
        } else {
            uint256 amountEth = rocketETH.getEthValue(specifiedAmount);
            return Fraction(amountEth, rocketETH.getRethValue(amountEth));
        }
    }
}

interface RocketTokenRETHInterface is IERC20 {
    function getEthValue(uint256 _rethAmount) external view returns (uint256);

    function getRethValue(uint256 _ethAmount) external view returns (uint256);

    function getExchangeRate() external view returns (uint256);

    function getTotalCollateral() external view returns (uint256);

    function getCollateralRate() external view returns (uint256);

    function depositExcess() external payable;

    function depositExcessCollateral() external;

    function mint(uint256 _ethAmount, address _to) external;

    function burn(uint256 _rethAmount) external;
}

interface RocketStorageInterface {
    // Deploy status
    function getDeployedStatus() external view returns (bool);

    // Guardian
    function getGuardian() external view returns (address);

    function setGuardian(address _newAddress) external;

    function confirmGuardian() external;

    // Getters
    function getAddress(bytes32 _key) external view returns (address);

    function getUint(bytes32 _key) external view returns (uint256);

    function getString(bytes32 _key) external view returns (string memory);

    function getBytes(bytes32 _key) external view returns (bytes memory);

    function getBool(bytes32 _key) external view returns (bool);

    function getInt(bytes32 _key) external view returns (int256);

    function getBytes32(bytes32 _key) external view returns (bytes32);

    // Setters
    function setAddress(bytes32 _key, address _value) external;

    function setUint(bytes32 _key, uint256 _value) external;

    function setString(bytes32 _key, string calldata _value) external;

    function setBytes(bytes32 _key, bytes calldata _value) external;

    function setBool(bytes32 _key, bool _value) external;

    function setInt(bytes32 _key, int256 _value) external;

    function setBytes32(bytes32 _key, bytes32 _value) external;

    // Deleters
    function deleteAddress(bytes32 _key) external;

    function deleteUint(bytes32 _key) external;

    function deleteString(bytes32 _key) external;

    function deleteBytes(bytes32 _key) external;

    function deleteBool(bytes32 _key) external;

    function deleteInt(bytes32 _key) external;

    function deleteBytes32(bytes32 _key) external;

    // Arithmetic
    function addUint(bytes32 _key, uint256 _amount) external;

    function subUint(bytes32 _key, uint256 _amount) external;

    // Protected storage
    function getNodeWithdrawalAddress(address _nodeAddress)
        external
        view
        returns (address);

    function getNodePendingWithdrawalAddress(address _nodeAddress)
        external
        view
        returns (address);

    function setWithdrawalAddress(
        address _nodeAddress,
        address _newWithdrawalAddress,
        bool _confirm
    ) external;

    function confirmWithdrawalAddress(address _nodeAddress) external;
}

interface RocketDepositPoolInterface {
    function getBalance() external view returns (uint256);

    function getNodeBalance() external view returns (uint256);

    function getUserBalance() external view returns (int256);

    function getExcessBalance() external view returns (uint256);

    function deposit() external payable;

    function getMaximumDepositAmount() external view returns (uint256);

    function nodeDeposit(uint256 _totalAmount) external payable;

    function nodeCreditWithdrawal(uint256 _amount) external;

    function recycleDissolvedDeposit() external payable;

    function recycleExcessCollateral() external payable;

    function recycleLiquidatedStake() external payable;

    function assignDeposits() external;

    function maybeAssignDeposits() external returns (bool);

    function withdrawExcessBalance(uint256 _amount) external;
}

interface RocketDAOProtocolSettingsDepositInterface {
    function getDepositEnabled() external view returns (bool);

    function getAssignDepositsEnabled() external view returns (bool);

    function getMinimumDeposit() external view returns (uint256);

    function getMaximumDepositPoolSize() external view returns (uint256);

    function getMaximumDepositAssignments() external view returns (uint256);

    function getMaximumDepositSocialisedAssignments()
        external
        view
        returns (uint256);

    function getDepositFee() external view returns (uint256);
}

enum MinipoolDeposit {
    None, // Marks an invalid deposit type
    Full, // The minipool requires 32 ETH from the node operator, 16 ETH of
        // which will be refinanced from user deposits
    Half, // The minipool required 16 ETH from the node operator to be matched
        // with 16 ETH from user deposits
    Empty, // The minipool requires 0 ETH from the node operator to be matched
        // with 32 ETH from user deposits (trusted nodes only)
    Variable // Indicates this minipool is of the new generation that supports a
        // variable deposit amount

}

interface RocketMinipoolQueueInterface {
    function getTotalLength() external view returns (uint256);

    function getContainsLegacy() external view returns (bool);

    function getLengthLegacy(MinipoolDeposit _depositType)
        external
        view
        returns (uint256);

    function getLength() external view returns (uint256);

    function getTotalCapacity() external view returns (uint256);

    function getEffectiveCapacity() external view returns (uint256);

    function getNextCapacityLegacy() external view returns (uint256);

    function getNextDepositLegacy()
        external
        view
        returns (MinipoolDeposit, uint256);

    function enqueueMinipool(address _minipool) external;

    function dequeueMinipoolByDepositLegacy(MinipoolDeposit _depositType)
        external
        returns (address minipoolAddress);

    function dequeueMinipools(uint256 _maxToDequeue)
        external
        returns (address[] memory minipoolAddress);

    function removeMinipool(MinipoolDeposit _depositType) external;

    function getMinipoolAt(uint256 _index) external view returns (address);

    function getMinipoolPosition(address _minipool)
        external
        view
        returns (int256);
}
