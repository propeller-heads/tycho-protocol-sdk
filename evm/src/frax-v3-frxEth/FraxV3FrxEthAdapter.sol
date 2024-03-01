// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {IERC20, ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {ERC20} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import {SafeERC20} from
    "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

library FixedPointMathLib {
    uint256 internal constant MAX_UINT256 = 2 ** 256 - 1;

    function mulDivDown(uint256 x, uint256 y, uint256 denominator)
        internal
        pure
        returns (uint256 z)
    {
        /// @solidity memory-safe-assembly
        assembly {
            // Equivalent to require(denominator != 0 && (y == 0 || x <=
            // type(uint256).max / y))
            if iszero(
                mul(denominator, iszero(mul(y, gt(x, div(MAX_UINT256, y)))))
            ) { revert(0, 0) }

            // Divide x * y by the denominator.
            z := div(mul(x, y), denominator)
        }
    }
}

/// @title FraxV3FrxEthAdapter
/// Adapter for frxETH and sfrxETH tokens of FraxV3
/// @dev This contract only supports: ETH -> sfrxETH and frxETH <-> sfrxETH
contract FraxV3FrxEthAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;
    using FixedPointMathLib for uint256;

    uint256 constant PRECISE_UNIT = 1e18;

    IFrxEth immutable frxEth;
    IFrxEthMinter immutable frxEthMinter;
    ISfrxEth immutable sfrxEth;

    constructor(address _frxEthMinter, address _sfrxEth) {
        sfrxEth = ISfrxEth(_sfrxEth);
        frxEth = IFrxEth(address(sfrxEth.asset()));
        require(frxEth.minters(_frxEthMinter), "Minter not enabled");

        frxEthMinter = IFrxEthMinter(_frxEthMinter);
    }

    /// @dev Modifier to check input tokens for allowed trades
    modifier onlySupportedTokens(address sellToken, address buyToken) {
        if (
            sellToken != address(frxEth) && sellToken != address(sfrxEth)
                && sellToken != address(0)
                || buyToken != address(frxEth) && buyToken != address(sfrxEth)
                || sellToken == address(0) && buyToken != address(sfrxEth)
                || buyToken == sellToken
        ) {
            revert Unavailable(
                "Only supported swaps are: ETH -> sfrxETH and frxETH <-> sfrxETH"
            );
        }

        _;
    }

    /// @dev enable receive to fill the contract with ether for payable swaps
    receive() external payable {}

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32,
        IERC20 sellToken,
        IERC20 buyToken,
        uint256[] memory _specifiedAmounts
    )
        external
        view
        override
        onlySupportedTokens(address(sellToken), address(buyToken))
        returns (Fraction[] memory _prices)
    {
        _prices = new Fraction[](_specifiedAmounts.length);

        for (uint256 i = 0; i < _specifiedAmounts.length; i++) {
            _prices[i] = getPriceAt(sellToken, _specifiedAmounts[i]);
        }
    }

    /// @inheritdoc ISwapAdapter
    /// @notice Executes a swap on the contract.
    /// @param sellToken The token being sold.
    /// @param buyToken The token being bought.
    /// @param side Either buy or sell.
    /// @param specifiedAmount The amount to be traded.
    /// @return trade The amount of tokens being sold or bought.
    function swap(
        bytes32,
        IERC20 sellToken,
        IERC20 buyToken,
        OrderSide side,
        uint256 specifiedAmount
    )
        external
        override
        onlySupportedTokens(address(sellToken), address(buyToken))
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

        uint256 numerator = address(sellToken) == address(frxEth)
            || address(sellToken) == address(0)
            ? sfrxEth.previewDeposit(PRECISE_UNIT)
            : sfrxEth.previewRedeem(PRECISE_UNIT);

        trade.price = Fraction(numerator, PRECISE_UNIT);
    }

    /// @inheritdoc ISwapAdapter
    /// @dev there is no hard cap of eth that can be staked for sfrx, but
    /// type(uint256).max reverts,
    /// @dev we are using approximately ethereum circulating supply (120
    /// Millions) as limit
    function getLimits(bytes32, IERC20 sellToken, IERC20 buyToken)
        external
        view
        override
        onlySupportedTokens(address(sellToken), address(buyToken))
        returns (uint256[] memory limits)
    {
        limits = new uint256[](2);
        address sellTokenAddress = address(sellToken);
        address buyTokenAddress = address(buyToken);
        if (sellTokenAddress == address(0)) {
            limits[0] = 120000000 ether - frxEth.totalSupply();
            limits[1] = sfrxEth.previewDeposit(limits[0]);
        } else {
            if (sellTokenAddress == address(frxEth)) {
                limits[0] =
                    frxEth.totalSupply() - frxEth.balanceOf(buyTokenAddress);
                limits[1] = sfrxEth.previewDeposit(limits[0]);
            } else {
                limits[0] = sfrxEth.totalSupply() * 90 / 100;
                limits[1] = sfrxEth.previewRedeem(limits[0]);
            }
        }
    }

    /// @inheritdoc ISwapAdapter
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
        tokens = new IERC20[](3);

        tokens[0] = IERC20(address(0));
        tokens[1] = IERC20(frxEthMinter.frxETHToken());
        tokens[2] = IERC20(frxEthMinter.sfrxETHToken());
    }

    /// @inheritdoc ISwapAdapter
    /// @dev although FraxV3 frxETH has no pool ids, we return the sFrxETH and
    /// frxETHMinter addresses as pools
    function getPoolIds(uint256, uint256)
        external
        view
        override
        returns (bytes32[] memory ids)
    {
        ids = new bytes32[](2);
        ids[0] = bytes20(address(sfrxEth));
        ids[1] = bytes20(address(frxEthMinter));
    }

    /// @notice Executes a sell order on the contract.
    /// @param sellToken The token being sold.
    /// @param amount The amount to be traded.
    /// @return calculatedAmount The amount of tokens received.
    function sell(IERC20 sellToken, uint256 amount)
        internal
        returns (uint256 calculatedAmount)
    {
        if (address(sellToken) == address(0)) {
            return frxEthMinter.submitAndDeposit{value: amount}(msg.sender);
        }

        sellToken.safeTransferFrom(msg.sender, address(this), amount);

        if (address(sellToken) == address(sfrxEth)) {
            return sfrxEth.redeem(amount, msg.sender, address(this));
        } else {
            sellToken.safeIncreaseAllowance(address(sfrxEth), amount);
            return sfrxEth.deposit(amount, msg.sender);
        }
    }

    /// @notice Executes a buy order on the contract.
    /// @param sellToken The token being sold.
    /// @param amount The amount of buyToken to receive.
    /// @return calculatedAmount The amount of tokens received.
    function buy(IERC20 sellToken, uint256 amount)
        internal
        returns (uint256 calculatedAmount)
    {
        if (address(sellToken) == address(0)) {
            uint256 amountIn = sfrxEth.previewMint(amount);
            frxEthMinter.submit{value: amountIn}();
            IERC20(address(frxEth)).safeIncreaseAllowance(
                address(sfrxEth), amountIn
            );
            return sfrxEth.mint(amount, msg.sender);
        }

        if (address(sellToken) == address(sfrxEth)) {
            uint256 amountIn = sfrxEth.previewWithdraw(amount);
            sellToken.safeTransferFrom(msg.sender, address(this), amountIn);
            return sfrxEth.withdraw(amount, msg.sender, address(this));
        } else {
            uint256 amountIn = sfrxEth.previewMint(amount);
            sellToken.safeTransferFrom(msg.sender, address(this), amountIn);
            sellToken.safeIncreaseAllowance(address(sfrxEth), amountIn);
            return sfrxEth.mint(amount, msg.sender);
        }
    }

    /// @notice Calculates prices for a specified amount
    /// @dev frxEth is 1:1 eth
    /// @param sellToken the token to sell
    /// @param amountIn The amount of the token being sold.
    /// @return (fraction) price as a fraction corresponding to the provided
    /// amount.
    function getPriceAt(IERC20 sellToken, uint256 amountIn)
        internal
        view
        returns (Fraction memory)
    {
        address sellTokenAddress = address(sellToken);

        if (
            sellTokenAddress == address(frxEth)
                || sellTokenAddress == address(0)
        ) {
            // calculate price sfrxEth/frxEth
            uint256 totStoredAssets = sfrxEth.totalAssets() + amountIn;
            uint256 newMintedShares = sfrxEth.previewDeposit(amountIn);
            uint256 totMintedShares = sfrxEth.totalSupply() + newMintedShares;
            uint256 numerator =
                PRECISE_UNIT.mulDivDown(totMintedShares, totStoredAssets);
            return Fraction(numerator, PRECISE_UNIT);
        } else {
            // calculate price frxEth/sfrxEth
            uint256 fraxAmountRedeemed = sfrxEth.previewRedeem(amountIn);
            uint256 totStoredAssets = sfrxEth.totalAssets() - fraxAmountRedeemed;
            uint256 totMintedShares = sfrxEth.totalSupply() - amountIn;
            uint256 numerator = totMintedShares == 0
                ? PRECISE_UNIT
                : PRECISE_UNIT.mulDivDown(totStoredAssets, totMintedShares);
            return Fraction(numerator, PRECISE_UNIT);
        }
    }
}

