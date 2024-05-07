// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {IERC20, ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {SafeERC20} from
    "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {ERC20} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";

/// @title Renzo Protocol Adapter
contract RenzoAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    uint256 constant SCALE_FACTOR = 10 ** 18;

    IRestakeManager immutable restakeManager;
    IRenzoOracle immutable renzoOracle;
    IERC20 immutable ezETH;

    constructor(address _restakeManager) {
        restakeManager = IRestakeManager(_restakeManager);
        renzoOracle = restakeManager.renzoOracle();
        ezETH = IERC20(address(restakeManager.ezETH()));
    }

    /// @dev enable receive
    receive() external payable {}

    /// @dev check if buyToken is supported(only ezETH is available as
    /// buyToken); the restakeManager reverts internally if sellToken is not a
    /// supported collateral
    modifier checkBuyToken(address _buyToken) {
        if (_buyToken != address(ezETH)) {
            revert Unavailable(
                "This adapter only supports ETH and token -> ezETH swaps"
            );
        }
        _;
    }

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32,
        IERC20 _sellToken,
        IERC20 _buyToken,
        uint256[] memory _specifiedAmounts
    )
        external
        view
        override
        checkBuyToken(address(_buyToken))
        returns (Fraction[] memory _prices)
    {
        _prices = new Fraction[](_specifiedAmounts.length);
        address sellTokenAddress = address(_sellToken);

        for (uint256 i = 0; i < _specifiedAmounts.length; i++) {
            _prices[i] = getPriceAt(sellTokenAddress, true);
        }
    }

    /// @inheritdoc ISwapAdapter
    function swap(
        bytes32,
        IERC20 sellToken,
        IERC20 buyToken,
        OrderSide side,
        uint256 specifiedAmount
    )
        external
        override
        checkBuyToken(address(buyToken))
        returns (Trade memory trade)
    {
        if (specifiedAmount == 0) {
            return trade;
        }

        uint256 gasBefore = gasleft();
        if (side == OrderSide.Sell) {
            trade.calculatedAmount = sell(sellToken, specifiedAmount);
        } else {
            trade.calculatedAmount = buy(sellToken, specifiedAmount);
        }
        trade.gasUsed = gasBefore - gasleft();
        trade.price = getPriceAt(
            address(sellToken),
            false
        );
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32, IERC20 sellToken, IERC20 buyToken)
        external
        view
        override
        checkBuyToken(address(buyToken))
        returns (uint256[] memory limits)
    {
        limits = new uint256[](2);
        uint256 tokenIndex =
            restakeManager.getCollateralTokenIndex(address(sellToken));
        (uint256[][] memory operatorDelegatorTokenTVLs,, uint256 totalTvl) =
            restakeManager.calculateTVLs();
        uint256 limitInValue = totalTvl - restakeManager.maxDepositTVL();

        if (restakeManager.maxDepositTVL() != 0) {
            limitInValue = restakeManager.maxDepositTVL() - totalTvl;
        }

        uint256 collateralTvlLimitSellToken =
            restakeManager.collateralTokenTvlLimits(sellToken);

        if (collateralTvlLimitSellToken != 0) {
            uint256 currentTokenTVL = 0;
            uint256 odLength = operatorDelegatorTokenTVLs.length;
            for (uint256 i = 0; i < odLength;) {
                currentTokenTVL += operatorDelegatorTokenTVLs[i][tokenIndex];
                unchecked {
                    ++i;
                }
            }
            if (collateralTvlLimitSellToken - currentTokenTVL < limitInValue) {
                limitInValue = collateralTvlLimitSellToken - currentTokenTVL;
            }
        }
        limits[0] = limitInValue == 0
            ? type(uint256).max
            : renzoOracle.lookupTokenAmountFromValue(sellToken, limitInValue);
        /// @dev as buyToken is always ezETH but it cannot be sold since
        /// delayed/in queue, we set its limit to 0
        limits[1] = 0;
    }

    function getCapabilities(bytes32, IERC20, IERC20)
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
        returns (IERC20[] memory tokens)
    {
        uint256 tokensLength = restakeManager.getCollateralTokensLength();
        tokens = new IERC20[](tokensLength + 1);
        for (uint256 i = 0; i < tokensLength; i++) {
            tokens[i] = IERC20(restakeManager.collateralTokens(i));
        }
        tokens[tokensLength] = ezETH;
    }

    function getPoolIds(uint256, uint256)
        external
        pure
        returns (bytes32[] memory)
    {
        revert NotImplemented("RenzoAdapter.getPoolIds");
    }

    /// @notice Get swap price, incl. fee
    /// @param sellToken token to sell
    /// @param simulateTrade determine if the trade should be simulated(post trade)
    /// @dev As the price is independent of the amount, we use PRECISE_UNIT depending on the sellToken decimals to calculate prices
    /// in order to avoid rounding errors
    function getPriceAt(address sellToken, bool simulateTrade)
        internal
        view
        returns (Fraction memory)
    {
        uint256 amount = 10**ERC20(sellToken).decimals() / 100000;
        (,, uint256 totalTVL) = restakeManager.calculateTVLs();
        uint256 collateralTokenValue =
            renzoOracle.lookupTokenValue(IERC20(sellToken), amount);
        uint256 ezETHToMint = renzoOracle.calculateMintAmount(
            totalTVL, collateralTokenValue, ezETH.totalSupply()
        );

        if(simulateTrade) {
            uint256 ezETHPostTrade = renzoOracle.calculateMintAmount(
                totalTVL + collateralTokenValue, collateralTokenValue, ezETH.totalSupply() + ezETHToMint
            );
            return Fraction(ezETHPostTrade, amount);
        }

        return Fraction(ezETHToMint, amount);
    }

    /// @notice Executes a sell(mint) order on the contract.
    /// @param sellToken The token being sold.
    /// @param amount The amount to be traded.
    /// @return calculatedAmount The amount of ezETH received.
    function sell(IERC20 sellToken, uint256 amount)
        internal
        returns (uint256 calculatedAmount)
    {
        sellToken.safeTransferFrom(msg.sender, address(this), amount);
        sellToken.safeIncreaseAllowance(address(restakeManager), amount);
        uint256 balBefore = ezETH.balanceOf(address(this));
        restakeManager.deposit(sellToken, amount);
        uint256 balAfter = ezETH.balanceOf(address(this));

        calculatedAmount = balAfter - balBefore;
        ezETH.safeTransfer(msg.sender, calculatedAmount);
    }

    /// @notice Executes a buy(mint) order on the contract.
    /// @param sellToken The token being sold.
    /// @param amount The amount of ezETH being bought.
    /// @return calculatedAmount The amount of sellToken spent.
    function buy(IERC20 sellToken, uint256 amount)
        internal
        returns (uint256)
    {
        (, , uint256 totalTvl) = restakeManager.calculateTVLs();
        uint256 amountIn = calculateAmountIn(sellToken, totalTvl, ezETH.totalSupply(), amount);
        sellToken.safeTransferFrom(msg.sender, address(this), amountIn);
        sellToken.safeIncreaseAllowance(address(restakeManager), amountIn);
        restakeManager.deposit(sellToken, amountIn);

        ezETH.safeTransfer(msg.sender, amount);

        return amountIn;
    }

    /// @notice Calculate amountIn required to buy 'mintAmount' ezETH
    /// @param sellToken token to sell
    /// @param currentValueInProtocol totalTvl in protocol
    /// @param existingEzETHSupply current ezETH totalSupply
    /// @param mintAmount amount of ezETH to buy
    /// @return (uint256) amount of sellToken to spend
    function calculateAmountIn(IERC20 sellToken, uint256 currentValueInProtocol, uint256 existingEzETHSupply, uint256 mintAmount) internal view returns (uint256) {
        
        uint256 newEzETHSupply = existingEzETHSupply + mintAmount;

        uint256 inflationPercentaage = (newEzETHSupply * SCALE_FACTOR / existingEzETHSupply) - SCALE_FACTOR;

        uint256 newValueAdded = (inflationPercentaage * currentValueInProtocol) / (SCALE_FACTOR - inflationPercentaage);

        return renzoOracle.lookupTokenAmountFromValue(sellToken, newValueAdded);
    }
}

