// SPDX-License-Identifier: MIT
pragma solidity 0.8.13;

/**
 * @notice This defines a slippage curve using the definition in '23-11-21 Fixed Nabla Spec II'
 */
interface ISlippageCurve {
    /**
     * @notice Calculates the output of the function Psi for input values b and l
     * @param _b the input value `b`
     * @param _l the input value `l`
     * @param _decimals the number of decimals used for _b and _l and also for the return value
     * @return _psi the value Psi(b,l)
     */
    function psi(
        uint256 _b,
        uint256 _l,
        uint8 _decimals
    ) external view returns (uint256 _psi);

    /**
     * @notice Given b,l and B >= Psi(b, l), this function determines the unique t>=0
        such that Psi(b+t, l+t) = B
     * @param _b the input value `b`
     * @param _l the input value `l`
     * @param _capitalB the input value `B`
     * @param _decimals the number of decimals used for _b, _l, _capitalB and also for the return value
     * @return _t the number t
     */
    function inverseDiagonal(
        uint256 _b,
        uint256 _l,
        uint256 _capitalB,
        uint8 _decimals
    ) external view returns (uint256 _t);

    /**
     * @notice Given b,l and B >= Psi(b, l), this function determines the unique t>=0
        such that Psi(b+t, l) = B
     * @param _b the input value `b`
     * @param _l the input value `l`
     * @param _capitalB the input value `B`
     * @param _decimals the number of decimals used for _b, _l, _capitalB and also for the return value
     * @return _t the number t
     */
    function inverseHorizontal(
        uint256 _b,
        uint256 _l,
        uint256 _capitalB,
        uint8 _decimals
    ) external view returns (uint256 _t);
}
