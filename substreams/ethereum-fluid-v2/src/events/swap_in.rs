use crate::{
    abi::d3_swap_module::events::LogSwapIn,
    events::EventTrait,
    storage::{dex_v2, storage_view::StorageChangesView},
};
use substreams_ethereum::pb::eth::v2::StorageChange;
use tycho_substreams::prelude::*;

impl EventTrait for LogSwapIn {
    fn get_changed_attributes(
        &self,
        storage_changes: &[StorageChange],
        dex_v2_address: &[u8],
    ) -> (String, Vec<Attribute>) {
        let storage_view = StorageChangesView::new_filtered(dex_v2_address, storage_changes);
        let dex_type = self.dex_type.to_u64();
        let mut attrs = Vec::new();
        attrs.extend(dex_v2::dex_variables_attributes(&storage_view, &self.dex_id, dex_type));
        attrs.extend(dex_v2::token_reserves_attributes(&storage_view, &self.dex_id, dex_type));
        (hex::encode(self.dex_id), attrs)
    }

    fn get_balance_delta(&self, _tx: &Transaction, _ordinal: u64) -> Vec<BalanceDelta> {
        vec![]
    }
}
