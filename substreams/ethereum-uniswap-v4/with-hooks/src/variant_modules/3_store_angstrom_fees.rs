use crate::{
    pb::uniswap::angstrom::AngstromFees,
    store_tokens_to_pool_id_angstrom::generate_store_key_from_assets,
};
use ethabi::ethereum_types::Address;
use ethereum_uniswap_v4_shared::abi::angstrom::{BatchUpdatePools, PoolConfigured};
use std::str::FromStr;
use substreams::store::{StoreGet, StoreGetString, StoreNew, StoreSet, StoreSetProto};
use substreams_ethereum::pb::eth::v2::{self as eth};
use substreams_helper::{event_handler::EventHandler, hex::Hexable};

#[substreams::handlers::store]
pub fn store_angstrom_fees(
    controller_address: String,
    block: eth::Block,
    tokens_to_id_store: StoreGetString,
    fee_store: StoreSetProto<AngstromFees>,
) {
    // Track PoolConfigured events from Angstrom Controller contract
    let mut on_pool_configured = |event: PoolConfigured,
                                  _tx: &eth::TransactionTrace,
                                  _log: &eth::Log| {
        let store_key = generate_store_key_from_assets(&event.asset0, &event.asset1);

        let component_id = tokens_to_id_store
            .get_last(store_key.clone())
            .expect("Component ID should exist for Angstrom pool assets store");

        let fees = AngstromFees {
            bundle_fee: event.bundle_fee.clone(),
            unlocked_fee: event.unlocked_fee.clone(),
            protocol_unlocked_fee: event.protocol_unlocked_fee.clone(),
        };

        // Store the fees with the component_id as key
        fee_store.set(0, &component_id, &fees);

        substreams::log::debug!(
            "Storing Angstrom fees for assets {}/{} with component id: {:?} - bundle: {}, unlocked: {}, protocol: {}",
            event.asset0.to_hex(),
            event.asset1.to_hex(),
            component_id,
            hex::encode(&fees.bundle_fee),
            hex::encode(&fees.unlocked_fee),
            hex::encode(&fees.protocol_unlocked_fee),
        );
    };

    // Process batchUpdatePools calls
    for tx in block.transactions() {
        for call in &tx.calls {
            if call.state_reverted {
                continue;
            }

            let call_address = call.address.to_hex().to_lowercase();
            if call_address == controller_address {
                if let Ok(batch_update) = BatchUpdatePools::decode_call(&call.input) {
                    for pool_update in batch_update.updates {
                        let store_key = generate_store_key_from_assets(
                            &pool_update.asset_a,
                            &pool_update.asset_b,
                        );

                        if let Some(component_id) = tokens_to_id_store.get_last(&store_key) {
                            substreams::log::debug!(
                                "Updating Angstrom fees via batchUpdatePools for assets {}/{} with component id: {:?} - bundle: {}, unlocked: {}, protocol: {}",
                                pool_update.asset_a.to_hex(),
                                pool_update.asset_b.to_hex(),
                                component_id,
                                hex::encode(&pool_update.bundle_fee),
                                hex::encode(&pool_update.unlocked_fee),
                                hex::encode(&pool_update.protocol_unlocked_fee));

                            let fees = AngstromFees {
                                bundle_fee: pool_update.bundle_fee,
                                unlocked_fee: pool_update.unlocked_fee,
                                protocol_unlocked_fee: pool_update.protocol_unlocked_fee,
                            };

                            fee_store.set(0, &component_id, &fees);
                        }
                    }
                }
            }
        }
    }

    let mut eh = EventHandler::new(&block);
    eh.filter_by_address(vec![Address::from_str(&controller_address).unwrap()]);
    eh.on::<PoolConfigured, _>(&mut on_pool_configured);
    eh.handle_events();
}
