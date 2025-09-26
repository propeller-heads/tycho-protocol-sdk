use crate::modules::utils::Params;
use crate::pb::cowamm::CowPool;
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{key, pb::substreams::StoreDeltas, prelude::{BigInt, StoreGetProto}, store::StoreGet};
use substreams_ethereum::pb::eth::v2::{Block};
use substreams_helper::hex::Hexable;
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes_builder, prelude::*,
};
use std::{str::FromStr};

#[substreams::handlers::map]
fn map_protocol_changes(
    params: String,
    block: Block,
    protocol_components: BlockTransactionProtocolComponents,
    balance_deltas: BlockBalanceDeltas,
    pool_store: StoreGetProto<CowPool>,
    balance_store: StoreDeltas,
) -> Result<BlockChanges, substreams::errors::Error> {
    let params = Params::parse_from_query(&params)?;
    let factory_address = params
        .decode_addresses()
        .expect("unable to extract factory address");

    let mut transaction_changes: HashMap<_, TransactionChangesBuilder> = HashMap::new();
    // Aggregate newly created components per tx
    protocol_components
        .tx_components
        .iter()
        .for_each(|tx_component| {
            // initialise builder if not yet present for this tx
            let tx = tx_component.tx.as_ref().unwrap();
            let builder = transaction_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionChangesBuilder::new(tx));

            // iterate over individual components created within this tx
            tx_component
                .components
                .iter()
                .for_each(|component| {
                    builder.add_protocol_component(component);
                    // Register the Pool liquidities for token_a , token_b and lp_token_supply as Entity Changes
                    balance_store
                        .clone()
                        .deltas
                        .into_iter()
                        .zip(balance_deltas.balance_deltas.clone())
                        .for_each(|(store_delta, balance_delta)| { 
                            let tx = balance_delta.tx.unwrap();

                            let new_value_bigint = BigInt::from_str(key::segment_at(
                                &String::from_utf8(store_delta.new_value).unwrap(),
                                1,
                            ))
                            .unwrap();
                            //Perform check to know value of the delta to assign to which attribute to
                            //check and verify thec correctness of the token ordering
                            let mut attr_vec : Vec<Attribute> = Vec::new(); 
                            match balance_delta.token {
                                token_address => {
                                    if token_address == component.tokens[0] {
                                        attr_vec.push(
                                        Attribute {
                                                name: "liquidity_a".to_string(),
                                                value: new_value_bigint.to_signed_bytes_be(),
                                                change: ChangeType::Update.into(),
                                        },
                                        );
                                    };
                                    if token_address == component.tokens[1] {
                                        attr_vec.push(
                                        Attribute {
                                                name: "liquidity_b".to_string(),
                                                value: new_value_bigint.to_signed_bytes_be(),
                                                change: ChangeType::Update.into(),
                                        },
                                        );
                                    };
                                    if token_address == component.tokens[2] {
                                        attr_vec.push(
                                        Attribute {
                                                name: "lp_token_supply".to_string(),
                                                value: new_value_bigint.to_signed_bytes_be(),
                                                change: ChangeType::Update.into(),
                                        },
                                        );
                                    }
                                },
                                _ => panic!("unknown token balance delta")
                            }
                            builder.add_entity_change(&EntityChanges {
                                component_id: balance_delta.component_id.to_hex(), //verify correctness of this
                                attributes: attr_vec,
                            });
                        });
                });
        });

    // // Aggregate absolute balances per transaction.
    // we do not want to create a balance change for the lp_token_supply, because it does not reflect 
    // a change to the TVL of the component
    //the balance_store is additive, so we should have the final result of the delta
    // create a new filtered vec to filter all the balance_deltas and balace_store that have the lp_token address so 
    //we don't create balance deltas for lp_token_supply changes
    aggregate_balances_changes(balance_store, balance_deltas)
        .into_iter()
        .for_each(|(_, (tx, balances))| {
            let builder = transaction_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionChangesBuilder::new(&tx));
            balances
                .values()
                .for_each(|token_bc_map| {
                    token_bc_map
                        .values()
                        .for_each(|bc| { 
                            builder.add_balance_change(bc)
                        })
                });
        });
    
    extract_contract_changes_builder(
        &block,
        |address| {
            pool_store
                .get_last(format!("Pool:0x{}", hex::encode(address)))
                .is_some()
                || address.eq(factory_address.as_slice())
        },
        &mut transaction_changes,
    );
    block
        .transactions()
        .for_each(|block_tx| {
            block_tx.calls.iter().for_each(|call| {
                if call.address == factory_address {
                    let mut contract_change =
                        InterimContractChange::new(call.address.as_slice(), true);

                    if let Some(code_change) = &call.code_changes.first() {
                        contract_change.set_code(&code_change.new_code);
                    }

                    let builder = transaction_changes
                        .entry(block_tx.index.into())
                        .or_insert_with(|| TransactionChangesBuilder::new(&(block_tx.into())));
                    builder.add_contract_changes(&contract_change);
                }
            });
        });

    transaction_changes
        .iter_mut()
        .for_each(|(_, change)| {
            // this indirection is necessary due to borrowing rules.
            let addresses = change
                .changed_contracts()
                .map(|e| e.to_vec())
                .collect::<Vec<_>>();
            addresses
                .into_iter()
                .for_each(|address| {
                    // check if the address is not the factory address
                    if address != factory_address.as_slice() {
                        let pool = pool_store
                            .get_last(format!("Pool:0x{}", hex::encode(address)))
                            .unwrap();
                        change.mark_component_as_updated(&pool.address.to_hex());
                    }
                })
        });

    Ok(BlockChanges {
        block: Some((&block).into()),
        changes: transaction_changes
            .drain()
            .sorted_unstable_by_key(
                |(index, _): &(u64, tycho_substreams::models::TransactionChangesBuilder)| *index,
            )
            .filter_map(|(_, builder)| builder.build())
            .collect::<Vec<_>>(),
    })
}
