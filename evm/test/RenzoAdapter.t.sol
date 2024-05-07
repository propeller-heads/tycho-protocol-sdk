// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import "src/renzo/RenzoAdapter.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "forge-std/console.sol";
import {FractionMath} from "src/libraries/FractionMath.sol";

interface IwstETH is IERC20 {
    function unwrap(uint256 _wstETHAmount) external returns (uint256);
    function getStETHByWstETH(uint256 _wstETHAmount) external view returns (uint256);
}

contract RenzoAdapterTest is Test, ISwapAdapterTypes {

/// @dev Error for 0x0 address inputs
error InvalidZeroInput();    

/// @dev Error for already added items to a list
error AlreadyAdded();    

/// @dev Error for not found items in a list
error NotFound();    

/// @dev Error for hitting max TVL
error MaxTVLReached();

/// @dev Error for caller not having permissions
error NotRestakeManagerAdmin();

/// @dev Error for call not coming from deposit queue contract
error NotDepositQueue();

/// @dev Error for contract being paused
error ContractPaused(); 

/// @dev Error for exceeding max basis points (100%)
error OverMaxBasisPoints();

/// @dev Error for invalid token decimals for collateral tokens (must be 18)
error InvalidTokenDecimals(uint8 expected, uint8 actual);

/// @dev Error when withdraw is already completed
error WithdrawAlreadyCompleted();

/// @dev Error when a different address tries to complete withdraw
error NotOriginalWithdrawCaller(address expectedCaller);

/// @dev Error when caller does not have OD admin role
error NotOperatorDelegatorAdmin();

/// @dev Error when caller does not have Oracle Admin role
error NotOracleAdmin();

/// @dev Error when caller is not RestakeManager contract
error NotRestakeManager();

/// @dev Errror when caller does not have ETH Restake Admin role
error NotNativeEthRestakeAdmin();

/// @dev Error when delegation address was already set - cannot be set again
error DelegateAddressAlreadySet();

/// @dev Error when caller does not have ERC20 Rewards Admin role
error NotERC20RewardsAdmin();

/// @dev Error when ending ETH fails
error TransferFailed();

/// @dev Error when caller does not have ETH Minter Burner Admin role
error NotEzETHMinterBurner();

/// @dev Error when caller does not have Token Admin role
error NotTokenAdmin();

/// @dev Error when price oracle is not configured
error OracleNotFound();

/// @dev Error when price oracle data is stale
error OraclePriceExpired();

/// @dev Error when array lengths do not match
error MismatchedArrayLengths();

/// @dev Error when caller does not have Deposit Withdraw Pauser role
error NotDepositWithdrawPauser();

/// @dev Error when an individual token TVL is over the max
error MaxTokenTVLReached();

/// @dev Error when Oracle price is invalid
error InvalidOraclePrice();

/// @dev Error when calculating token amounts is invalid
error InvalidTokenAmount();
    using FractionMath for Fraction;

    RenzoAdapter adapter;
    IERC20 ezETH;
    IRestakeManager constant restakeManager =
        IRestakeManager(0x74a09653A083691711cF8215a6ab074BB4e99ef5);
    IERC20 wBETH = IERC20(0xa2E3356610840701BDf5611a53974510Ae27E2e1);
    IERC20 stETH = IERC20(0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84);
    IwstETH constant wstETH = IwstETH(0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0);
    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 19797651; //19797651;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);
        adapter = new RenzoAdapter(address(restakeManager));
        ezETH = IERC20(address(restakeManager.ezETH()));

        vm.label(address(adapter), "RenzoAdapter");
        vm.label(address(ezETH), "ezETH");
        vm.label(address(wBETH), "wBETH");
    }
    
    function dealStEthTokens(uint256 amount) internal {
        uint256 wstETHAmount = wstETH.getStETHByWstETH(amount);
        deal(address(wstETH), address(this), wstETHAmount);
        wstETH.unwrap(wstETHAmount);
    }

    function testPriceFuzzRenzo(uint256 amount0, uint256 amount1) public {
        bytes32 pair = bytes32(0);
        uint256[] memory limits = adapter.getLimits(pair, wBETH, ezETH);
        /// @dev Amounts below 10**6 cause underflow, so this is implicitly the
        /// min. limit
        vm.assume(amount0 < limits[0] && amount0 > 10 ** 6);
        vm.assume(amount1 < limits[0] && amount1 > 10 ** 6);

        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;

        Fraction[] memory prices = adapter.price(pair, wBETH, ezETH, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testSwapFuzzRenzo() public {
        OrderSide side = OrderSide.Buy; // isBuy ? OrderSide.Buy : OrderSide.Sell;

        bytes32 pair = bytes32(0);
        uint256[] memory limits = adapter.getLimits(pair, wBETH, ezETH);

        uint256 specifiedAmount = 10**18 / 10;

        if (side == OrderSide.Buy) {
            // vm.assume(specifiedAmount < limits[1] && specifiedAmount > 10 ** 6);

            deal(address(wBETH), address(this), type(uint256).max);
            wBETH.approve(address(adapter), type(uint256).max);
        } else {
            // vm.assume(specifiedAmount < limits[0] && specifiedAmount > 10 ** 6);

            deal(address(wBETH), address(this), specifiedAmount);
            wBETH.approve(address(adapter), specifiedAmount);
        }

        uint256 wBETH_balance = wBETH.balanceOf(address(this));
        uint256 ezETH_balance = ezETH.balanceOf(address(this));

        Trade memory trade =
            adapter.swap(pair, wBETH, ezETH, side, specifiedAmount);

        if (trade.calculatedAmount > 0) {
            if (side == OrderSide.Buy) {
                assertEq(
                    specifiedAmount,
                    ezETH.balanceOf(address(this)) - ezETH_balance
                );
                assertEq(
                    trade.calculatedAmount,
                    wBETH_balance - wBETH.balanceOf(address(this))
                );
            } else {
                assertEq(
                    specifiedAmount,
                    wBETH_balance - wBETH.balanceOf(address(this))
                );
                assertEq(
                    trade.calculatedAmount,
                    ezETH.balanceOf(address(this)) - ezETH_balance
                );
            }
        }
    }

    function testSwapSellIncreasingRenzo() public {
        executeIncreasingSwapsRenzo(OrderSide.Sell);
    }

    function testSwapBuyIncreasingRenzo() public {
        executeIncreasingSwapsRenzo(OrderSide.Buy);
    }

    function executeIncreasingSwapsRenzo(OrderSide side) internal {
        bytes32 pair = bytes32(0);

        uint256 amountConstant_ =
            side == OrderSide.Sell ? 1000 * 10 ** 7 : 10 ** 17;

        uint256[] memory amounts = new uint256[](TEST_ITERATIONS);
        amounts[0] = amountConstant_;
        for (uint256 i = 1; i < TEST_ITERATIONS; i++) {
            amounts[i] = amountConstant_ * i;
        }

        Trade[] memory trades = new Trade[](TEST_ITERATIONS);
        uint256 beforeSwap;
        for (uint256 i = 1; i < TEST_ITERATIONS; i++) {
            beforeSwap = vm.snapshot();

            deal(address(wBETH), address(this), amounts[i]);
            wBETH.approve(address(adapter), amounts[i]);

            trades[i] = adapter.swap(pair, wBETH, ezETH, side, amounts[i]);
            vm.revertTo(beforeSwap);
        }

        for (uint256 i = 1; i < TEST_ITERATIONS - 1; i++) {
            assertLe(trades[i].calculatedAmount, trades[i + 1].calculatedAmount);
            assertLe(trades[i].gasUsed, trades[i + 1].gasUsed);
        }
    }

    function testGetCapabilitiesRenzo(bytes32 pair, address t0, address t1)
        public
    {
        Capability[] memory res =
            adapter.getCapabilities(pair, IERC20(t0), IERC20(t1));

        assertEq(res.length, 3);
    }

    function testGetTokensRenzo() public {
        bytes32 pair = bytes32(0);
        IERC20[] memory tokens = adapter.getTokens(pair);

        assertGe(tokens.length, 2);
    }

    function testGetLimitsRenzo() public {
        bytes32 pair = bytes32(0);
        uint256[] memory limits = adapter.getLimits(pair, wBETH, ezETH);

        assertEq(limits.length, 2);
    }
}
