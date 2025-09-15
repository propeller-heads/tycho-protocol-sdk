use substreams::store::{StoreNew, StoreSetIfNotExists, StoreSetIfNotExistsInt64};
use tycho_substreams::prelude::*;

/// Set of token that are used by BalancerV3. This is used to filter out account balances updates
/// for unknown tokens.
#[substreams::handlers::store]
pub fn store_token_set(map: BlockTransactionProtocolComponents, store: StoreSetIfNotExistsInt64) {
    map.tx_components
        .into_iter()
        .for_each(|tx_pc| {
            tx_pc
                .components
                .into_iter()
                .for_each(|pc| {
                    pc.tokens
                        .into_iter()
                        .for_each(|token| store.set_if_not_exists(0, hex::encode(token), &1));
                })
        });
}
