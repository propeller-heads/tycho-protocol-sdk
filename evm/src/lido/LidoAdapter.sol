// SPDX-License-Identifier: AGPL-3.0-or-later
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import {ISwapAdapter} from "src/interfaces/ISwapAdapter.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from
    "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import "forge-std/console.sol";

/// @title Lido DAO Adapter
contract LidoAdapter is ISwapAdapter {
    using SafeERC20 for IERC20;

    address public immutable wstETHAddress;
    address public immutable stETHAddress;

    constructor(address _wstETH, address _stETH) {
        wstETHAddress = _wstETH;
        stETHAddress = _stETH;
    }

    function stETH() private view returns (IStETH) {
        return IStETH(stETHAddress);
    }

    function wstETH() private view returns (IwstETH) {
        return IwstETH(wstETHAddress);
    }

    /// @notice Internal check for input and output tokens
    /// @dev This contract only supports swaps of tokens: Eth(address(0)), stEth
    /// and wstEth, Eth cannot be used as buy token because its withdrawal is
    /// not instant
    modifier checkInputTokens(address sellToken, address buyToken) {
        bool supported = true;

        if (sellToken == buyToken) {
            supported = false;
        } else {
            if (
                sellToken != address(wstEth) && sellToken != address(stEth)
                    && sellToken != address(0)
            ) {
                supported = false;
            } else {
                if (buyToken != address(wstEth) && buyToken != address(stEth)) {
                    supported = false;
                }
            }
        }

        if (!supported) {
            revert Unavailable(
                "This contract only supports wstEth<->stEth, Eth(address(0))->wstEth and Eth(address(0))->stEth swaps"
            );
        }
        _;
    }

    /// @dev enable receive to deposit ether for payable swaps
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
            prices[i] = _getPriceAt(specifiedAmounts[i], sellToken, buyToken);
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

        // Buy order: User specifies how much of buyToken they want to receive
        if (side == OrderSide.Buy) {
            if (sellToken == stETHAddress) {
                uint256 stETHTotalSupply = stETH().totalSupply();
                uint256 stETHTotalShares = stETH().getTotalShares();
                uint256 ethAmountDirectCall = stETH().getPooledEthByShares(
                    specifiedAmount
                );
                console.log("A | ethAmountDirectCall: ", ethAmountDirectCall);
                uint256 sharesAmountCalculated = (ethAmountDirectCall *
                    stETHTotalShares) / stETHTotalSupply;
                uint256 stETHBalanceCalculated = (sharesAmountCalculated *
                    stETHTotalSupply) / stETHTotalShares;

                IERC20(stETHAddress).safeTransferFrom(
                    msg.sender,
                    address(this),
                    stETHBalanceCalculated
                );
                IERC20(stETHAddress).safeIncreaseAllowance(
                    wstETHAddress,
                    stETHBalanceCalculated
                );
                uint256 receivedWstETH = wstETH().wrap(stETHBalanceCalculated);
                if (receivedWstETH < specifiedAmount - 2) {
                    revert Unavailable("Insufficient wstETH received");
                }
                IERC20(wstETHAddress).safeTransfer(msg.sender, receivedWstETH);
                trade.calculatedAmount = stETHBalanceCalculated;
            } else if (sellToken == wstETHAddress) {
                // uint256 wstETHAmountIn = (specifiedAmount *
                //     wstETH().tokensPerStEth()) / 1e18;
                // IERC20(wstETHAddress).safeTransferFrom(
                //     msg.sender,
                //     address(this),
                //     wstETHAmountIn
                // );
                // uint256 receivedStETH = wstETH().unwrap(wstETHAmountIn);
                // if (receivedStETH < specifiedAmount - 100) {
                //     revert Unavailable("Insufficient stETH received");
                // }
                // IERC20(stETHAddress).safeTransfer(msg.sender, receivedStETH);
                // trade.calculatedAmount = wstETHAmountIn;
                                // wstETH-stETH
                uint256 amountIn = wstETH().getWstETHByStETH(specifiedAmount);
                IERC20(wstETHAddress).safeTransferFrom(
                    msg.sender, address(this), amountIn
                );
                uint256 receivedStETH = wstETH().unwrap(amountIn);
                IERC20(stETHAddress).safeTransfer(msg.sender, receivedStETH);
                trade.calculatedAmount = amountIn;

            } else {
                // ETH -> stETH
                if (buyToken == stETHAddress) {
                    uint256 stETHTotalSupply = stETH().totalSupply();
                    uint256 stETHTotalShares = stETH().getTotalShares();

                    uint256 receivedSharesFromSubmit = stETH().submit{
                        value: specifiedAmount
                    }(address(0));
                    uint256 stETHBalanceCalculated = (receivedSharesFromSubmit *
                        stETHTotalSupply) / stETHTotalShares;

                    if (stETHBalanceCalculated < specifiedAmount - 2) {
                        revert Unavailable("Insufficient stETH received");
                    }

                    IERC20(stETHAddress).safeTransfer(
                        msg.sender,
                        stETHBalanceCalculated
                    );
                    trade.calculatedAmount = specifiedAmount;
                } else {
                    // ETH -> wstETH

                    // totalSupply() calls _getTotalPooledEther(), same as
                    // getTotalPooledEther()
                    // we use totalSupply and not getTotalPooledEther() because
                    // getTotalPooledEther() reverts with "revert:
                    // NON_EMPTY_DATA"
                    uint256 stETHTotalSupply = stETH().totalSupply();
                    uint256 stETHTotalShares = stETH().getTotalShares();
                    uint256 ethAmountDirectCall = stETH().getPooledEthByShares(
                        specifiedAmount
                    );
                    console.log(
                        "A | ethAmountDirectCall: ",
                        ethAmountDirectCall
                    );

                    uint256 sharesAmountCalculated = (ethAmountDirectCall *
                        stETHTotalShares) / stETHTotalSupply;

                    uint256 stETHBalanceCalculated = (sharesAmountCalculated *
                        stETHTotalSupply) / stETHTotalShares;

                    uint256 receivedSharesfromSubmit = stETH().submit{
                        value: ethAmountDirectCall
                    }(address(0));
                    console.log(
                        "A | stETH receivedShares returned from submit: ",
                        receivedSharesfromSubmit
                    );

                    IERC20(stETHAddress).safeIncreaseAllowance(
                        wstETHAddress,
                        stETHBalanceCalculated
                    );

                    uint256 receivedWstETHfromWrap = wstETH().wrap(
                        stETHBalanceCalculated
                    );
                    console.log(
                        "A | stETH receivedWstETH returned from wrap: ",
                        receivedWstETHfromWrap
                    );
                    if (receivedWstETHfromWrap < specifiedAmount - 3) {
                        revert Unavailable("Insufficient wstETH received");
                    }
                    IERC20(wstETHAddress).safeTransfer(
                        msg.sender,
                        receivedWstETHfromWrap
                    );
                    trade.calculatedAmount = ethAmountDirectCall;
                }
            }
            // In OrdeSide.Buy the specifiedAmount is the amount of buyToken to
            // buy
            // and trade.calculatedAmount is the amount of sellToken to sell in
            // order to receive the buyToken specifiedAmount
            // Price is always calculated as buyTokenAmount / sellTokenAmount
            trade.price = Fraction(specifiedAmount, trade.calculatedAmount);
        } else {
            if (sellToken == stETHAddress) {
                IERC20(stETHAddress).safeTransferFrom(
                    msg.sender,
                    address(this),
                    specifiedAmount
                );
                IERC20(stETHAddress).safeIncreaseAllowance(
                    wstETHAddress,
                    specifiedAmount
                );
                uint256 wstETHCalculatedAmountOut = wstETH().getWstETHByStETH(
                    specifiedAmount
                );
                uint256 receivedWstETH = wstETH().wrap(specifiedAmount);
                if (receivedWstETH < wstETHCalculatedAmountOut) {
                    revert Unavailable("Insufficient wstETH received");
                }
                IERC20(wstETHAddress).safeTransfer(msg.sender, receivedWstETH);
                trade.calculatedAmount = wstETHCalculatedAmountOut;
            } else if (sellToken == address(0)) {
                if (buyToken == wstETHAddress) {
                    uint256 stETHTotalSupply = stETH().totalSupply();
                    uint256 stETHTotalShares = stETH().getTotalShares();
                    uint256 sharesAmountCalculated = (specifiedAmount *
                        stETHTotalShares) / stETHTotalSupply;

                    uint256 stETHBalanceCalculated = (sharesAmountCalculated *
                        stETHTotalSupply) / stETHTotalShares;

                    uint256 wstETHCalculatedAmountOut = stETH()
                        .getSharesByPooledEth(stETHBalanceCalculated);

                    uint256 receivedSharesfromSubmit = stETH().submit{
                        value: specifiedAmount
                    }(address(0));

                    console.log(
                        "A | stETH receivedShares returned from submit: ",
                        receivedSharesfromSubmit
                    );

                    IERC20(stETHAddress).safeIncreaseAllowance(
                        wstETHAddress,
                        stETHBalanceCalculated
                    );

                    uint256 receivedWstETHfromWrap = wstETH().wrap(
                        stETHBalanceCalculated
                    );
                    console.log(
                        "A | stETH receivedWstETH returned from wrap: ",
                        receivedWstETHfromWrap
                    );
                    if (
                        receivedWstETHfromWrap < wstETHCalculatedAmountOut - 3
                    ) {
                        revert Unavailable("Insufficient wstETH received");
                    }
                    IERC20(wstETHAddress).safeTransfer(
                        msg.sender,
                        receivedWstETHfromWrap
                    );
                    trade.calculatedAmount = wstETHCalculatedAmountOut;
                } else {
                    // ETH -> stETH
                    uint256 stETHTotalSupply = stETH().totalSupply();
                    uint256 stETHTotalShares = stETH().getTotalShares();
                    uint256 sharesAmountCalculated = (specifiedAmount *
                        stETHTotalShares) / stETHTotalSupply;

                    uint256 stETHBalanceCalculated = (sharesAmountCalculated *
                        stETHTotalSupply) / stETHTotalShares;

                    (bool sent_, ) = stETHAddress.call{value: specifiedAmount}(
                        ""
                    );
                    if (!sent_) {
                        revert Unavailable(
                            "Ether transfer to stETH contract failed"
                        );
                    }
                    IERC20(stETHAddress).safeTransfer(msg.sender, specifiedAmount);
                    trade.calculatedAmount = stETHBalanceCalculated;
                }
            } else {
                // sellToken is wstETH, buyToken is stETH | wstETH -> stETH
                // ratio updates every few blocks
                IERC20(wstETHAddress).safeTransferFrom(
                    msg.sender,
                    address(this),
                    specifiedAmount
                );
                IERC20(wstETHAddress).safeIncreaseAllowance(
                    wstETHAddress,
                    specifiedAmount
                );
                // unwrap function returns the amount of stETH user receives
                // after unwrap

                uint256 calculatedStETHBalance = wstETH().getStETHByWstETH(
                    specifiedAmount
                );
                console.log("calculatedStETHBalance: ", calculatedStETHBalance);
                uint256 receivedStETH = wstETH().unwrap(specifiedAmount);
                console.log("receivedStETH: ", receivedStETH);
                IERC20(stETHAddress).safeTransfer(msg.sender, receivedStETH);
                trade.calculatedAmount = receivedStETH;
            }
            // In OrdeSide.Sell the specifiedAmount is the amount of sellToken
            // to sell
            // and trade.calculatedAmount is the amount of buyToken received by
            // selling the sellToken specifiedAmount
            // Price is always calculated as buyTokenAmount / sellTokenAmount
            trade.price = Fraction(trade.calculatedAmount, specifiedAmount);
        }

        trade.gasUsed = gasBefore - gasleft();
        return trade;
    }

    /// @inheritdoc ISwapAdapter
    function getLimits(bytes32, address sellToken, address buyToken)
        external
        view
        override
        checkInputTokens(sellToken, buyToken)
        returns (uint256[] memory limits)
    {
        uint256 currentStakeLimitStETH = 10 * 1e18; //  stETH().getCurrentStakeLimit();
        // // same
        // as ETH stake limit
        uint256 currentStakeLimitWstETH = 10 * 1e18;
        // wstETH().getWstETHByStETH(currentStakeLimitStETH);

        limits = new uint256[](2);
        if (sellToken == address(stEth)) {
            // stEth-wstEth
            limits[0] = currentStakeLimitStETH;
            limits[1] = currentStakeLimitWstETH;
        } else if (sellToken == wstETHAddress) {
            // wstETH-stETH
            limits[0] = currentStakeLimitWstETH;
            limits[1] = currentStakeLimitStETH;
        } else {
            // Eth-wstEth and Eth-stEth

            /// @dev Fix for side == Buy, because getTotalPooledEthByShares
            /// would be higher than currentStakeLimit if using the limit as
            /// amount
            uint256 pooledEthByShares_ = IStETH(stETHAddress).getPooledEthByShares(
                currentStakeLimitStETH
            );
            if (pooledEthByShares_ > currentStakeLimitStETH) {
                currentStakeLimitStETH = currentStakeLimitStETH
                    - (pooledEthByShares_ - currentStakeLimitStETH);
            }

            limits[0] = currentStakeLimitStETH;
            if (buyToken == address(stEth)) {
                limits[1] = currentStakeLimitStETH;
            } else {
                limits[1] = currentStakeLimitWstETH;
            }
        }
    }

    function getAmountInForWstETHSpecifiedAmount(
        uint256 wstETHAmount
    ) public view returns (uint256 ethAmountIn) {
        ethAmountIn = wstETH().getStETHByWstETH(wstETHAmount) + 100;
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
        capabilities[3] = Capability.ConstantPrice;
        capabilities[4] = Capability.HardLimits;
    }

    /// @inheritdoc ISwapAdapter
    function getTokens(bytes32)
        external
        view
        override
        returns (address[] memory tokens)
    {
        tokens = new address[](3);
        tokens[0] = address(0);
        tokens[1] = wstETHAddress;
        tokens[2] = stETHAddress;
    }

    function getPoolIds(uint256, uint256)
        external
        pure
        override
        returns (bytes32[] memory)
    {
        revert NotImplemented("LidoAdapter.getPoolIds");
    }

    /// @notice Get swap price between two tokens with a given specifiedAmount
    /// @param specifiedAmount amount to swap
    /// @param sellTokenAddress address of the token to sell
    /// @param buyTokenAddress address of the token to buy
    /// @return uint256 swap price
    function _getPriceAt(
        uint256 specifiedAmount,
        address sellTokenAddress,
        address buyTokenAddress
    ) internal view returns (Fraction memory) {
        uint256 amount0;
        uint256 amount1;

        if (sellTokenAddress == address(stEth)) {
            // stEth-wstEth, Eth is not possible as checked through
            // checkInputTokens
            amount0 = wstETH().getWstETHByStETH(specifiedAmount);
            amount1 = wstETH().getStETHByWstETH(amount0);
        } else if (sellTokenAddress == wstETHAddress) {
            // wstETH-stETH, ETH is not possible as checked through
            // checkInputTokens
            amount0 = wstETH().getStETHByWstETH(specifiedAmount);
            amount1 = wstETH().getWstETHByStETH(amount0);
        } else {
            // ETH (address(0))
            if (buyTokenAddress == stETHAddress) {
                amount0 = stETH().getSharesByPooledEth(specifiedAmount);
                amount1 = stETH().getPooledEthByShares(amount0);
            } else {
                uint256 stETHAmount = stETH().getSharesByPooledEth(
                    specifiedAmount
                );
                amount0 = wstETH().getWstETHByStETH(stETHAmount);
                amount1 = stETH().getPooledEthByShares(
                    wstETH().getStETHByWstETH(amount0)
                );
            }
        }

        return Fraction(amount0, amount1);
    }
}

/// @dev Wrapped and extended interface for stEth
interface IStETH is IERC20 {
    function getPooledEthByShares(uint256 _sharesAmount)
        external
        view
        returns (uint256);

    function getSharesByPooledEth(uint256 _pooledEthAmount)
        external
        view
        returns (uint256);

    function submit(address _referral) external payable returns (uint256);

    function getCurrentStakeLimit() external view returns (uint256);

    function getTotalPooledEth() external view returns (uint256);

    function getTotalShares() external view returns (uint256);
}

/// @dev Wrapped interface for wstEth
interface IwstETH is IERC20 {
    function stEth() external view returns (IStETH);

    function wrap(uint256 _stETHAmount) external returns (uint256);

    function unwrap(uint256 _wstETHAmount) external returns (uint256);

    function getWstETHByStETH(uint256 _stETHAmount)
        external
        view
        returns (uint256);

    function getStETHByWstETH(uint256 _wstETHAmount)
        external
        view
        returns (uint256);

    function stEthPerToken() external view returns (uint256);

    function tokensPerStEth() external view returns (uint256);
}
