use crate::modules::utils::Params;
use crate::pb::cowamm::{CowPool, BlockPoolChanges};
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{key, pb::substreams::{StoreDelta, StoreDeltas}, prelude::{BigInt, StoreGetProto}, store::StoreGet};
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
    block_pool_changes: BlockPoolChanges,
    pool_store: StoreGetProto<CowPool>,
    balance_store: StoreDeltas,
) -> Result<BlockChanges, substreams::errors::Error> {
    let protocol_components = block_pool_changes.tx_protocol_components.expect("no tx components"); //change this to a return
    let balance_deltas = block_pool_changes
                                .block_balance_deltas
                                .expect("no block balance deltas")
                                .balance_deltas
                                .into_iter()
                                .map(Into::into)
                                .collect::<Vec<BalanceDelta>>();

    let params = Params::parse_from_query(&params)?;
    let factory_address = params
        .decode_addresses()
        .expect("unable to extract factory address");

    let mut transaction_changes: HashMap<_, TransactionChangesBuilder> = HashMap::new();
    // Aggregate newly created components per tx
    protocol_components
        .tx_components
        .into_iter()
        .for_each(|tx_component| {
            let tx = tx_component.tx.unwrap();
            let builder = transaction_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionChangesBuilder::new(&(&tx).into()));

            // iterate over individual components created within this tx
            tx_component
                .components
                .iter()
                .for_each(|component| {
                    builder.add_protocol_component(&component.into());
                });
        });
        // Register the Pool liquidities for token_a , token_b and lp_token_supply as Entity Changes
    balance_store
        .clone()
        .deltas
        .into_iter()
        .zip(balance_deltas.clone())
        .for_each(|(store_delta, balance_delta)| { 
            let tx = balance_delta.tx.clone().unwrap();
            let new_value_bigint = BigInt::from_str(
                &String::from_utf8(store_delta.new_value).unwrap(),
            )
            .unwrap();
            
            let address = String::from_utf8(balance_delta.component_id).unwrap();
            let builder = transaction_changes
                                .entry(tx.index)
                                .or_insert_with(|| TransactionChangesBuilder::new(&tx));
            let pool = pool_store
                    .must_get_last(format!("Pool:{}", address));

            let mut attr_vec : Vec<Attribute> = Vec::new(); 
            match balance_delta.token {
                    t if t == pool.token_a => {
                        attr_vec.push(
                        Attribute {
                                name: "liquidity_a".to_string(),
                                value: new_value_bigint.to_signed_bytes_be(),
                                change: ChangeType::Update.into(),
                        },
                        );
                    }
                    t if t == pool.token_b => {
                        attr_vec.push(
                        Attribute {
                                name: "liquidity_b".to_string(),
                                value: new_value_bigint.to_signed_bytes_be(),
                                change: ChangeType::Update.into(),
                        },
                        );
                    }
                    t if t ==  pool.lp_token => {
                        attr_vec.push(
                        Attribute {
                                name: "lp_token_supply".to_string(),
                                value: new_value_bigint.to_signed_bytes_be(),
                                change: ChangeType::Update.into(),
                        });
                    }
                _ => panic!("unknown token balance delta") // not even possible 
            }
            builder.add_entity_change(&EntityChanges {
                component_id: address.clone(),
                attributes: attr_vec,
            });
        });

    // Aggregate absolute balances per transaction.

    // We do not want to create balance changes (that is BalanceChange objects) for the changes in the 
    // lp_token_supply, because its change does not reflect a change to the TVL of the component, the 
    // purpose of creating balance_deltas was for the lp_token_supply entity changes.

    // So we create new vecs with the balance_deltas and balance_store_deltas 
    // of the lp_token_supply filtered out

    //Remember that the lp_token_address is the same as the pool address (which is the component_id)
    
    let store_delta_vec = balance_store.deltas.into_iter().filter(|store_delta| {
        let component_id = key::segment_at(&store_delta.key, 0); //key is component_id/pool_address + token
        let token_address = key::segment_at(&store_delta.key, 1); 
        let formatted_token_address = format!("{}", token_address); 
        component_id != formatted_token_address
    }).collect::<Vec<StoreDelta>>();

    let new_balance_store = StoreDeltas { deltas : store_delta_vec };

    let balance_deltas_vec = balance_deltas.into_iter().filter(|balance_delta| {
        let delta = String::from_utf8(balance_delta.component_id.clone()).unwrap();
        let address = format!("{}", delta); 
        balance_delta.token.to_hex() != address
    }).collect::<Vec<BalanceDelta>>();

    let new_balance_deltas = BlockBalanceDeltas { balance_deltas: balance_deltas_vec };

    aggregate_balances_changes(new_balance_store, new_balance_deltas)
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
                        change.mark_component_as_updated(&pool.address.to_hex()); // does this overwrites the previous entity changes -> no
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
