mod abi;
mod modules;

#[substreams::handlers::map]
fn map_changes(
    block: eth::v2::Block,
) -> Result<tycho::BlockContractChanges, substreams::errors::Error> {
    modules::map_protocol_changes(
        block,
        modules::map_components(block.clone()?)?,
        modules::map_relative_balances(block.clone()?, modules::store_components_get())?,
        modules::store_components_get(),
        modules::store_balances_deltas(),
    )