interface IFrxEth {
    // function minters_array(uint256) external view returns (address[] memory);

    function balanceOf(address) external view returns (uint256);

    function totalSupply() external view returns (uint256);

    function minters(address) external view returns (bool);
}

interface ISfrxEth {
    /// @dev even though the balance address of frxETH token is around 223,701
    /// tokens, it returns 0 when the
    /// address of frxEth is passed as an argument
    function balanceOf(address) external view returns (uint256);

    /// @dev to be clarified if the accepted asset is ETH or frxETH
    function previewDeposit(uint256 assets) external view returns (uint256);

    /// @dev It should accept sfrxETH, to be clarified if it returns ETH or
    /// frxET
    function previewMint(uint256 shares) external view returns (uint256);

    /// @dev It should accept sfrxETH, to be clarified if it returns ETH or
    /// frxET
    function previewRedeem(uint256 shares) external view returns (uint256);

    /// @dev It should accept sfrxETH, to be clarified if it returns ETH or
    /// frxET
    function previewWithdraw(uint256 assets) external view returns (uint256);

    /// @dev returns the totalSupply of frxETH
    function totalSupply() external view returns (uint256);

    /// @notice Compute the amount of tokens available to share holders
    function totalAssets() external view returns (uint256);

    function asset() external view returns (ERC20);

    function deposit(uint256 assets, address receiver)
        external
        returns (uint256 shares);

    function mint(uint256 shares, address receiver)
        external
        returns (uint256 assets);

    function storedTotalAssets() external view returns (uint256);

    function withdraw(uint256 assets, address receiver, address owner)
        external
        returns (uint256 shares);

    function redeem(uint256 shares, address receiver, address owner)
        external
        returns (uint256 assets);
}

interface IFrxEthMinter {
    //function sfrxETHTokenContract() external view returns (ISfrxEth);

    function sfrxETHToken() external view returns (address);

    function frxETHToken() external view returns (address);

    function currentWithheldETH() external view returns (uint256);

    function DEPOSIT_SIZE() external view returns (uint256);

    /// @notice Mint frxETH to the sender depending on the ETH value sent
    function submit() external payable;

    /// @notice Mint frxETH and deposit it to receive sfrxETH in one transaction
    function submitAndDeposit(address recipient)
        external
        payable
        returns (uint256 shares);
}
