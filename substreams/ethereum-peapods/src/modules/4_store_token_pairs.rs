use substreams::store::*;
use tycho_substreams::models::{BlockTransactionProtocolComponents, ProtocolComponent};

// sets the exchange price of token pairs in the store
#[substreams::handlers::store]
pub fn store_token_pairs(
    token_pairs_proto_comp: BlockTransactionProtocolComponents,
    store: StoreSetProto<ProtocolComponent>,
) {
    for tx_comp in token_pairs_proto_comp.tx_components {
        for proto_comp in tx_comp.components {
            store.set(0, proto_comp.id.clone(), &proto_comp);
        }
    }
}
