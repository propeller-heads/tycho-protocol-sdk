// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "src/interfaces/ISwapAdapterTypes.sol";
import "src/inception/InceptionSwapAdapter.sol";
import "src/libraries/FractionMath.sol";

/// @title InceptionStEthSwapAdapterTest
contract InceptionSwapAdapterTest is Test, ISwapAdapterTypes {
    using FractionMath for Fraction;
    using SafeCast160 for uint256;

    struct AdapterData {
        InceptionSwapAdapter adapter;
        address token;
        address inToken;
        bytes32 pair;
    }

    AdapterData[] adapters;

    address constant WETH = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;

    uint256 constant TEST_ITERATIONS = 100;

    function setUp() public {
        uint256 forkBlock = 20000000;
        vm.createSelectFork(vm.rpcUrl("mainnet"), forkBlock);

        adapters.push(AdapterData(
            new InceptionSwapAdapter(IInceptionVault(0x814CC6B8fd2555845541FB843f37418b05977d8d), 100, 1e23),
            0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84,
            0x7FA768E035F956c41d6aeaa3Bd857e7E5141CAd5,
            "insteth"
        ));
        adapters.push(AdapterData(
            new InceptionSwapAdapter(IInceptionVault(0x1Aa53BC4Beb82aDf7f5EDEE9e3bBF3434aD59F12), 100, 1e23),
            0xae78736Cd615f374D3085123A210448E74Fc6393,
            0x80d69e79258FE9D056c822461c4eb0B4ca8802E2,
            "inreth"
        ));
        adapters.push(AdapterData(
            new InceptionSwapAdapter(IInceptionVault(0x4878F636A9Aa314B776Ac51A25021C44CAF86bEd), 100, 1e23),
            0x856c4Efb76C1D1AE02e20CEB03A2A6a08b0b8dC3,
            0x9181f633E9B9F15A32d5e37094F4C93b333e0E92,
            "inoeth"
        ));
        //~~~ ratio problem~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // adapters.push(AdapterData(
        //     new InceptionSwapAdapter(IInceptionVault(0xA9F8c770661BeE8DF2D026edB1Cb6FF763C780FF), 100, 1e23),
        //     0xf1C9acDc66974dFB6dEcB12aA385b9cD01190E38,
        //     0xfD07fD5EBEa6F24888a397997E262179Bf494336,
        //     "inoseth"
        // ));
        // adapters.push(AdapterData(
        //     new InceptionSwapAdapter(IInceptionVault(0x36B429439AB227fAB170A4dFb3321741c8815e55), 100, 1e23),
        //     0xe05A08226c49b636ACf99c40Da8DC6aF83CE5bB3,
        //     0xfa2629B9cF3998D52726994E0FcdB750224D8B9D,
        //     "inankreth"
        // ));
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        adapters.push(AdapterData(
            new InceptionSwapAdapter(IInceptionVault(0xfE715358368416E01d3A961D3a037b7359735d5e), 100, 1e23),
            0xBe9895146f7AF43049ca1c1AE358B0541Ea49704,
            0xBf19Eead55a6B100667f04F8FBC5371E03E8ab2E,
            "incbeth"
        ));
        adapters.push(AdapterData(
            new InceptionSwapAdapter(IInceptionVault(0xC0660932C5dCaD4A1409b7975d147203B1e9A2B6), 100, 1e23),
            0xa2E3356610840701BDf5611a53974510Ae27E2e1,
            0xDA9B11Cd701e10C2Ec1a284f80820eDD128c5246,
            "inwbeth"
        ));
        adapters.push(AdapterData(
            new InceptionSwapAdapter(IInceptionVault(0xc4181dC7BB31453C4A48689ce0CBe975e495321c), 100, 1e23),
            0xf951E335afb289353dc249e82926178EaC7DEd78,
            0xC3ADe5aCe1bBb033CcAE8177C12Ecbfa16bD6A9D,
            "insweth"
        ));
        adapters.push(AdapterData(
            new InceptionSwapAdapter(IInceptionVault(0x90E80E25ABDB6205B08DeBa29a87f7eb039023C2), 100, 1e23),
            0xA35b1B31Ce002FBF2058D22F30f95D405200A15b,
            0x57a5a0567187FF4A8dcC1A9bBa86155E355878F2,
            "inethx"
        ));
        adapters.push(AdapterData(
            new InceptionSwapAdapter(IInceptionVault(0x295234B7E370a5Db2D2447aCA83bc7448f151161), 100, 1e23),
            0xac3E018457B222d93114458476f3E3416Abbe38F,
            0x668308d77be3533c909a692302Cb4D135Bf8041C,
            "insfrxeth"
        ));
        adapters.push(AdapterData(
            new InceptionSwapAdapter(IInceptionVault(0xd0ee89d82183D7Ddaef14C6b4fC0AA742F426355), 100, 1e23),
            0xd5F7838F5C461fefF7FE49ea5ebaF7728bB0ADfa,
            0xeCf3672A6d2147E2A77f07069Fb48d8Cf6F6Fbf9,
            "inmeth"
        ));
        adapters.push(AdapterData(
            new InceptionSwapAdapter(IInceptionVault(0x6E17a8b5D33e6DBdB9fC61d758BF554b6AD93322), 100, 1e19),
            0x8c1BEd5b9a0928467c9B1341Da1D7BD5e10b6549,
            0x94B888E11a9E960A9c3B3528EB6aC807B27Ca62E,
            "inlseth"
        ));
    }

    function testInceptionPriceFuzz(uint256 amount0, uint256 amount1) public view {
        uint256[] memory amounts = new uint256[](2);
        amounts[0] = amount0;
        amounts[1] = amount1;

        for (uint256 i = 0; i < adapters.length; i++) {
            AdapterData storage adapterData = adapters[i];
            Fraction[] memory prices = adapterData.adapter.price(adapterData.pair, adapterData.token, adapterData.inToken, amounts);

            for (uint256 j = 0; j < prices.length; j++) {
                assertGe(prices[j].numerator, 0);
                assertGe(prices[j].denominator, 0);
            }
        }
    }


    function testInceptionSwapFuzz(uint256 specifiedAmount, bool) public {
        for (uint8 i = 0; i < adapters.length; i++) {
            AdapterData storage adapterData = adapters[i];

            uint256[] memory limits = adapterData.adapter.getLimits(adapterData.pair, adapterData.token, adapterData.inToken);
            vm.assume(specifiedAmount > limits[0]);
            vm.assume(specifiedAmount < limits[1]);

            if (adapterData.pair == "insteth") {
                IStEth(adapterData.token).submit{value: specifiedAmount}(address(0));
                IERC20(adapterData.token).transfer(address(adapterData.adapter), specifiedAmount);
            } else if (adapterData.pair == "inoeth") {
                IOEthVault ovault = IOEthVault(0x39254033945AA2E4809Cc2977E7087BEE48bd7Ab);
                deal(WETH, address(this), specifiedAmount);
                IERC20(WETH).approve(address(ovault), specifiedAmount);
                ovault.mint(WETH, specifiedAmount, 0);
                IERC20(adapterData.token).transfer(address(adapterData.adapter), specifiedAmount);
            } else if (adapterData.pair == "inlseth") {
                uniswapSwapExactOut(specifiedAmount, i, 500);
                IERC20(adapterData.token).transfer(address(adapterData.adapter), specifiedAmount);
            } else {
                deal(adapterData.token, address(adapterData.adapter), specifiedAmount);
            }

            uint256 tokenBalanceBefore = IERC20(adapterData.token).balanceOf(address(adapterData.adapter));
            uint256 inTokenBalanceBefore = IERC20(adapterData.inToken).balanceOf(address(adapterData.adapter));

            Trade memory trade = adapterData.adapter.swap(adapterData.pair, adapterData.token, adapterData.inToken, OrderSide.Sell, specifiedAmount);

            if (trade.calculatedAmount > 0) {
                assertApproxEqAbs(
                    specifiedAmount,
                    tokenBalanceBefore - IERC20(adapterData.token).balanceOf(address(adapterData.adapter)),
                    2
                );
                assertApproxEqAbs(
                    trade.calculatedAmount,
                    IERC20(adapterData.inToken).balanceOf(address(adapterData.adapter)) - inTokenBalanceBefore,
                    2
                );
                assertLe(trade.gasUsed, 260000);
            }
        }
    }

    function uniswapSwapExactOut(uint256 output, uint8 adapterIndex, uint24 fee) public {
          uint256 input = 2 * output;
          AdapterData storage adapterData = adapters[adapterIndex];

          deal(WETH, address(this), input);
          address router = 0x3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD;
          address permit2 = 0x000000000022D473030F116dDEE9F6B43aC78BA3;

          IERC20(WETH).approve(permit2, input);
          IPermit2(permit2).approve(
            WETH,
            address(router),
            input.toUint160(),
            uint48(block.timestamp)
          );

          bytes memory commands = abi.encodePacked(bytes1(uint8(0x01)));
          bytes memory path = abi.encodePacked(adapterData.token, fee, WETH);

          bytes[] memory inputs = new bytes[](1);
          inputs[0] = abi.encode(address(this), output, input, path, true);

          IUniswapUniversalRouter(router).execute(commands, inputs);
      }
}

library SafeCast160 {
  error UnsafeCast();

  /// @notice Safely casts uint256 to uint160
  /// @param value The uint256 to be cast
  function toUint160(uint256 value) internal pure returns (uint160) {
    if (value > type(uint160).max) revert UnsafeCast();
    return uint160(value);
  }
}

interface IStEth {
    function submit(address) external payable;
}

interface IOEthVault {
    function mint(address, uint256, uint256) external;
}

interface IUniswapUniversalRouter {
    function execute(bytes calldata, bytes[] calldata) external payable;
}

interface IPermit2 {
  function approve(
    address token,
    address spender,
    uint160 amount,
    uint48 expiration
  ) external;
}
