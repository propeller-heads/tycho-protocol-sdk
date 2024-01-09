// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {IERC20, ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";

/// @title RocketPool Adapter
contract RocketPoolAdapter is ISwapAdapter {

    RocketStorageInterface rocketStorage;

    constructor(RocketStorageInterface _rocketStorage) {
        rocketStorage = _rocketStorage;
    }

    /// @notice Internal check for input and output tokens
    /// @dev This contract only supports swaps between rETH<=>ETH
    /// We also check that input or output token is address(0) as ETH, to assure no wrong path prices or swaps can be executed
    modifier checkInputTokens(IERC20 sellToken, IERC20 buyToken) {
        address sellTokenAddress = address(sellToken);
        address buyTokenAddress = address(buyToken);
        address rEthTokenAddress = _getrEthTokenAddress();
        if(sellTokenAddress != rEthTokenAddress && buyTokenAddress != rEthTokenAddress) {
            revert Unavailable("This contract only supports rETH<=>ETH(address(0)) swaps");
        }
        if(sellTokenAddress != address(0) && buyTokenAddress != address(0)) {
            revert Unavailable("This contract only supports rETH<=>ETH(address(0)) swaps");
        }
        _;
    }

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32,
        IERC20 _sellToken,
        IERC20 _buyToken,
        uint256[] memory _specifiedAmounts
    ) checkInputTokens(_sellToken, _buyToken) external view override returns (Fraction[] memory _prices) {
        _prices = new Fraction[](_specifiedAmounts.length);
        
        for (uint256 i = 0; i < _specifiedAmounts.length; i++) {
            _prices[i] = getPriceAt(_specifiedAmounts[i], _sellToken);
        }
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

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32, IERC20 sellToken, IERC20 buyToken)
        checkInputTokens(sellToken, buyToken)
        external
        view
        override
        returns (uint256[] memory limits)
    {
        RocketDepositPoolInterface rocketPool = _getRocketPool();
        RocketDAOProtocolSettingsDepositInterface rocketDaoSettings = _getRocketDaoSettings();
        RocketTokenRETHInterface rocketETH = RocketTokenRETHInterface(_getrEthTokenAddress());
        uint256 maximumDepositPoolSize = rocketDaoSettings.getMaximumDepositPoolSize();
        uint256 rocketPoolBalance = rocketPool.getBalance();

        limits = new uint256[](2);
        if(address(sellToken) == address(0)) {
            limits[0] = maximumDepositPoolSize > rocketPoolBalance ? maximumDepositPoolSize - rocketPool.getBalance() : 0;
            limits[1] = rocketETH.getTotalCollateral();
        }
        else {
            limits[0] = rocketETH.getTotalCollateral();
            limits[1] = maximumDepositPoolSize > rocketPoolBalance ? maximumDepositPoolSize - rocketPool.getBalance() : 0;
        }
    }

    function getCapabilities(bytes32, IERC20, IERC20)
        external
        returns (Capability[] memory capabilities)
    {
        revert NotImplemented("TemplateSwapAdapter.getCapabilities");
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(bytes32)
        external
        view
        override
        returns (IERC20[] memory tokens)
    {
        tokens = new IERC20[](2);
        tokens[0] = IERC20(address(0));
        tokens[1] = IERC20(_getrEthTokenAddress());
    }

    function getPoolIds(uint256 offset, uint256 limit)
        external
        returns (bytes32[] memory ids)
    {
        revert NotImplemented("TemplateSwapAdapter.getPoolIds");
    }

    function _getRocketPool() internal view returns (RocketDepositPoolInterface) {
        address rocketDepositPoolAddress = rocketStorage.getAddress(keccak256(abi.encodePacked("contract.address", "rocketDepositPool")));
        RocketDepositPoolInterface rocketDepositPool = RocketDepositPoolInterface(rocketDepositPoolAddress);
        return rocketDepositPool;
    }

    function _getrEthTokenAddress() internal view returns (address) {
        address rEthTokenAddress = rocketStorage.getAddress(keccak256(abi.encodePacked("contract.address", "rocketTokenRETH")));
        return rEthTokenAddress;
    }

    function _getRocketDaoSettings() internal view returns (RocketDAOProtocolSettingsDepositInterface) {
        RocketDAOProtocolSettingsDepositInterface rocketDAOProtocolSettingsDeposit = RocketDAOProtocolSettingsDepositInterface(
            rocketStorage.getAddress(keccak256(abi.encodePacked("contract.address", "rocketDAOProtocolSettingsDeposit")))
        );
        return rocketDAOProtocolSettingsDeposit;
    }

    /// @notice Get swap price including fee
    /// @dev RocketPool only supports rETH<=>ETH swaps, thus we can check just one token to retrieve the price
    /// @param sellToken token to sell
    function getPriceAt(uint256 specifiedAmount, IERC20 sellToken) internal view returns (Fraction memory) {
        RocketTokenRETHInterface rocketETH = RocketTokenRETHInterface(_getrEthTokenAddress());
        RocketDAOProtocolSettingsDepositInterface rocketDaoSettings = _getRocketDaoSettings();

        if(address(sellToken) == address(0)) {
            uint256 depositFee = specifiedAmount * rocketDaoSettings.getDepositFee() / 10**18;
            return Fraction(
                specifiedAmount - depositFee,
                10**18
            );
        }
        else {
            return Fraction(
                rocketETH.getEthValue(specifiedAmount),
                10**18
            );
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
    function getGuardian() external view returns(address);
    function setGuardian(address _newAddress) external;
    function confirmGuardian() external;

    // Getters
    function getAddress(bytes32 _key) external view returns (address);
    function getUint(bytes32 _key) external view returns (uint);
    function getString(bytes32 _key) external view returns (string memory);
    function getBytes(bytes32 _key) external view returns (bytes memory);
    function getBool(bytes32 _key) external view returns (bool);
    function getInt(bytes32 _key) external view returns (int);
    function getBytes32(bytes32 _key) external view returns (bytes32);

    // Setters
    function setAddress(bytes32 _key, address _value) external;
    function setUint(bytes32 _key, uint _value) external;
    function setString(bytes32 _key, string calldata _value) external;
    function setBytes(bytes32 _key, bytes calldata _value) external;
    function setBool(bytes32 _key, bool _value) external;
    function setInt(bytes32 _key, int _value) external;
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
    function getNodeWithdrawalAddress(address _nodeAddress) external view returns (address);
    function getNodePendingWithdrawalAddress(address _nodeAddress) external view returns (address);
    function setWithdrawalAddress(address _nodeAddress, address _newWithdrawalAddress, bool _confirm) external;
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
    function getMaximumDepositSocialisedAssignments() external view returns (uint256);
    function getDepositFee() external view returns (uint256);
}
