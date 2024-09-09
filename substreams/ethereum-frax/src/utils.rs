use substreams::store::{StoreGet, StoreGetProto, StoreSet, StoreSetProto};
use tycho_substreams::models::ProtocolComponent;

pub fn maybe_get_pool_tokens(
    store: &StoreGetProto<ProtocolComponent>,
    component_id: &str,
) -> Option<(Vec<u8>, Vec<u8>)> {
    store
        .get_last(format!("pool:{}", component_id))
        .map(|component| (component.tokens[0].to_vec(), component.tokens[1].to_vec()))
}

pub fn address_to_hex(address: &[u8]) -> String {
    format!("0x{}", hex::encode(address))
}

pub fn string_to_bytes(string: &str) -> Vec<u8> {
    string.as_bytes().to_vec()
}

pub fn save_component(store: &StoreSetProto<ProtocolComponent>, component: &ProtocolComponent) {
    store.set(1, format!("pool:{}", component.id), component);
}
