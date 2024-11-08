// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {SafeERC20} from
    "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {ERC20, IERC20} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";

/// @title Renzo Protocol Adapter
/// @dev This adapter only supports (supported token, ETH)->ezETH swaps
contract RenzoAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    uint256 constant SCALE_FACTOR = 10 ** 18;

    /// @dev custom value for limits(underestimate)
    uint256 constant RESERVE_LIMIT_FACTOR = 10;

    /// @dev custom scale factor for amounts used for prices, to avoid rounding
    /// errors after and before trade due to large amounts
    uint256 constant PRICE_SCALE_FACTOR = 10 ** 5;

    IRestakeManager immutable restakeManager;
    IRenzoOracle immutable renzoOracle;
    IERC20 immutable ezETH;

    constructor(address _restakeManager, address _renzoOracle, address _ezETH) {
        restakeManager = IRestakeManager(_restakeManager);
        renzoOracle = IRenzoOracle(_renzoOracle);
        ezETH = IERC20(_ezETH);
    }

    /// @dev enable receive
    receive() external payable {}

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32,
        address _sellToken,
        address,
        uint256[] memory _specifiedAmounts
    ) external view override returns (Fraction[] memory _prices) {
        _prices = new Fraction[](_specifiedAmounts.length);

        for (uint256 i = 0; i < _specifiedAmounts.length; i++) {
            _prices[i] =
                getPriceAt(_sellToken, _specifiedAmounts[i], true);
        }
    }

    /// @inheritdoc ISwapAdapter
    function swap(
        bytes32,
        address sellToken,
        address,
        OrderSide side,
        uint256 specifiedAmount
    ) external override returns (Trade memory trade) {
        if (specifiedAmount == 0) {
            return trade;
        }

        uint256 gasBefore = gasleft();
        if (side == OrderSide.Sell) {
            trade.calculatedAmount = sell(IERC20(sellToken), specifiedAmount);
        } else {
            trade.calculatedAmount = buy(IERC20(sellToken), specifiedAmount);
        }
        trade.gasUsed = gasBefore - gasleft();
        trade.price = getPriceAt(
            sellToken,
            sellToken == address(0)
                ? 10 ** 18 / PRICE_SCALE_FACTOR
                : 10 ** ERC20(sellToken).decimals() / PRICE_SCALE_FACTOR,
            false
        );
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32, address sellToken, address)
        external
        view
        override
        returns (uint256[] memory limits)
    {
        limits = new uint256[](2);
        (uint256[][] memory operatorDelegatorTokenTVLs,, uint256 totalTvl) =
            restakeManager.calculateTVLs();
        uint256 limitInValue;
        uint256 secondLimitInValue;

        if (restakeManager.maxDepositTVL() != 0) {
            limitInValue = restakeManager.maxDepositTVL() - totalTvl;
        }

        if (sellToken != address(0)) {
            uint256 tokenIndex =
                restakeManager.getCollateralTokenIndex(sellToken);

            uint256 collateralTvlLimitSellToken =
                restakeManager.collateralTokenTvlLimits(IERC20(sellToken));

            if (collateralTvlLimitSellToken != 0) {
                uint256 currentTokenTVL = 0;
                uint256 odLength = operatorDelegatorTokenTVLs.length;
                for (uint256 i = 0; i < odLength; i++) {
                    currentTokenTVL += operatorDelegatorTokenTVLs[i][tokenIndex];
                }

                secondLimitInValue =
                    collateralTvlLimitSellToken - currentTokenTVL;
            }
        }

        uint256 ezEthTotalSupply = ezETH.totalSupply();

        if (sellToken == address(0)) {
            if (limitInValue == 0) {
                limits[0] = ezEthTotalSupply / RESERVE_LIMIT_FACTOR;
            } else {
                limits[0] = limitInValue;
            }
            limits[1] = renzoOracle.calculateMintAmount(
                totalTvl, limits[0], ezEthTotalSupply
            );
        } else {
            if (limitInValue == 0) {
                if (secondLimitInValue == 0) {
                    limits[0] = ezEthTotalSupply / RESERVE_LIMIT_FACTOR;
                    limits[1] = renzoOracle.calculateMintAmount(
                        totalTvl,
                        renzoOracle.lookupTokenValue(IERC20(sellToken), limits[0]),
                        ezEthTotalSupply
                    );
                } else {
                    limits[0] = renzoOracle.lookupTokenAmountFromValue(
                        IERC20(sellToken), secondLimitInValue
                    );
                    limits[1] = renzoOracle.calculateMintAmount(
                        totalTvl, limits[0], ezEthTotalSupply
                    );
                }
            } else {
                if (secondLimitInValue < limitInValue) {
                    limits[0] = renzoOracle.lookupTokenAmountFromValue(
                        IERC20(sellToken), secondLimitInValue
                    );
                    limits[1] = renzoOracle.calculateMintAmount(
                        totalTvl, limits[0], ezEthTotalSupply
                    );
                } else {
                    limits[0] = renzoOracle.lookupTokenAmountFromValue(
                        IERC20(sellToken), limitInValue
                    );
                    limits[1] = renzoOracle.calculateMintAmount(
                        totalTvl, limits[0], ezEthTotalSupply
                    );
                }
            }
        }
    }

    /// @inheritdoc ISwapAdapter
    function getCapabilities(bytes32, address, address)
        external
        pure
        override
        returns (Capability[] memory capabilities)
    {
        capabilities = new Capability[](4);
        capabilities[0] = Capability.SellOrder;
        capabilities[1] = Capability.BuyOrder;
        capabilities[2] = Capability.PriceFunction;
        capabilities[3] = Capability.HardLimits;
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(bytes32)
        external
        view
        override
        returns (address[] memory tokens)
    {
        uint256 tokensLength = restakeManager.getCollateralTokensLength();
        tokens = new address[](tokensLength + 2);
        for (uint256 i = 0; i < tokensLength; i++) {
            tokens[i] = restakeManager.collateralTokens(i);
        }
        tokens[tokensLength] = address(ezETH);
        tokens[tokensLength + 1] = address(0);
    }

    /// @inheritdoc ISwapAdapter
    function getPoolIds(uint256, uint256)
        external
        pure
        returns (bytes32[] memory)
    {
        return new bytes32[](1);
    }

    /// @notice Get swap price, incl. fee
    /// @param sellToken token to sell
    /// @param amount the amount to get the price of
    /// @param simulateTrade determine if the price is postTrade
    function getPriceAt(address sellToken, uint256 amount, bool simulateTrade)
        internal
        view
        returns (Fraction memory)
    {
        (,, uint256 totalTVL) = restakeManager.calculateTVLs();
        uint256 collateralTokenValue = sellToken == address(0)
            ? amount
            : renzoOracle.lookupTokenValue(IERC20(sellToken), amount);
        uint256 ezETHToMint = renzoOracle.calculateMintAmount(
            totalTVL, collateralTokenValue, ezETH.totalSupply()
        );

        if (simulateTrade) {
            amount = sellToken == address(0)
                ? (10 ** 18) / PRICE_SCALE_FACTOR
                : (10 ** ERC20(sellToken).decimals()) / PRICE_SCALE_FACTOR;
            uint256 collateralTokenValueAfter = sellToken == address(0)
                ? amount
                : renzoOracle.lookupTokenValue(IERC20(sellToken), amount);
            uint256 ezETHPostTrade = renzoOracle.calculateMintAmount(
                totalTVL + collateralTokenValue,
                collateralTokenValueAfter,
                ezETH.totalSupply() + ezETHToMint
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
        uint256 balBefore = ezETH.balanceOf(address(this));
        if (address(sellToken) != address(0)) {
            sellToken.safeTransferFrom(msg.sender, address(this), amount);
            sellToken.safeIncreaseAllowance(address(restakeManager), amount);

            restakeManager.deposit(sellToken, amount);
        } else {
            restakeManager.depositETH{value: amount}();
        }
        calculatedAmount = ezETH.balanceOf(address(this)) - balBefore;
        ezETH.safeTransfer(msg.sender, calculatedAmount);
    }

    /// @notice Executes a buy(mint) order on the contract.
    /// @param sellToken The token being sold.
    /// @param amount The amount of ezETH being bought.
    /// @return calculatedAmount The amount of sellToken spent.
    function buy(IERC20 sellToken, uint256 amount) internal returns (uint256) {
        (,, uint256 totalTvl) = restakeManager.calculateTVLs();
        uint256 amountIn =
            calculateAmountIn(sellToken, totalTvl, ezETH.totalSupply(), amount);
        uint256 ezEthBalBefore = ezETH.balanceOf(address(this));

        if (address(sellToken) != address(0)) {
            sellToken.safeTransferFrom(msg.sender, address(this), amountIn);
            sellToken.safeIncreaseAllowance(address(restakeManager), amountIn);
            restakeManager.deposit(sellToken, amountIn);
        } else {
            restakeManager.depositETH{value: amountIn}();
        }

        ezETH.safeTransfer(
            msg.sender, ezETH.balanceOf(address(this)) - ezEthBalBefore
        );

        return amountIn;
    }

    /// @notice Calculate amountIn required to buy 'mintAmount' ezETH
    /// @param sellToken token to sell
    /// @param currentValueInProtocol totalTvl in protocol
    /// @param existingEzETHSupply current ezETH totalSupply
    /// @param mintAmount amount of ezETH to buy
    /// @return (uint256) amount of sellToken to spend
    function calculateAmountIn(
        IERC20 sellToken,
        uint256 currentValueInProtocol,
        uint256 existingEzETHSupply,
        uint256 mintAmount
    ) internal view returns (uint256) {
        uint256 newEzETHSupply = existingEzETHSupply + mintAmount;

        uint256 inflationPercentaage = SCALE_FACTOR
            * (newEzETHSupply - existingEzETHSupply) / newEzETHSupply;

        uint256 newValueAdded = (inflationPercentaage * currentValueInProtocol)
            / (SCALE_FACTOR - inflationPercentaage);

        return address(sellToken) == address(0)
            ? newValueAdded
            : renzoOracle.lookupTokenAmountFromValue(sellToken, newValueAdded);
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

    function depositETH() external payable;
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
