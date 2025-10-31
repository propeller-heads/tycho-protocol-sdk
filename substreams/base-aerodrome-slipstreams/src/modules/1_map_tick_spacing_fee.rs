use crate::{
    abi::factory::events::TickSpacingEnabled,
    modules::utils::Params,
    pb::tycho::evm::aerodrome::{TickSpacingFee, TickSpacingFees},
};
use ethabi::ethereum_types::Address;
use std::str::FromStr;
use substreams_ethereum::pb::eth::v2::{self as eth};
use substreams_helper::event_handler::EventHandler;

#[substreams::handlers::map]
pub fn map_tick_spacing_fee(
    params: String,
    block: eth::Block,
) -> Result<TickSpacingFees, substreams::errors::Error> {
    let params = Params::parse_from_query(&params)?;
    let mut tick_spacing_to_fees = TickSpacingFees::default();
    get_tick_spacing_to_fees(&block, params.factory.as_str(), &mut tick_spacing_to_fees);
    Ok(tick_spacing_to_fees)
}

fn get_tick_spacing_to_fees(
    block: &eth::Block,
    factory_address: &str,
    tick_spacing_to_fees: &mut TickSpacingFees,
) {
    let mut on_tick_spacing_enabled =
        |event: TickSpacingEnabled, _tx: &eth::TransactionTrace, _log: &eth::Log| {
            tick_spacing_to_fees
                .tick_spacing_fees
                .push(TickSpacingFee {
                    tick_spacing: event.tick_spacing.to_i32(),
                    fee: event.fee.to_u64(),
                })
        };

    let mut eh = EventHandler::new(block);

    eh.filter_by_address(vec![Address::from_str(factory_address).unwrap()]);

    eh.on::<TickSpacingEnabled, _>(&mut on_tick_spacing_enabled);
    eh.handle_events();
}
