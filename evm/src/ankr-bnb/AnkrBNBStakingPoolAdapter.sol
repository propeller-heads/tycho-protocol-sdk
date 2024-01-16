// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {IERC20, ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";

/// @dev ankr constants
uint256 constant FEE_MAX = 10000;

/// @title Ankr BNBStakingPool Adapter
/// Adapter for Ankr staking pools which implement ankrBNB and BNBStakingPool functions
contract AnkrBNBStakingPoolAdapter is ISwapAdapter {

    IAnkrBNBStakingPool pool;

    constructor(IAnkrBNBStakingPool _pool) {
        pool = _pool;
    }

    /// @notice Internal check for input and output tokens
    /// @dev This contract only supports swaps between ankrBNB<=>BNB
    /// We also check that input or output token is address(0) as BNB, to assure no wrong path prices or swaps can be executed
    modifier checkInputTokens(IERC20 sellToken, IERC20 buyToken) {
        address sellTokenAddress = address(sellToken);
        address buyTokenAddress = address(buyToken);
        address certificateTokenAddress = getCertificateTokenAddress();
        if(sellTokenAddress != certificateTokenAddress && buyTokenAddress != certificateTokenAddress) {
            revert Unavailable("This contract only supports ankrBNB<=>BNB(address(0)) swaps");
        }
        if(sellTokenAddress != address(0) && buyTokenAddress != address(0)) {
            revert Unavailable("This contract only supports ankrBNB<=>BNB(address(0)) swaps");
        }
        _;
    }

    /// @dev enable receive to fill the contract with ether for payable swaps
    receive() external payable {}

    /// @inheritdoc ISwapAdapter
    /// @dev This pool only supports BNB(ether)<=>ankrBNB(certificateToken) operations, and thus prices
    function price(
        bytes32,
        IERC20 _sellToken,
        IERC20 _buyToken,
        uint256[] memory _specifiedAmounts
    ) checkInputTokens(_sellToken, _buyToken) external view override returns (Fraction[] memory _prices) {
        _prices = new Fraction[](_specifiedAmounts.length);
        address sellTokenAddress = address(_sellToken);
        address certificateTokenAddress = getCertificateTokenAddress();

        for(uint256 i = 0; i < _specifiedAmounts.length; i++) {
            _prices[i] = getPriceAt(_specifiedAmounts[i], ICertificateToken(certificateTokenAddress), sellTokenAddress != certificateTokenAddress);
        }
    }

    /// @inheritdoc ISwapAdapter
    function swap(
        bytes32,
        IERC20 sellToken,
        IERC20 buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) checkInputTokens(sellToken, buyToken) external returns (Trade memory trade) {
        if (specifiedAmount == 0) {
            return trade;
        }

        uint256 gasBefore = gasleft();
        ICertificateToken certificateToken = ICertificateToken(address(sellToken));

        if(address(sellToken) != address(0)) {
            if(side == OrderSide.Buy) {
                uint256 amountToSpendWithoutSwapFee = certificateToken.bondsToShares(specifiedAmount);
                uint256 amountToSpend = amountToSpendWithoutSwapFee + (amountToSpendWithoutSwapFee * pool.getFlashUnstakeFee() / FEE_MAX);
                certificateToken.transferFrom(msg.sender, address(this), amountToSpend);
                pool.swap(amountToSpend, address(msg.sender));
            }
            else {
                certificateToken.transferFrom(msg.sender, address(this), specifiedAmount);
                pool.swap(specifiedAmount, address(msg.sender));
            }
        }
        else {
            if(side == OrderSide.Buy) {
                uint256 amountToSpend = certificateToken.sharesToBonds(specifiedAmount); 
                pool.stakeCerts{value: amountToSpend}();
            }
            else {
                pool.stakeCerts{value: specifiedAmount}();
            }
        }

        trade.gasUsed = gasBefore - gasleft();
        trade.price = getPriceAt(specifiedAmount, certificateToken, false);
    }

    /// @inheritdoc ISwapAdapter
    /// @return limits [4]: [0, 1]: max. amounts(BNB, ankrBNB), [2, 3]: min. amounts(BNB, ankrBNB); values are inverted if sellToken is certificateTokenAddress
    function getLimits(bytes32, IERC20 sellToken, IERC20 buyToken)
        checkInputTokens(sellToken, buyToken)
        external
        view
        override
        returns (uint256[] memory limits)
    {
        limits = new uint256[](4);
        address certificateTokenAddress = getCertificateTokenAddress();
        ICertificateToken certificateToken = ICertificateToken(certificateTokenAddress);
        address sellTokenAddress = address(sellToken);
        if(sellTokenAddress != certificateTokenAddress && address(buyToken) != certificateTokenAddress) {
            revert Unavailable("This contract only supports ankrBNB<=>BNB swaps");
        }

        uint256 minBNBAmount = pool.getMinUnstake();
        uint256 maxBNBAmount = pool.flashPoolCapacity();
        if(sellTokenAddress == certificateTokenAddress) {
            limits[0] = certificateToken.bondsToShares(maxBNBAmount);
            limits[1] = maxBNBAmount;
            limits[2] = certificateToken.bondsToShares(minBNBAmount);
            limits[3] = minBNBAmount;
        }
        else {
            limits[0] = maxBNBAmount;
            limits[1] = certificateToken.bondsToShares(maxBNBAmount);
            limits[2] = minBNBAmount;
            limits[3] = certificateToken.bondsToShares(minBNBAmount);
        }
    }

    /// @inheritdoc ISwapAdapter
    function getCapabilities(bytes32, IERC20, IERC20)
        external
        pure
        override
        returns (Capability[] memory capabilities)
    {
        capabilities = new Capability[](2);
        capabilities[0] = Capability.SellOrder;
        capabilities[1] = Capability.BuyOrder;
        capabilities[2] = Capability.PriceFunction;
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
        tokens[1] = IERC20(getCertificateTokenAddress());
    }

    function getPoolIds(uint256, uint256)
        external
        pure
        override
        returns (bytes32[] memory)
    {
        revert NotImplemented("AnkrBNBStakingPoolAdapter.getPoolIds");
    }

    /// @notice Get swap price at `amount`
    /// @param amount amount to check price at
    /// @param certificateToken instance of the pool's certificateToken(ankrBNB)
    /// @param inputTokenIsEther true: input: ether, output = `amount` ether to certificateToken; false: input: certificateToken, output = `amount` certificateToken to ether
    function getPriceAt(uint256 amount, ICertificateToken certificateToken, bool inputTokenIsEther) internal view returns (Fraction memory) {
        if(inputTokenIsEther) {
            uint256 amountToShares = certificateToken.bondsToShares(amount);
            return Fraction(
                certificateToken.sharesToBonds(amountToShares),
                amountToShares
            );
        }
        uint256 amountToBonds = certificateToken.sharesToBonds(amount);
        return Fraction(
            certificateToken.bondsToShares(amountToBonds),
            amountToBonds
        );
    }

    /// @notice Get ankrBNB(certificateToken) address
    /// @dev as contract includes a function to change this token at any time, we load it from contract directly instead of declaring as static.
    function getCertificateTokenAddress() internal view returns (address certificateTokenAddress) {
        (, certificateTokenAddress) = pool.getTokens();
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

library MathUtils {

    function saturatingMultiply(uint256 a, uint256 b) internal pure returns (uint256) {
    unchecked {
        if (a == 0) return 0;
        uint256 c = a * b;
        if (c / a != b) return type(uint256).max;
        return c;
    }
    }

    function saturatingAdd(uint256 a, uint256 b) internal pure returns (uint256) {
    unchecked {
        uint256 c = a + b;
        if (c < a) return type(uint256).max;
        return c;
    }
    }

    // Preconditions:
    //  1. a may be arbitrary (up to 2 ** 256 - 1)
    //  2. b * c < 2 ** 256
    // Returned value: min(floor((a * b) / c), 2 ** 256 - 1)
    function multiplyAndDivideFloor(
        uint256 a,
        uint256 b,
        uint256 c
    ) internal pure returns (uint256) {
        return
        saturatingAdd(
            saturatingMultiply(a / c, b),
            ((a % c) * b) / c // can't fail because of assumption 2.
        );
    }

    // Preconditions:
    //  1. a may be arbitrary (up to 2 ** 256 - 1)
    //  2. b * c < 2 ** 256
    // Returned value: min(ceil((a * b) / c), 2 ** 256 - 1)
    function multiplyAndDivideCeil(
        uint256 a,
        uint256 b,
        uint256 c
    ) internal pure returns (uint256) {
        require(c != 0, "c == 0");
        return
        saturatingAdd(
            saturatingMultiply(a / c, b),
            ((a % c) * b + (c - 1)) / c // can't fail because of assumption 2.
        );
    }
}

/// @notice Custom wrapped interface containing additional functions to ILiquidTokenStakingPool not included in the interface
/// but implemented and required by the pool contract
interface IAnkrBNBStakingPool is ILiquidTokenStakingPool {

    function swap(uint256 shares, address receiverAddress) external;

    function getTokens() external view returns (address, address);

    function flashPoolCapacity() external view returns (uint256);

    function getFlashUnstakeFee() external view returns (uint256);

}
