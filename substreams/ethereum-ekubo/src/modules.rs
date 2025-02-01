use serde::Deserialize;
use substreams_ethereum::{pb::eth, Event};
use tiny_keccak::{Hasher, Keccak};
use tycho_substreams::prelude::*;

#[derive(Debug, Deserialize)]
struct Params {
    core_address: String,
    oracle_address: String,
}

#[substreams::handlers::map]
fn map_protocol_changes(
    params: String,
    block: eth::v2::Block,
) -> Result<BlockContractChanges, substreams::errors::Error> {
    let query: Params = serde_qs::from_str(params.as_str()).unwrap();

    let mut transaction_contract_changes = Vec::<TransactionContractChanges>::new();
    let decoded_core_address = hex::decode(query.core_address)?;
    let decoded_oracle_address = hex::decode(query.oracle_address)?;

    block
        .transactions()
        .into_iter()
        .for_each(|t| {
            t.logs_with_calls()
                .into_iter()
                .for_each(|(l, _)| {
                    if let Some(pi) = crate::abi::core::events::PoolInitialized::match_and_decode(l)
                    {
                        let mut hasher = Keccak::v256();
                        hasher.update(pi.pool_key.0.clone().as_slice());
                        hasher.update(pi.pool_key.1.clone().as_slice());
                        hasher.update(pi.pool_key.2.to_bytes_be().1.as_slice());
                        hasher.update(pi.pool_key.3.to_bytes_be().1.as_slice());
                        hasher.update(pi.pool_key.4.as_slice());

                        let mut output = [0u8; 32];
                        hasher.finalize(&mut output);
                        let pool_id = hex::encode(output);

                        transaction_contract_changes.push(TransactionContractChanges {
                            tx: Some(t.into()),
                            balance_changes: vec![],
                            contract_changes: vec![],
                            component_changes: vec![ProtocolComponent {
                                id: pool_id,
                                tx: Some(t.into()),
                                tokens: vec![pi.pool_key.0.clone(), pi.pool_key.1.clone()],
                                contracts: if decoded_oracle_address == pi.pool_key.4 {
                                    vec![
                                        decoded_core_address.clone(),
                                        decoded_oracle_address.clone(),
                                    ]
                                } else {
                                    vec![decoded_core_address.clone()]
                                },
                                change: ChangeType::Creation.into(),
                                protocol_type: Some(ProtocolType {
                                    name: "EKUBO".to_string(),
                                    financial_type: FinancialType::Swap.into(),
                                    implementation_type: ImplementationType::Vm.into(),
                                    attribute_schema: vec![],
                                }),
                                static_att: vec![
                                    Attribute {
                                        change: ChangeType::Creation.into(),
                                        name: "token0".to_string(),
                                        value: pi.pool_key.0,
                                    },
                                    Attribute {
                                        change: ChangeType::Creation.into(),
                                        name: "token1".to_string(),
                                        value: pi.pool_key.1,
                                    },
                                    Attribute {
                                        change: ChangeType::Creation.into(),
                                        name: "fee".to_string(),
                                        value: pi.pool_key.2.to_bytes_be().1,
                                    },
                                    Attribute {
                                        change: ChangeType::Creation.into(),
                                        name: "tick_spacing".to_string(),
                                        value: pi.pool_key.3.to_bytes_be().1,
                                    },
                                    Attribute {
                                        change: ChangeType::Creation.into(),
                                        name: "extension".to_string(),
                                        value: pi.pool_key.4,
                                    },
                                ],
                            }],
                        });
                    }
                })
        });

    // TODO: protocol specific logic goes here
    Ok(BlockContractChanges { block: Some((&block).into()), changes: transaction_contract_changes })
}
