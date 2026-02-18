use itertools::Itertools;
use std::{collections::HashMap, str::FromStr};
use substreams::{pb::substreams::StoreDeltas, scalar::BigInt};
use substreams_ethereum::pb::eth::v2::Block;
use tycho_substreams::models::{
    Attribute, BlockChanges, BlockTransactionProtocolComponents, ChangeType, EntityChanges,
    Transaction, TransactionChangesBuilder,
};

use crate::pb::adapter::v1::FunctionCalls;

#[substreams::handlers::map]
pub fn map_protocol_changes(
    block: Block,
    func_calls: FunctionCalls,
    token_pairs: BlockTransactionProtocolComponents,
    exchange_price_deltas: StoreDeltas,
) -> Result<BlockChanges, substreams::errors::Error> {
    let mut transaction_changes: HashMap<_, TransactionChangesBuilder> = HashMap::new();

    // Aggregate newly created components per tx
    token_pairs
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
                });
        });

    exchange_price_deltas
        .deltas
        .iter()
        .zip(func_calls.calls)
        .for_each(|(store_delta, call)| {
            let new_value_bigint = BigInt::from_str(
                String::from_utf8(store_delta.new_value.clone())
                    .expect("error while converting a string from store delta")
                    .as_str(),
            )
            .unwrap();
            let tx = call.transaction.unwrap();
            let builder = transaction_changes
                .entry(tx.index as u64)
                .or_insert_with(|| {
                    TransactionChangesBuilder::new(&Transaction {
                        hash: tx.hash.clone(),
                        from: tx.from.clone(),
                        to: tx.to.clone(),
                        index: tx.index as u64,
                    })
                });

            builder.add_entity_change(&EntityChanges {
                component_id: call.id,
                attributes: vec![Attribute {
                    name: "price".to_string(),
                    value: new_value_bigint.to_signed_bytes_be(),
                    change: ChangeType::Update.into(),
                }],
            });
        });

    Ok(BlockChanges {
        block: Some((&block).into()),
        changes: transaction_changes
            .drain()
            .sorted_unstable_by_key(|(index, _)| *index)
            .filter_map(|(_, builder)| builder.build())
            .collect::<Vec<_>>(),
    })
}
