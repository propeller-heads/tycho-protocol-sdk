// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.26;

import "./AdapterTest.sol";
import {BalancerV3SwapAdapter, IERC20, IVault, IBatchRouter} from "src/balancer-v3/BalancerV3SwapAdapter.sol";
import {ERC20} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";

import {FractionMath} from "src/libraries/FractionMath.sol";
import "forge-std/Test.sol";

contract BalancerV3SwapAdapterTest is AdapterTest, ERC20 {
    error NotStaticCall();
    error VaultQueriesDisabled();
    using FractionMath for Fraction;

    IVault constant balancerV3Vault =
        IVault(payable(0xBC582d2628FcD404254a1e12CB714967Ce428915));
    BalancerV3SwapAdapter adapter;
    IBatchRouter router =
        IBatchRouter(0x4232e5EEaA16Bcf483d93BEA469296B4EeF22503); // Batch router

    address constant DAI_USDT_POOL_ADDRESS =
        0xD320B050444aA50F24e6666e22A503f765E74263;
    address constant DAI = 0xB77EB1A70A96fDAAeB31DB1b42F2b8b5846b2613;
    address constant USDT = 0x6bF294B80C7d8Dc72DEE762af5d01260B756A051;

    uint256 constant TEST_ITERATIONS = 100;

    constructor() ERC20("", "") {
        
    }

    function setUp() public {
        uint256 forkBlock = 7249768;
        vm.createSelectFork(vm.rpcUrl("sepolia"), forkBlock);

        adapter = new BalancerV3SwapAdapter(
            payable(address(balancerV3Vault)),
            address(router)
        );

        vm.label(address(balancerV3Vault), "BalancerV3Vault");
        vm.label(address(router), "BalancerV3BatchRouter");
        vm.label(DAI, "DAI");
        vm.label(USDT, "USDT");
        vm.label(address(adapter), "BalancerV3SwapAdapter");
    }

    function testPriceFuzzBalancerV3(uint256 amount0, uint256 amount1) public {
        bytes32 pool = bytes32(bytes20(DAI_USDT_POOL_ADDRESS));
        uint256[] memory limits = adapter.getLimits(pool, USDT, DAI);

        vm.assume(amount0 < limits[0] && amount0 > 0);
        vm.assume(amount1 < limits[1] && amount1 > 0);

        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;

        __prankStaticCall();
        Fraction[] memory prices =
            adapter.price(pool, USDT, DAI, amounts);

        for (uint256 i = 0; i < prices.length; i++) {
            assertGt(prices[i].numerator, 0);
            assertGt(prices[i].denominator, 0);
        }
    }

    function testGetCapabilitiesBalancerV3(
        bytes32 pool,
        address t0,
        address t1
    ) public view {
        Capability[] memory res = adapter.getCapabilities(pool, t0, t1);

        assertGe(res.length, 4);
    }

    function testGetTokensBalancerV3() public view {
        address[] memory tokens = adapter.getTokens(bytes32(bytes20(DAI_USDT_POOL_ADDRESS)));
        assertGe(tokens.length, 2);
    }

    function testGetPoolIdsBalancerV3() public {
        vm.expectRevert(
            abi.encodeWithSelector(
                NotImplemented.selector,
                "BalancerV3SwapAdapter.getPoolIds"
            )
        );
        adapter.getPoolIds(100, 200);
    }

    function __prankStaticCall() internal {
        // Prank address 0x0 for both msg.sender and tx.origin (to identify as a staticcall).
        vm.prank(address(0), address(0));
    }
}
