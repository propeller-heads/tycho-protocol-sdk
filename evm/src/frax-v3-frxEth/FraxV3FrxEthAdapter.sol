// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {IERC20, ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {ERC20} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import {SafeERC20} from
    "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import "src/libraries/FractionMath.sol";

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
    using FractionMath for Fraction;
    using FixedPointMathLib for uint256;

    uint256 constant PRECISE_UNIT = 1e18;

    IFrxEth immutable frxEth;
    IFrxEthMinter immutable frxEthMinter;
    ISfrxEth immutable sfrxEth;

    constructor(address _frxEth, address _frxEthMinter, address _sfrxEth) {
        frxEth = IFrxEth(_frxEth);
        frxEthMinter = IFrxEthMinter(_frxEthMinter);
        sfrxEth = ISfrxEth(_sfrxEth);
    }

    /// @dev Modifier to check input tokens for allowed trades
    modifier onlySupportedTokens(address sellToken, address buyToken) {
        address sellTokenAddress = sellToken;
        address buyTokenAddress = buyToken;

        if (
            (sellTokenAddress == address(0) && buyTokenAddress != address(sfrxEth)) || // Condition 1
            (sellTokenAddress == address(frxEth) && buyTokenAddress != address(sfrxEth)) || // Condition 2
            (sellTokenAddress == address(sfrxEth) && buyTokenAddress != address(frxEth)) || // Condition 3
            (sellTokenAddress != address(0) && sellTokenAddress != address(frxEth) && sellTokenAddress != address(sfrxEth)) || // Condition 4
            (buyTokenAddress != address(frxEth) && buyTokenAddress != address(sfrxEth)) // Condition 5
        ) {
            revert Unavailable("Only supported swaps are: ETH -> sfrxETH and frxETH <-> sfrxETH");
        }
    
        _;
    }

    /// @inheritdoc ISwapAdapter
    function price(
        bytes32,
        IERC20 sellToken,
        IERC20 buyToken,
        uint256[] memory _specifiedAmounts
    ) external view override onlySupportedTokens(address(sellToken), address(buyToken)) returns (Fraction[] memory _prices) {
        
        _prices = new Fraction[](_specifiedAmounts.length);

        for(uint256 i = 0; i < _specifiedAmounts.length; i++) {
            _prices[i] = getPriceAt(sellToken, buyToken, _specifiedAmounts[i]);
        }
    }

    /// @inheritdoc ISwapAdapter
    /// @notice Executes a swap on the contract.
    /// @param sellToken The token being sold.
    /// @param buyToken The token being bought.
    /// @param side Either buy or sell.
    /// @param specifiedAmount The amount to be traded.
    /// @return calculatedAmount The amount of tokens being sold or bought
    function swap(
        bytes32,
        IERC20 sellToken,
        IERC20 buyToken,
        OrderSide side,
        uint256 specifiedAmount
    ) external override onlySupportedTokens(address(sellToken), address(buyToken)) returns (Trade memory trade) {
        revert("Not implemented yet");

        uint256 gasBefore = gasleft();

        if(side == OrderSide.sell)  {
            trade.calculatedAmount = sell(sellToken, specifiedAmount);
        } else {
            trade.calculatedAmount = buy(sellToken, specifiedAmount);
        }

        trade.gasUsed = gasBefore - gasleft();

        uint256 numerator = address(sellToken) == address(frxEth) || address(sellToken) == address(0)
            ? sfraxEth.previewDeposit(PRECISE_UNIT)
            : sfraxEth.previewRedeem(PRECISE_UNIT);

        trade.price = Fraction(numerator, PRECISE_UNIT);
    }

    /// @notice Executes a sell order on the contract.
    /// @param sellToken The token being sold.
    /// @param amount The amount to be traded.
    /// @return calculatedAmount The amount of tokens received.
    function sell(IERC20 sellToken, uint256 amount) internal returns (uint256 calculatedAmount){
        sellToken.safeTransferFrom(msg.sender, address(this), amount);
        if(address(sellToken) == address(sfrxEth)) {

            return sfrxEth.redeem(amount, msg.sender, address(this));
            
        } else {

            sellToken.approve(address(sfrxEth));

            if(address(sellToken) == address(frxEth)) {
                return sfrxEth.deposit(amount, msg.sender);

            } else {
                sellToken.approve(address(frxEthMinter));
                return frxEthMinter.submitAndDeposit(msg.sender);
            }
        }

    }

    /// @notice Executes a buy order on the contract.
    /// @param sellToken The token being sold.
    /// @param amount The amount of buyToken to receive.
    /// @return calculatedAmount The amount of tokens received.
    function buy(IERC20 sellToken, uint256 amount) internal returns (uint256 calculatedAmount) {
        if(address(sellToken) == address(sfrxEth)) {

            uint256 amountIn = sfrxEth.previewWithdraw(amount);
            sellToken.safeTransferFrom(msg.sender, address(this), amountIn);

            return sfrxEth.withdraw(amount, msg.sender, address(this));
            
        } else {

            if(address(sellToken) == address(frxEth)) {

                uint256 amountIn = sfrxEth.previewMint(amount);
                sellToken.safeTransferFrom(msg.sender, address(this), amountIn);
                sellToken.approve(address(sfrxEth), amountIn);
                return sfrxEth.mint(amount, msg.sender);

            } else {
                uint256 amountIn = sfrxEth.previewMint(amount);
                sellToken.safeTransferFrom(msg.sender, address(this), amountIn);
                sellToken.approve(address(sfrxEth), amountIn);
                sellToken.approve(address(frxEthMinter), amountIn);
                return frxEthMinter.submitAndDeposit(msg.sender);
            }
        }
    }

    /// @inheritdoc ISwapAdapter
    /// @dev there is no hard cap of eth that can be staked for sfrx, but type(uint256).max reverts,
    /// we are using approximately ethereum circulating supply (120 Millions) as limit
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
        if(sellTokenAddress == address(0) && buyTokenAddress == address(sfrxEth)) {

            limits[0] = 120000000 ether;
            limits[1] = sfrxEth.previewDeposit(limits[0]);

        } else {

            if (sellTokenAddress == address(frxEth) && buyTokenAddress == address(sfrxEth)) {

                limits[0] = frxEth.totalSupply() - sfrxEth.balanceOf(sellTokenAddress);
                limits[1] = sfrxEth.previewDeposit(limits[0]);

            } else {
                
                limits[0] = sfrxEth.totalSupply();
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
        returns (IERC20[] memory tokens)
    {
        tokens = new IERC20[](3);

        tokens[0] = IERC20(address(0));
        tokens[1] = IERC20(frxEthMinter.frxETHToken());
        tokens[2] = IERC20(frxEthMinter.sfrxETHToken());
    }

    /// @inheritdoc ISwapAdapter
    /// @dev although FraxV3 frxETH has no pool ids, we return the sFrxETH and frxETHMinter addresses as pools
    function getPoolIds(uint256, uint256)
        external
        returns (bytes32[] memory ids)
    {
        ids = new bytes32[](2);
        ids[0] = bytes20(address(sfrxEth));
        ids[1] = bytes20(address(frxEthMinter));
    }


    /// @notice Calculates prices for a specified amount
    /// @param amountIn The amount of the token being sold.
    /// @return (fraction) price as a fraction corresponding to the provided
    /// amount.
    function getPriceAt(IERC20 sellToken, IERC20 buyToken, uint256 amountIn)
        internal
        view
        returns (Fraction memory)
    {

        address sellTokenAddress = address(sellToken); 
        address buyTokenAddress = address(buyToken); 

        if(sellTokenAddress == address(0)) {

            uint256 storedTotalAssets = sfrxEth.totalAssets() + amountIn;
            uint256 newMintedShares = sfrxEth.previewDeposit(amountIn);
            uint256 totalSupply = sfrxEth.totalSupply() + newMintedShares;


            uint256 numerator =
                PRECISE_UNIT.mulDivDown(totalSupply, storedTotalAssets);
            return Fraction(numerator, PRECISE_UNIT);



        } else {

            if (sellTokenAddress == address(frxEth)) {
                // calculate price sfrxEth/frxEth

                uint256 storedTotalAssets = sfrxEth.totalAssets() + amountIn;
                uint256 newMintedShares = sfrxEth.previewDeposit(amountIn);
                uint256 totalSupply = sfrxEth.totalSupply() + newMintedShares;


                uint256 numerator =
                    PRECISE_UNIT.mulDivDown(totalSupply, storedTotalAssets);

                return Fraction(numerator, PRECISE_UNIT);

            } else {
                // calculate price frxEth/sfrxEth

                uint256 fraxEthRedeemed = sfrxEth.previewRedeem(amountIn);
                uint256 storedTotalAssets = sfrxEth.totalAssets() - fraxEthRedeemed;
                uint256 totalSupply = sfrxEth.totalSupply() - amountIn;

                uint256 numerator = totalSupply == 0
                    ? PRECISE_UNIT
                    : PRECISE_UNIT.mulDivDown(storedTotalAssets, totalSupply);

                return Fraction(numerator, PRECISE_UNIT);

            }

        }

    }

}


interface IFrxEth {
    // function minters_array(uint256) external view returns (address[] memory);

    function balanceOf(address) external view returns (uint256);

    function totalSupply() external view returns (uint256);
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

    /// @notice missing a public function for storedTotaAssets

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
