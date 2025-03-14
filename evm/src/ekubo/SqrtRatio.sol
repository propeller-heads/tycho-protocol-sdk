// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

import {ISwapAdapterTypes} from "src/interfaces/ISwapAdapterTypes.sol";

type SqrtRatioFloat is uint96;
type SqrtRatioFixed is uint192;

uint192 constant BIT_MASK = 0xc00000000000000000000000;
uint192 constant NOT_BIT_MASK = 0x3fffffffffffffffffffffff;

uint256 constant TWO_POW_128 = 1 << 128;

SqrtRatioFloat constant MIN_SQRT_RATIO = SqrtRatioFloat.wrap(0);
SqrtRatioFloat constant MAX_SQRT_RATIO = SqrtRatioFloat.wrap(type(uint96).max);

function toFixed(SqrtRatioFloat sqrtRatioFloat) pure returns (SqrtRatioFixed sqrtRatioFixed) {
    uint96 f = SqrtRatioFloat.unwrap(sqrtRatioFloat);

    sqrtRatioFixed = SqrtRatioFixed.wrap(
        (f & NOT_BIT_MASK) << (2 + ((f & BIT_MASK) >> 89))
    );
}

function toRational(SqrtRatioFixed sqrtRatioFixed) pure returns (ISwapAdapterTypes.Fraction memory price) {
    price = ISwapAdapterTypes.Fraction(
        SqrtRatioFixed.unwrap(sqrtRatioFixed),
        TWO_POW_128
    );
}

using {toFixed} for SqrtRatioFloat global;
using {toRational} for SqrtRatioFixed global;
