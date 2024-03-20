use anyhow::{anyhow, bail};
use ethabi::{decode, ParamType};
use substreams_ethereum::pb::eth::v2::Call;
use tycho_substreams::prelude::{ImplementationType, ProtocolComponent, Transaction};

pub const CREATE_PAIR_SIGNATURE: [u8; 4] = [0xc9, 0xc6, 0x56, 0x90];

pub fn decode_create_pair_call(
    call: &Call,
    tx: &Transaction,
) -> Result<ProtocolComponent, anyhow::Error> {
    let abi_types = &[ParamType::Address, ParamType::Address];

    if let Ok(params) = decode(abi_types, &call.input[4..]) {
        let token_a = params[0]
            .to_owned()
            .into_address()
            .ok_or_else(|| anyhow!("Failed to convert to address: {:?}", &params[0]))?
            .to_fixed_bytes()
            .to_vec();

        let token_b = params[1]
            .to_owned()
            .into_address()
            .ok_or_else(|| anyhow!("Failed to convert to address: {:?}", &params[1]))?
            .to_fixed_bytes()
            .to_vec();

        let mut tokens = vec![token_a.clone(), token_b.clone()];
        tokens.sort();

        let id = format!("{}{}", hex::encode(token_a), hex::encode(token_b));

        Ok(ProtocolComponent::at_contract(&call.address, tx)
            .with_tokens(&tokens)
            .as_swap_type("uniswap_v2_pair", ImplementationType::Vm))
    } else {
        bail!("Failed to decode ABI call parameters.".to_string())
    }
}