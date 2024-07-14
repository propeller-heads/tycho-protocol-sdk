use super::modules::{GRAIL_ADDRESS, XGRAIL_ADDRESS};
use crate::abi;
use substreams_ethereum::{
    pb::eth::v2::{Log, TransactionTrace},
    Event,
};
use tycho_substreams::{models::Transaction, prelude::*};

pub fn address_map(log: &Log, tx_trace: &TransactionTrace) -> Option<ProtocolComponent> {
    let tx: Transaction = tx_trace.into();
    if let Some(ev) = abi::factory::events::PairCreated::match_and_decode(log) {
        let pool_address = ev.pair;
        let token_0 = ev.token0;
        let token_1 = ev.token1;

        Some(
            ProtocolComponent::at_contract(&pool_address, &tx)
                .with_tokens(&[token_0, token_1])
                .as_swap_type("camelot_pair", ImplementationType::Vm),
        )
    } else if is_deployment_tx(tx_trace, &XGRAIL_ADDRESS) {
        Some(
            ProtocolComponent::at_contract(&XGRAIL_ADDRESS, &tx)
                .with_tokens(&[XGRAIL_ADDRESS, GRAIL_ADDRESS])
                .as_swap_type("camelot_grail", ImplementationType::Vm),
        )
    } else {
        None
    }
}

fn is_deployment_tx(tx: &TransactionTrace, vault_address: &[u8]) -> bool {
    let created_accounts = tx
        .calls
        .iter()
        .flat_map(|call| {
            call.account_creations
                .iter()
                .map(|ac| ac.account.to_owned())
        })
        .collect::<Vec<_>>();

    if let Some(deployed_address) = created_accounts.first() {
        return deployed_address.as_slice() == vault_address
    }
    false
}
