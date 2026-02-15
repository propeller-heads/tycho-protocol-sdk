// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "forge-std/StdStorage.sol";
import "forge-std/StdCheats.sol";

import "./AdapterTest.sol";
import "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/libraries/FractionMath.sol";

import {AmpleforthSwapAdapter} from "src/ampleforth/AmpleforthSwapAdapter.sol";

contract AmpleforthSwapAdapterTest is AdapterTest {
    using FractionMath for Fraction;
    using stdStorage for StdStorage;

    // -----------------------------------------
    // Addresses for testing
    // -----------------------------------------
    /// @notice AMPL token address (proxy).
    address constant AMPL = 0xD46bA6D942050d489DBd938a2C909A5d5039A161;
    /// @notice WAMPL token address (wrapped AMPL).
    address constant WAMPL = 0xEDB171C18cE90B633DB442f2A6F72874093b49Ef;
    /// @notice SPOT token address (senior AMPL perpetual).
    address constant SPOT = 0xC1f33e0cf7e40a67375007104B929E49a581bafE;
    /// @notice STAMPL contract address (junior AMPL perpetual and swap pool).
    address constant STAMPL = 0x82A91a0D599A45d8E9Af781D67f695d7C72869Bd;
    /// @notice USDC token address.
    address constant USDC = 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48;
    /// @notice BillBroker contract address (SPOT/USDC swap pool).
    address constant BILL_BROKER = 0xA088Aef966CAD7fE0B38e28c2E07590127Ab4ccB;
    /// @notice SPOT fee policy contract address.
    address constant SPOT_FEE_POLICY =
        0xE22977381506bF094CB3ed50CB8834E358F7ef6c;

    // -----------------------------------------
    // AMPL GON Balance storage slot
    // -----------------------------------------
    /// @notice The storage slot index for _gonBalances mapping in AMPL (found
    /// via inspection).
    uint256 internal constant GON_BALANCES_SLOT_INDEX = 158;

    // -----------------------------------------
    // Pool IDs (AmpleforthSwapAdapter uses the
    // address cast into bytes32 as the poolId)
    // -----------------------------------------
    /// @notice WAMPL poolId cast from WAMPL address.
    bytes32 constant WAMPL_POOLID = bytes32(bytes20(WAMPL));
    /// @notice STAMPL poolId cast from STAMPL address.
    bytes32 constant STAMPL_POOLID = bytes32(bytes20(STAMPL));
    /// @notice BillBroker poolId cast from BILL_BROKER address.
    bytes32 constant BILL_BROKER_POOLID = bytes32(bytes20(BILL_BROKER));

    // -----------------------------------------
    // The adapter under test
    // -----------------------------------------
    /// @notice The AmpleforthSwapAdapter instance used throughout these tests.
    AmpleforthSwapAdapter public adapter;

    // -----------------------------------------
    // Test parameters
    // -----------------------------------------
    /// @notice The number of iterations used in certain fuzz or loop-based
    /// tests.
    uint256 constant TEST_ITERATIONS = 50;

    /**
     * @notice Test setup, run before each test.
     * @dev Creates a mainnet fork at a specific block, then deploys an
     * AmpleforthSwapAdapter
     *      instance and labels addresses for easier debugging. Also mocks
     * certain storage
     *      values for the SPOT_FEE_POLICY contract to limit stAMPL swaps.
     */
    function setUp() public {
        // We create a mainnet fork
        vm.createSelectFork(vm.rpcUrl("mainnet"), 21588855);

        // Deploy your adapter
        adapter = new AmpleforthSwapAdapter();

        // Label for readability in Foundry traces
        vm.label(AMPL, "AMPL");
        vm.label(WAMPL, "WAMPL");
        vm.label(SPOT, "SPOT");
        vm.label(STAMPL, "STAMPL");
        vm.label(USDC, "USDC");
        vm.label(BILL_BROKER, "BILL_BROKER");
        vm.label(SPOT_FEE_POLICY, "SPOT_FEE_POLICY");
        vm.label(address(adapter), "AmpleforthSwapAdapter");

        // Mocking STAMPL DR bounds, which limits swapping
        vm.store(
            SPOT_FEE_POLICY, bytes32(uint256(102)), bytes32(uint256(50000000))
        );
        vm.store(
            SPOT_FEE_POLICY, bytes32(uint256(103)), bytes32(uint256(1000000000))
        );
    }

    /**
     * @notice Tests price quotes for WAMPL pool (AMPL <-> WAMPL).
     * @dev Asserts that computed price values match expected references for
     * forward
     *      (AMPL -> WAMPL) and reverse (WAMPL -> AMPL) direction in multiple
     * amounts.
     */
    function testWAMPLPoolPrice() public {
        // -----------------------------------------
        // current supply = 172,488,857.573770079
        // total_wampl = 10,000,000.000000000000000000 (10m e18)
        // p = 10000000000000000000000000 / 172488857573770079 = 57974759
        // p' = 172488857573770079 * 1e18 / 10000000000000000000000000  =
        // 17248885757

        uint256[] memory amounts0 = new uint256[](3);
        amounts0[0] = 15e9;
        amounts0[1] = 5e7;
        amounts0[2] = 100e9;

        // AMPL -> WAMPL
        Fraction[] memory prices0 =
            adapter.price(WAMPL_POOLID, AMPL, WAMPL, amounts0);

        assertEq(
            prices0[0].numerator / prices0[0].denominator,
            57974759,
            "Unexpected price"
        );
        assertEq(
            prices0[1].numerator / prices0[1].denominator,
            57974759,
            "Unexpected price"
        );
        assertEq(
            prices0[2].numerator / prices0[2].denominator,
            57974759,
            "Unexpected price"
        );

        uint256[] memory amounts1 = new uint256[](3);
        amounts1[0] = 15e18;
        amounts1[1] = 5e17;
        amounts1[2] = 100e18;

        // WAMPL -> AMPL
        Fraction[] memory prices1 =
            adapter.price(WAMPL_POOLID, WAMPL, AMPL, amounts1);

        assertEq(
            prices1[0].numerator * 1e18 / prices1[0].denominator,
            17248885757,
            "Unexpected price"
        );
        assertEq(
            prices1[1].numerator * 1e18 / prices1[1].denominator,
            17248885756,
            "Unexpected price"
        );
        assertEq(
            prices1[2].numerator * 1e18 / prices1[2].denominator,
            17248885757,
            "Unexpected price"
        );
    }

    /**
     * @notice Fuzz test for selling AMPL into WAMPL using a range of amounts.
     * @param specifiedAmount The amount of AMPL to sell.
     */
    function testFuzzAMPL2WAMPLSwap(uint256 specifiedAmount) public {
        uint256[] memory limits = adapter.getLimits(WAMPL_POOLID, AMPL, WAMPL);
        vm.assume(specifiedAmount < limits[0]);

        _dealAMPL(address(this), specifiedAmount);
        IERC20(AMPL).approve(address(adapter), specifiedAmount);

        uint256 amplBefore = IERC20(AMPL).balanceOf(address(this));
        uint256 wamplBefore = IERC20(WAMPL).balanceOf(address(this));

        Trade memory trade = adapter.swap(
            WAMPL_POOLID, AMPL, WAMPL, OrderSide.Sell, specifiedAmount
        );

        uint256 amplSpent = amplBefore - IERC20(AMPL).balanceOf(address(this));
        uint256 wamplGained =
            IERC20(WAMPL).balanceOf(address(this)) - wamplBefore;
        assertEq(amplSpent, specifiedAmount, "Wrong AMPL spent");
        assertEq(wamplGained, trade.calculatedAmount, "Wrong WAMPL gained");
    }

    /**
     * @notice Fuzz test for selling WAMPL into AMPL using a range of amounts.
     * @param specifiedAmount The amount of WAMPL to sell.
     */
    function testFuzzWAMPL2AMPLSwap(uint256 specifiedAmount) public {
        uint256[] memory limits = adapter.getLimits(WAMPL_POOLID, WAMPL, AMPL);
        vm.assume(specifiedAmount < limits[0]);

        deal(WAMPL, address(this), specifiedAmount + 1e18);
        IERC20(WAMPL).approve(address(adapter), specifiedAmount);

        uint256 wamplBefore = IERC20(WAMPL).balanceOf(address(this));
        uint256 amplBefore = IERC20(AMPL).balanceOf(address(this));

        Trade memory trade = adapter.swap(
            WAMPL_POOLID, WAMPL, AMPL, OrderSide.Sell, specifiedAmount
        );

        uint256 wamplSpent =
            wamplBefore - IERC20(WAMPL).balanceOf(address(this));
        uint256 amplGained = IERC20(AMPL).balanceOf(address(this)) - amplBefore;
        assertEq(wamplSpent, specifiedAmount, "Wrong WAMPL spent");
        assertEq(amplGained, trade.calculatedAmount, "Wrong AMPL gained");
    }

    /**
     * @notice Tests price quotes for STAMPL pool (AMPL <-> SPOT).
     * @dev Verifies forward and reverse quotes against expected references.
     */
    function testSTAMPLPoolPrice() public {
        // exchange_rate: 1 spot <=> ~1.16 AMPL

        uint256[] memory amounts0 = new uint256[](3);
        amounts0[0] = 15e9;
        amounts0[1] = 5e7;
        amounts0[2] = 100e9;

        Fraction[] memory prices0 =
            adapter.price(STAMPL_POOLID, AMPL, SPOT, amounts0);

        assertEq(
            prices0[0].numerator * 1e9 / prices0[0].denominator,
            838355956,
            "Unexpected price"
        );
        assertEq(
            prices0[1].numerator * 1e9 / prices0[1].denominator,
            838355940,
            "Unexpected price"
        );
        assertEq(
            prices0[2].numerator * 1e9 / prices0[2].denominator,
            838355956,
            "Unexpected price"
        );

        uint256[] memory amounts1 = new uint256[](3);
        amounts1[0] = 15e9;
        amounts1[1] = 5e7;
        amounts1[2] = 100e9;

        Fraction[] memory prices1 =
            adapter.price(STAMPL_POOLID, SPOT, AMPL, amounts1);

        assertEq(
            prices1[0].numerator * 1e9 / prices1[0].denominator,
            1046691436,
            "Unexpected price"
        );
        assertEq(
            prices1[1].numerator * 1e9 / prices1[1].denominator,
            1046691420,
            "Unexpected price"
        );
        assertEq(
            prices1[2].numerator * 1e9 / prices1[2].denominator,
            1046691436,
            "Unexpected price"
        );
    }

    /**
     * @notice Fuzz test for selling AMPL into SPOT via the STAMPL pool using a
     * range of amounts.
     * @param specifiedAmount The amount of AMPL to sell.
     */
    function testFuzzAMPL2SPOTSwap(uint256 specifiedAmount) public {
        // Retrieve the swap limits for AMPL -> SPOT on the STAMPL pool
        uint256[] memory limits = adapter.getLimits(STAMPL_POOLID, AMPL, SPOT);

        // Filter out any specifiedAmount that exceeds the sell limit
        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 1e9);

        // Deal AMPL to this test contract so we can sell it
        _dealAMPL(address(this), specifiedAmount);

        // Approve the adapter to pull the specifiedAmount of AMPL
        IERC20(AMPL).approve(address(adapter), specifiedAmount);

        // Track balances before swap
        uint256 amplBefore = IERC20(AMPL).balanceOf(address(this));
        uint256 spotBefore = IERC20(SPOT).balanceOf(address(this));

        // Perform the swap via STAMPL pool
        Trade memory trade = adapter.swap(
            STAMPL_POOLID, AMPL, SPOT, OrderSide.Sell, specifiedAmount
        );

        // Calculate spent/received amounts
        uint256 amplSpent = amplBefore - IERC20(AMPL).balanceOf(address(this));
        uint256 spotGained = IERC20(SPOT).balanceOf(address(this)) - spotBefore;

        // Assert correctness of spent/received vs. trade.calculatedAmount
        assertEq(amplSpent, specifiedAmount, "Wrong AMPL spent");
        assertEq(spotGained, trade.calculatedAmount, "Wrong SPOT gained");
    }

    /**
     * @notice Fuzz test for selling SPOT into AMPL via the STAMPL pool using a
     * range of amounts.
     * @param specifiedAmount The amount of SPOT to sell.
     */
    function testFuzzSPOT2AMPLSwap(uint256 specifiedAmount) public {
        // Retrieve the swap limits for SPOT -> AMPL on the STAMPL pool
        uint256[] memory limits = adapter.getLimits(STAMPL_POOLID, SPOT, AMPL);

        // Filter out any specifiedAmount that exceeds the sell limit
        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 1e9);

        // Deal SPOT to this test contract so we can sell it
        deal(SPOT, address(this), specifiedAmount);

        // Approve the adapter to pull the specifiedAmount of SPOT
        IERC20(SPOT).approve(address(adapter), specifiedAmount);

        // Track balances before swap
        uint256 spotBefore = IERC20(SPOT).balanceOf(address(this));
        uint256 amplBefore = IERC20(AMPL).balanceOf(address(this));

        // Perform the swap via STAMPL pool
        Trade memory trade = adapter.swap(
            STAMPL_POOLID, SPOT, AMPL, OrderSide.Sell, specifiedAmount
        );

        // Calculate spent/received amounts
        uint256 spotSpent = spotBefore - IERC20(SPOT).balanceOf(address(this));
        uint256 amplGained = IERC20(AMPL).balanceOf(address(this)) - amplBefore;

        // Assert correctness of spent/received vs. trade.calculatedAmount
        assertEq(spotSpent, specifiedAmount, "Wrong SPOT spent");
        assertEq(amplGained, trade.calculatedAmount, "Wrong AMPL gained");
    }

    /**
     * @notice Tests price quotes for BillBroker pool (SPOT <-> USDC).
     * @dev Demonstrates multiple amounts to ensure correct pricing at different
     * sizes.
     */
    function testBillBrokerPoolPrice() public {
        // exchange_rate: 1 spot <=> ~1.16 AMPL and 1 AMPL target = 1.19 USD

        uint256[] memory amounts0 = new uint256[](6);
        amounts0[0] = 15e9;
        amounts0[1] = 5e7;
        amounts0[2] = 100e9;
        amounts0[3] = 3403e9;
        amounts0[4] = 25985e9;
        amounts0[5] = 199999e9;

        Fraction[] memory prices0 =
            adapter.price(BILL_BROKER_POOLID, SPOT, USDC, amounts0);

        assertEq(
            prices0[0].numerator * 1e9 / prices0[0].denominator,
            1284575,
            "Unexpected price"
        );
        assertEq(
            prices0[1].numerator * 1e9 / prices0[1].denominator,
            1284580,
            "Unexpected price"
        );
        assertEq(
            prices0[2].numerator * 1e9 / prices0[2].denominator,
            1284548,
            "Unexpected price"
        );
        assertEq(
            prices0[3].numerator * 1e9 / prices0[3].denominator,
            1283504,
            "Unexpected price"
        );
        assertEq(
            prices0[4].numerator * 1e9 / prices0[4].denominator,
            1276489,
            "Unexpected price"
        );
        assertEq(
            prices0[5].numerator * 1e9 / prices0[5].denominator,
            1228548,
            "Unexpected price"
        );

        uint256[] memory amounts1 = new uint256[](7);
        amounts1[0] = 15e6;
        amounts1[1] = 5e5;
        amounts1[2] = 100e6;
        amounts1[3] = 3403e6;
        amounts1[4] = 25985e6;
        amounts1[5] = 199999e6;
        amounts1[6] = 599999e6;

        Fraction[] memory prices1 =
            adapter.price(BILL_BROKER_POOLID, USDC, SPOT, amounts1);

        assertEq(
            prices1[0].numerator * 1e6 / prices1[0].denominator,
            699483265,
            "Unexpected price"
        );
        assertEq(
            prices1[1].numerator * 1e6 / prices1[1].denominator,
            699482550,
            "Unexpected price"
        );
        assertEq(
            prices1[2].numerator * 1e6 / prices1[2].denominator,
            699483271,
            "Unexpected price"
        );
        assertEq(
            prices1[3].numerator * 1e6 / prices1[3].denominator,
            699483278,
            "Unexpected price"
        );
        assertEq(
            prices1[4].numerator * 1e6 / prices1[4].denominator,
            699483278,
            "Unexpected price"
        );
        assertEq(
            prices1[5].numerator * 1e6 / prices1[5].denominator,
            699483278,
            "Unexpected price"
        );
        assertEq(
            prices1[6].numerator * 1e6 / prices1[6].denominator,
            699483278,
            "Unexpected price"
        );
    }

    /**
     * @notice Fuzz test for selling SPOT into USDC via the BILL_BROKER pool
     * using a range of amounts.
     * @param specifiedAmount The amount of SPOT to sell.
     */
    function testFuzzSPOT2USDCSwap(uint256 specifiedAmount) public {
        // Retrieve the swap limits for SPOT -> USDC on the BILL_BROKER pool
        uint256[] memory limits =
            adapter.getLimits(BILL_BROKER_POOLID, SPOT, USDC);

        // Filter out any specifiedAmount that exceeds the sell limit
        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 1e9);

        // Deal SPOT to this test contract so we can sell it
        deal(SPOT, address(this), specifiedAmount);

        // Approve the adapter to pull the specifiedAmount of SPOT
        IERC20(SPOT).approve(address(adapter), specifiedAmount);

        // Track balances before swap
        uint256 spotBefore = IERC20(SPOT).balanceOf(address(this));
        uint256 usdcBefore = IERC20(USDC).balanceOf(address(this));

        // Perform the swap via BILL_BROKER pool
        Trade memory trade = adapter.swap(
            BILL_BROKER_POOLID, SPOT, USDC, OrderSide.Sell, specifiedAmount
        );

        // Calculate spent/received amounts
        uint256 spotSpent = spotBefore - IERC20(SPOT).balanceOf(address(this));
        uint256 usdcGained = IERC20(USDC).balanceOf(address(this)) - usdcBefore;

        // Assert correctness of spent/received vs. trade.calculatedAmount
        assertEq(spotSpent, specifiedAmount, "Wrong SPOT spent");
        assertEq(usdcGained, trade.calculatedAmount, "Wrong USDC gained");
    }

    /**
     * @notice Fuzz test for selling USDC into SPOT via the BILL_BROKER pool
     * using a range of amounts.
     * @param specifiedAmount The amount of USDC to sell.
     */
    function testFuzzUSDC2SPOTSwap(uint256 specifiedAmount) public {
        // Retrieve the swap limits for USDC -> SPOT on the BILL_BROKER pool
        uint256[] memory limits =
            adapter.getLimits(BILL_BROKER_POOLID, USDC, SPOT);

        // Filter out any specifiedAmount that exceeds the sell limit
        vm.assume(specifiedAmount < limits[0] && specifiedAmount > 1e6);

        // Deal USDC to this test contract so we can sell it
        deal(USDC, address(this), specifiedAmount);

        // Approve the adapter to pull the specifiedAmount of USDC
        IERC20(USDC).approve(address(adapter), specifiedAmount);

        // Track balances before swap
        uint256 usdcBefore = IERC20(USDC).balanceOf(address(this));
        uint256 spotBefore = IERC20(SPOT).balanceOf(address(this));

        // Perform the swap via BILL_BROKER pool
        Trade memory trade = adapter.swap(
            BILL_BROKER_POOLID, USDC, SPOT, OrderSide.Sell, specifiedAmount
        );

        // Calculate spent/received amounts
        uint256 usdcSpent = usdcBefore - IERC20(USDC).balanceOf(address(this));
        uint256 spotGained = IERC20(SPOT).balanceOf(address(this)) - spotBefore;

        // Assert correctness of spent/received vs. trade.calculatedAmount
        assertEq(usdcSpent, specifiedAmount, "Wrong USDC spent");
        assertEq(spotGained, trade.calculatedAmount, "Wrong SPOT gained");
    }

    /**
     * @notice Tests all getLimits calls for each recognized pool and direction.
     * @dev Asserts the returned limits match expected reference values.
     */
    function testGetLimits() public {
        uint256[] memory limits0 = adapter.getLimits(WAMPL_POOLID, AMPL, WAMPL);
        assertEq(limits0[0], 1707955334639347, "Unexpected limixt");
        assertEq(limits0[1], 99018299423131631370840, "Unexpected limit");

        uint256[] memory limits1 = adapter.getLimits(WAMPL_POOLID, WAMPL, AMPL);
        assertEq(limits1[0], 990182994231647527093596, "Unexpected limit");
        assertEq(limits1[1], 17079553346399185, "Unexpected limit");

        uint256[] memory limits2 = adapter.getLimits(STAMPL_POOLID, AMPL, SPOT);
        assertEq(limits2[0], 149695390960572, "Unexpected limit");
        assertEq(limits2[1], 125498022655029, "Unexpected limit");

        uint256[] memory limits3 = adapter.getLimits(STAMPL_POOLID, SPOT, AMPL);
        assertEq(limits3[0], 80957539013856, "Unexpected limit");
        assertEq(limits3[1], 84737562778306, "Unexpected limit");

        uint256[] memory limits4 =
            adapter.getLimits(BILL_BROKER_POOLID, SPOT, USDC);
        assertEq(limits4[0], 24146542215900, "Unexpected limit");
        assertEq(limits4[1], 34520542474, "Unexpected limit");

        uint256[] memory limits5 =
            adapter.getLimits(BILL_BROKER_POOLID, USDC, SPOT);
        assertEq(limits5[0], 190612139597, "Unexpected limit");
        assertEq(limits5[1], 153676512329221, "Unexpected limit");
    }

    /**
     * @notice Tests getCapabilities for each recognized pool/direction to
     * ensure correct capabilities.
     * @dev Asserts that the returned array of capabilities matches what we
     * expect from the adapter.
     */
    function testGetCapabilities() public view {
        Capability[] memory res0 =
            adapter.getCapabilities(WAMPL_POOLID, AMPL, WAMPL);
        assertEq(
            uint256(res0[0]),
            uint256(Capability.SellOrder),
            "Unexpected capability"
        );
        assertEq(
            uint256(res0[1]),
            uint256(Capability.PriceFunction),
            "Unexpected capability"
        );
        assertEq(
            uint256(res0[2]),
            uint256(Capability.ConstantPrice),
            "Unexpected capability"
        );
        assertEq(
            uint256(res0[3]),
            uint256(Capability.HardLimits),
            "Unexpected capability"
        );

        Capability[] memory res1 =
            adapter.getCapabilities(STAMPL_POOLID, AMPL, SPOT);
        assertEq(
            uint256(res1[0]),
            uint256(Capability.SellOrder),
            "Unexpected capability"
        );
        assertEq(
            uint256(res1[1]),
            uint256(Capability.PriceFunction),
            "Unexpected capability"
        );
        assertEq(
            uint256(res1[2]),
            uint256(Capability.HardLimits),
            "Unexpected capability"
        );

        Capability[] memory res2 =
            adapter.getCapabilities(BILL_BROKER_POOLID, SPOT, USDC);
        assertEq(
            uint256(res2[0]),
            uint256(Capability.SellOrder),
            "Unexpected capability"
        );
        assertEq(
            uint256(res2[1]),
            uint256(Capability.PriceFunction),
            "Unexpected capability"
        );
        assertEq(
            uint256(res2[2]),
            uint256(Capability.HardLimits),
            "Unexpected capability"
        );
    }

    /**
     * @notice Test getTokens for each poolId, ensuring correct tokens are
     * returned.
     * @dev Verifies that WAMPL_POOLID, STAMPL_POOLID, and BILL_BROKER_POOLID
     * match the
     *      expected underlying and wrapper tokens.
     */
    function testGetTokens() public view {
        address[] memory wmplTokens = adapter.getTokens(WAMPL_POOLID);
        assertEq(wmplTokens.length, 2, "WAMPL pool should have 2 tokens");
        assertEq(wmplTokens[0], AMPL, "Expected AMPL as the underlying");
        assertEq(wmplTokens[1], WAMPL, "Expected WAMPL as the wrapper");

        address[] memory stamplTokens = adapter.getTokens(STAMPL_POOLID);
        assertEq(stamplTokens.length, 2, "STAMPL pool should have 2 tokens");
        assertEq(stamplTokens[0], AMPL, "Expected AMPL for stAMPL pool");
        assertEq(stamplTokens[1], SPOT, "Expected SPOT for stAMPL pool");

        address[] memory billBrokerTokens =
            adapter.getTokens(BILL_BROKER_POOLID);
        assertEq(
            billBrokerTokens.length, 2, "BillBroker pool should have 2 tokens"
        );
        assertEq(billBrokerTokens[0], SPOT, "Expected SPOT for BillBroker");
        assertEq(billBrokerTokens[1], USDC, "Expected USDC for BillBroker");
    }

    /**
     * @notice Test getPoolIds to ensure the adapter returns three recognized
     * pool IDs.
     * @dev Checks the array length and exact ordering of returned IDs.
     */
    function testGetPoolIds() public view {
        bytes32[] memory ids = adapter.getPoolIds(0, 3);
        assertEq(ids.length, 1, "Expected 3 pool IDs");
        assertEq(ids[0], WAMPL_POOLID, "IDs[0] mismatch");
    }

    // NOTE: These tests fail for constant price pools.
    // function testAmpleforthPoolsBehaviour() public {
    //     bytes32[] memory poolIds = new bytes32[](3);
    //     poolIds[0] = WAMPL_POOLID;
    //     poolIds[1] = STAMPL_POOLID;
    //     poolIds[2] = BILL_BROKER_POOLID;
    //     runPoolBehaviourTest(adapter, poolIds);
    // }

    /**
     * @dev Deal a new fragment-balance to `to` in AMPL, updating gonBalances.
     *      - `give` is the final fragment-level balance (the amount you'd
     * expect to see in `balanceOf(to)`).
     *      - This function calculates the gons per fragment from totalSupply
     * and scaledTotalSupply,
     *        then adjusts the recipient's gon balance in storage accordingly.
     * @param to The address receiving the new AMPL balance.
     * @param give The fragment-level balance to allocate to `to`.
     */
    function _dealAMPL(address to, uint256 give) private {
        // -----------------------------------------
        // 1. Read old gonBalance(USER)
        //    _gonBalances[USER] => keccak256(abi.encode(USER, 14))
        // -----------------------------------------
        bytes32 gonSlot = keccak256(abi.encode(to, GON_BALANCES_SLOT_INDEX));
        bytes32 rawGons = vm.load(AMPL, gonSlot);
        uint256 oldGons = uint256(rawGons);

        // -----------------------------------------
        // 2. Update gonBalance to new value
        // -----------------------------------------
        (bool ok, bytes memory data) =
            AMPL.staticcall(abi.encodeWithSignature("totalSupply()"));
        require(ok, "AMPL deal: failed to read old totalSupply");
        uint256 totalSupply = abi.decode(data, (uint256));

        (ok, data) =
            AMPL.staticcall(abi.encodeWithSignature("scaledTotalSupply()"));
        require(ok, "AMPL deal: failed to read scaledTotalSupply");
        uint256 totalGons = abi.decode(data, (uint256));

        uint256 gonsPerFragment = totalGons / totalSupply;
        uint256 newGons = oldGons + (give * gonsPerFragment);
        vm.store(AMPL, gonSlot, bytes32(newGons));
    }
}
