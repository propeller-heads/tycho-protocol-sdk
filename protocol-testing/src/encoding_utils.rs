use alloy::{primitives::Keccak256, sol_types::SolValue};
use num_bigint::BigUint;
use tycho_common::Bytes;
use tycho_simulation::{
    evm::protocol::u256_num::biguint_to_u256,
    tycho_execution::encoding::{
        errors::EncodingError,
        evm::utils::bytes_to_address,
        models::{EncodedSolution, NativeAction, Solution, Transaction},
    },
};

/// Encodes a transaction for the Tycho Router using `singleSwap` method and regular token
/// transfers.
///
/// # Parameters
/// - `encoded_solution`: The solution already encoded by Tycho.
/// - `solution`: The high-level solution including tokens, amounts, and receiver info.
/// - `native_address`: The address used to represent the native token
///
/// # Returns
/// A `Result<Transaction, EncodingError>` that either contains the full transaction data (to,
/// value, data), or an error if the inputs are invalid.
pub fn encode_tycho_router_call(
    encoded_solution: EncodedSolution,
    solution: &Solution,
    native_address: &Bytes,
) -> Result<Transaction, EncodingError> {
    let (mut unwrap, mut wrap) = (false, false);
    if let Some(action) = solution.native_action.clone() {
        match action {
            NativeAction::Wrap => wrap = true,
            NativeAction::Unwrap => unwrap = true,
        }
    }

    let given_amount = biguint_to_u256(&solution.given_amount);
    let min_amount_out = biguint_to_u256(&solution.checked_amount);
    let given_token = bytes_to_address(&solution.given_token)?;
    let checked_token = bytes_to_address(&solution.checked_token)?;
    let receiver = bytes_to_address(&solution.receiver)?;

    let method_calldata = if encoded_solution
        .function_signature
        .contains("singleSwap")
    {
        (
            given_amount,
            given_token,
            checked_token,
            min_amount_out,
            wrap,
            unwrap,
            receiver,
            true,
            encoded_solution.swaps,
        )
            .abi_encode()
    } else {
        Err(EncodingError::FatalError("Invalid function signature for Tycho router".to_string()))?
    };

    let contract_interaction = encode_input(&encoded_solution.function_signature, method_calldata);
    let value = if solution.given_token == *native_address {
        solution.given_amount.clone()
    } else {
        BigUint::ZERO
    };
    Ok(Transaction { to: encoded_solution.interacting_with, value, data: contract_interaction })
}

/// Encodes the input data for a function call to the given function selector.
pub fn encode_input(selector: &str, mut encoded_args: Vec<u8>) -> Vec<u8> {
    let mut hasher = Keccak256::new();
    hasher.update(selector.as_bytes());
    let selector_bytes = &hasher.finalize()[..4];
    let mut call_data = selector_bytes.to_vec();
    // Remove extra prefix if present (32 bytes for dynamic data)
    // Alloy encoding is including a prefix for dynamic data indicating the offset or length
    // but at this point we don't want that
    if encoded_args.len() > 32 &&
        encoded_args[..32] ==
            [0u8; 31]
                .into_iter()
                .chain([32].to_vec())
                .collect::<Vec<u8>>()
    {
        encoded_args = encoded_args[32..].to_vec();
    }
    call_data.extend(encoded_args);
    call_data
}
