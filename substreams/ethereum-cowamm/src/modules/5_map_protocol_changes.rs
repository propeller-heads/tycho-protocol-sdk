use crate::{modules::utils::Params, pb::maverick::v2::Pool};
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{pb::substreams::StoreDeltas, prelude::StoreGetProto, store::StoreGet};
use substreams_ethereum::pb::eth::v2::Block;
use substreams_helper::hex::Hexable;
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes_builder, prelude::*,
};

#[substreams::handlers::map]
fn map_protocol_changes(
    block: eth::v2::Block,
    new_components: BlockTransactionProtocolComponents,
    components_store: StoreGetRaw,
    balance_store: StoreDeltas,
    deltas: BlockBalanceDeltas,
) -> Result<BlockChanges, substreams::errors::Error> {
    // We merge contract changes by transaction (identified by transaction index)
    // making it easy to sort them at the very end.
    let mut transaction_changes: HashMap<_, TransactionChangesBuilder> = HashMap::new();

    // Aggregate newly created components per tx
    new_components
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
                    // TODO: In case you require to add any dynamic attributes to the
                    //  component you can do so here:
                    /*
                        builder.add_entity_change(&EntityChanges {
                            component_id: component.id.clone(),
                            attributes: default_attributes.clone(),
                        });
                    */
                });
        });

    // Aggregate absolute balances per transaction.
    aggregate_balances_changes(balance_store, deltas)
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
                        .for_each(|bc| builder.add_balance_change(bc))
                });
        });

    // Extract and insert any storage changes that happened for any of the components.
    extract_contract_changes_builder(
        &block,
        |addr| {
            // we assume that the store holds contract addresses as keys and if it
            // contains a value, that contract is of relevance.
            // TODO: if you have any additional static contracts that need to be indexed,
            //  please add them here.
            components_store
                .get_last(hex::encode(addr))
                .is_some()
        },
        &mut transaction_changes,
    );

    // Process all `transaction_changes` for final output in the `BlockChanges`,
    //  sorted by transaction index (the key).
    Ok(BlockChanges {
        block: Some((&block).into()),
        changes: transaction_changes
            .drain()
            .sorted_unstable_by_key(|(index, _)| *index)
            .filter_map(|(_, builder)| builder.build())
            .collect::<Vec<_>>(),
    })
}
