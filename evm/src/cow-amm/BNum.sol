// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.13;

/**
 * @title BNum
 * @notice Includes functions for arithmetic operations with fixed-point
 * numbers.
 * @dev The arithmetic operations are implemented with a precision of BONE.
 */
library BNumLib {
    /// @notice The unit of precision used in the calculations.
    uint256 public constant BONE = 10 ** 18;
    /// @notice The minimum base value for the bpow calculation.
    uint256 public constant MIN_BPOW_BASE = 1 wei;
    /// @notice The maximum base value for the bpow calculation.
    uint256 public constant MAX_BPOW_BASE = (2 * BONE) - 1 wei;
    /// @notice The precision of the bpow calculation.
    uint256 public constant BPOW_PRECISION = BONE / 10 ** 10;
    /**
     * @notice Thrown when an overflow is encountered inside the add function
     */
    error BNum_AddOverflow();

    /**
     * @notice Thrown when an underflow is encountered inside the sub function
     */
    error BNum_SubUnderflow();

    /**
     * @notice Thrown when an overflow is encountered inside the mul function
     */
    error BNum_MulOverflow();

    /**
     * @notice Thrown when attempting to divide by zero
     */
    error BNum_DivZero();

    /**
     * @notice Thrown when an internal error occurs inside div function
     */
    error BNum_DivInternal();

    /**
     * @notice Thrown when the base is too low in the bpow function
     */
    error BNum_BPowBaseTooLow();

    /**
     * @notice Thrown when the base is too high in the bpow function
     */
    error BNum_BPowBaseTooHigh();

    function btoi(uint256 a) internal pure returns (uint256) {
        unchecked {
            return a / BONE;
        }
    }

    function bfloor(uint256 a) internal pure returns (uint256) {
        unchecked {
            return btoi(a) * BONE;
        }
    }

    function badd(uint256 a, uint256 b) internal pure returns (uint256) {
        unchecked {
            uint256 c = a + b;
            if (c < a) {
                revert BNum_AddOverflow();
            }
            return c;
        }
    }

    function bsub(uint256 a, uint256 b) internal pure returns (uint256) {
        unchecked {
            (uint256 c, bool flag) = bsubSign(a, b);
            if (flag) {
                revert BNum_SubUnderflow();
            }
            return c;
        }
    }

    function bsubSign(uint256 a, uint256 b)
        internal
        pure
        returns (uint256, bool)
    {
        unchecked {
            if (a >= b) {
                return (a - b, false);
            } else {
                return (b - a, true);
            }
        }
    }

    function bmul(uint256 a, uint256 b) internal pure returns (uint256) {
        unchecked {
            uint256 c0 = a * b;
            if (a != 0 && c0 / a != b) {
                revert BNum_MulOverflow();
            }
            // NOTE: using >> 1 instead of / 2
            uint256 c1 = c0 + (BONE >> 1);
            if (c1 < c0) {
                revert BNum_MulOverflow();
            }
            uint256 c2 = c1 / BONE;
            return c2;
        }
    }

    function bdiv(uint256 a, uint256 b) internal pure returns (uint256) {
        unchecked {
            if (b == 0) {
                revert BNum_DivZero();
            }
            uint256 c0 = a * BONE;
            if (a != 0 && c0 / a != BONE) {
                revert BNum_DivInternal(); // bmul overflow
            }
            // NOTE: using >> 1 instead of / 2
            uint256 c1 = c0 + (b >> 1);
            if (c1 < c0) {
                revert BNum_DivInternal(); //  badd require
            }
            uint256 c2 = c1 / b;
            return c2;
        }
    }

    // DSMath.wpow
    function bpowi(uint256 a, uint256 n) internal pure returns (uint256) {
        unchecked {
            uint256 z = n % 2 != 0 ? a : BONE;

            for (n /= 2; n != 0; n /= 2) {
                a = bmul(a, a);

                if (n % 2 != 0) {
                    z = bmul(z, a);
                }
            }
            return z;
        }
    }

    // Compute b^(e.w) by splitting it into (b^e)*(b^0.w).
    // Use `bpowi` for `b^e` and `bpowK` for k iterations
    // of approximation of b^0.w
    function bpow(uint256 base, uint256 exp) internal pure returns (uint256) {
        unchecked {
            if (base < MIN_BPOW_BASE) {
                revert BNum_BPowBaseTooLow();
            }
            if (base > MAX_BPOW_BASE) {
                revert BNum_BPowBaseTooHigh();
            }

            uint256 whole = bfloor(exp);
            uint256 remain = bsub(exp, whole);

            uint256 wholePow = bpowi(base, btoi(whole));

            if (remain == 0) {
                return wholePow;
            }

            uint256 partialResult = bpowApprox(base, remain, BPOW_PRECISION);
            return bmul(wholePow, partialResult);
        }
    }

    function bpowApprox(uint256 base, uint256 exp, uint256 precision)
        internal
        pure
        returns (uint256)
    {
        unchecked {
            // term 0:
            uint256 a = exp;
            (uint256 x, bool xneg) = bsubSign(base, BONE);
            uint256 term = BONE;
            uint256 sum = term;
            bool negative = false;

            // term(k) = numer / denom
            //         = (product(a - i - 1, i=1-->k) * x^k) / (k!)
            // each iteration, multiply previous term by (a-(k-1)) * x / k
            // continue until term is less than precision
            for (uint256 i = 1; term >= precision; i++) {
                uint256 bigK = i * BONE;
                (uint256 c, bool cneg) = bsubSign(a, bsub(bigK, BONE));
                term = bmul(term, bmul(c, x));
                term = bdiv(term, bigK);
                if (term == 0) break;

                if (xneg) negative = !negative;
                if (cneg) negative = !negative;
                if (negative) {
                    sum = bsub(sum, term);
                } else {
                    sum = badd(sum, term);
                }
            }

            return sum;
        }
    }
}
