use crate::{pb::cowamm::CowPool};
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{pb::substreams::StoreDeltas, prelude::StoreGetProto, store::StoreGet};
use substreams_ethereum::pb::eth::v2::Block;
use substreams_helper::hex::Hexable;
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes_builder, prelude::*,
};
use serde::Deserialize;

#[substreams::handlers::map]
fn map_protocol_changes(
    params: String,
    block: Block,
    protocol_components: BlockTransactionProtocolComponents,
    balance_deltas: BlockBalanceDeltas,
    pool_store: StoreGetProto<CowPool>,
    balance_store: StoreDeltas,
) -> Result<BlockChanges, substreams::errors::Error> {
    #[derive(Debug, Deserialize)]
    pub struct Params {
        pub factory_address: String,
    }
    let params: Params = serde_qs::from_str(params.as_str()).expect("Unable to deserialize params");
    let factory_address : String = params.factory_address;
    // We merge contract changes by transaction (identified by transaction index)
    // making it easy to sort them at the very end.
    let mut transaction_changes: HashMap<_, TransactionChangesBuilder> = HashMap::new();

    // Aggregate newly created components per tx
    // protocol_components
    //     .tx_components
    //     .iter()
    //     .for_each(|tx_component| {
    //         // initialise builder if not yet present for this tx
    //         let tx = tx_component.tx.as_ref().unwrap();
    //         let builder = transaction_changes
    //             .entry(tx.index)
    //             .or_insert_with(|| TransactionChangesBuilder::new(tx));

    //         // iterate over individual components created within this tx
    //         tx_component
    //             .components
    //             .iter()
    //             .for_each(|component| {
    //                 builder.add_protocol_component(component);
    //                 // TODO: In case you require to add any dynamic attributes to the
    //                 //  component you can do so here:
    //                 /*
    //                     builder.add_entity_change(&EntityChanges {
    //                         component_id: component.id.clone(),
    //                         attributes: default_attributes.clone(),
    //                     });
    //                 */
    //             });
    //     });

    // // Aggregate absolute balances per transaction.
    // aggregate_balances_changes(balance_store, deltas)
    //     .into_iter()
    //     .for_each(|(_, (tx, balances))| {
    //         let builder = transaction_changes
    //             .entry(tx.index)
    //             .or_insert_with(|| TransactionChangesBuilder::new(&tx));
    //         balances
    //             .values()
    //             .for_each(|token_bc_map| {
    //                 token_bc_map
    //                     .values()  
    //                     .for_each(|bc| builder.add_balance_change(bc))
    //             });
    //     });

    // Extract and insert any storage changes that happened for any of the components.
    // extract_contract_changes_builder(
    //     &block,
    //     |addr| {
    //         // we assume that the store holds contract addresses as keys and if it
    //         // contains a value, that contract is of relevance.
    //         // TODO: if you have any additional static contracts that need to be indexed,
    //         //  please add them here.
    //         pool_store
    //             .get_last(hex::encode(addr))
    //             .is_some() || 
    //             addr.eq(factory_address.as_slice())
    //     },
    //     &mut transaction_changes,
    // );
    // block
    // .transactions()
    // .for_each(|block_tx| {
    //     block_tx.calls.iter().for_each(|call| {
    //         if call.address == factory_address {
    //             let mut contract_change =
    //                 InterimContractChange::new(call.address.as_slice(), true);

    //             if let Some(code_change) = &call.code_changes.first() {
    //                 contract_change.set_code(&code_change.new_code);
    //             }

    //             let builder = transaction_changes
    //                 .entry(block_tx.index.into())
    //                 .or_insert_with(|| TransactionChangesBuilder::new(&(block_tx.into())));
    //             builder.add_contract_changes(&contract_change);
    //         }
    //     });
    // });

// transaction_changes
//     .iter_mut()
//     .for_each(|(_, change)| {
//         // this indirection is necessary due to borrowing rules.
//         let addresses = change
//             .changed_contracts()
//             .map(|e| e.to_vec())
//             .collect::<Vec<_>>();
//         addresses
//             .into_iter()
//             .for_each(|address| {
//                 // check if the address is not a pool
//                 if address != factory_address.as_slice()
//                 {
//                     let pool = pool_store
//                         .get_last(format!("Pool:0x{}", hex::encode(address)))
//                         .unwrap();
//                     change.mark_component_as_updated(&pool.address.to_hex());
//                 }
//             })
//     });
    // Process all `transaction_changes` for final output in the `BlockChanges`,
    //  sorted by transaction index (the key).
    Ok(BlockChanges {
        block: Some((&block).into()),
        changes: transaction_changes
            .drain()
            .sorted_unstable_by_key(|(index, _): &(u64, tycho_substreams::models::TransactionChangesBuilder)| *index)
            .filter_map(|(_, builder)| builder.build())
            .collect::<Vec<_>>(),
    })
}
