use crate::abi;
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{
    hex,
    pb::substreams::StoreDeltas,
    store::{StoreAdd, StoreAddBigInt, StoreAddInt64, StoreGet, StoreGetInt64, StoreNew},
};
use substreams_ethereum::{
    pb::eth::{self},
    Event,
};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes, prelude::*,
};

#[substreams::handlers::map]
pub fn map_components(
    params: String,
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents, anyhow::Error> {
    let restake_manager_address = hex::decode(params)?;
    let locked_assets = find_deployed_underlying_addresses(&restake_manager_address)
        .ok_or_else(|| anyhow!("No underlying assets found for restake manager"))?;

    Ok(BlockTransactionProtocolComponents {
        tx_components: block
            .transactions()
            .filter_map(|tx| {
                let components = tx
                    .calls()
                    .filter(|call| !call.call.state_reverted)
                    .filter_map(|_| {
                        if is_deployment_tx(tx, &restake_manager_address) {
                            Some(
                                ProtocolComponent::at_contract(&restake_manager_address, &tx.into())
                                    .with_tokens(&locked_assets.concat())
                                    .as_swap_type("renzo_vault", ImplementationType::Vm),
                            )
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                if !components.is_empty() {
                    Some(TransactionProtocolComponents { tx: Some(tx.into()), components })
                } else {
                    None
                }
            })
            .collect(),
    })
}

#[substreams::handlers::store]
pub fn store_components(map: BlockTransactionProtocolComponents, store: StoreAddInt64) {
    store.add_many(
        0,
        &map.tx_components
            .iter()
            .flat_map(|tx_components| &tx_components.components)
            .map(|component| format!("restake_manager:{}:{}", component.id, component.tokens.join(",")))
            .collect::<Vec<_>>(),
        1,
    );
}
fn find_deployed_underlying_addresses(restake_manager_address: &[u8]) -> Option<Vec<Vec<u8>>> {
    match restake_manager_address {
        hex!("74a09653A083691711cF8215a6ab074BB4e99ef5") => Some(vec![
            hex!("bf5495Efe5DB9ce00f80364C8B423567e58d2110").to_vec(),
            hex!("a2E3356610840701BDf5611a53974510Ae27E2e1").to_vec(),
            hex!("ae7ab96520DE3A18E5e111B5EaAb095312D7fE84").to_vec(),
            hex!("0000000000000000000000000000000000000000").to_vec(),
        ]),
        _ => None,
    }
}

fn is_deployment_tx(tx: &Transaction, address: &[u8]) -> bool {
    tx.logs().any(|log| log.address == address && tx.to().is_none())
}