interface IEzEthToken {}

interface IRestakeManager {
    function renzoOracle() external view returns (IRenzoOracle);

    function deposit(IERC20 _collateralToken, uint256 _amount) external;

    function ezETH() external view returns (IEzEthToken);

    function getCollateralTokensLength() external view returns (uint256);

    function getCollateralTokenIndex(address _collateralToken)
        external
        view
        returns (uint256);

    function collateralTokens(uint256 i) external view returns (address);

    function maxDepositTVL() external view returns (uint256);

    function calculateTVLs()
        external
        view
        returns (uint256[][] memory, uint256[] memory, uint256);

    function collateralTokenTvlLimits(IERC20 token)
        external
        view
        returns (uint256);
}

interface IRenzoOracle {
    function lookupTokenValue(IERC20 _token, uint256 _balance)
        external
        view
        returns (uint256);
    function lookupTokenAmountFromValue(IERC20 _token, uint256 _value)
        external
        view
        returns (uint256);
    function lookupTokenValues(
        IERC20[] memory _tokens,
        uint256[] memory _balances
    ) external view returns (uint256);
    function calculateMintAmount(
        uint256 _currentValueInProtocol,
        uint256 _newValueAdded,
        uint256 _existingEzETHSupply
    ) external pure returns (uint256);
    function calculateRedeemAmount(
        uint256 _ezETHBeingBurned,
        uint256 _existingEzETHSupply,
        uint256 _currentValueInProtocol
    ) external pure returns (uint256);
}
