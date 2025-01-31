// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@src/libraries/EfficientERC20.sol";
import "@interfaces/batch-swap-router/IBatchSwapRouterV1Structs.sol";

/**
 * @title ApprovalManagement for contracts
 * @author PropellerHeads Devs
 * @dev Allows a contract to tokens for trading on third party contracts.
 */
contract ApprovalManagement is IBatchSwapRouterV1Structs {
    using EfficientERC20 for IERC20;

    /**
     * @dev Set allowance to several addresses per token for transfer on behalf of this contract.
     */
    function _setApprovals(ExternalApproval[] calldata approvals) internal {
        for (uint256 i = 0; i < approvals.length; i++) {
            IERC20 token = approvals[i].token;
            uint256 allowance = approvals[i].allowance;
            for (uint256 j = 0; j < approvals[i].addresses.length; j++) {
                address beneficiary = approvals[i].addresses[j];
                token.safeApprove(beneficiary, allowance);
            }
        }
    }
}
