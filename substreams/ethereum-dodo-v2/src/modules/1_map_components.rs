use crate::{
    abi::{
        dpp_factory::events::{NewDpp, RemoveDpp},
        dsp_factory::events::{NewDsp, RemoveDsp},
        dvm_factory::events::{NewDvm, RemoveDvm},
        gsp_factory::events::{NewGsp, RemoveGsp},
    },
    modules::utils::Params,
};
use anyhow::{Ok, Result};
use ethabi::ethereum_types::Address;
use std::{cell::RefCell, rc::Rc};
use substreams_ethereum::pb::eth::v2::{Block, TransactionTrace};
use substreams_helper::{event_handler::EventHandler, hex::Hexable};
use tycho_substreams::prelude::*;

#[substreams::handlers::map]
pub fn map_components(params: String, block: Block) -> Result<BlockTransactionProtocolComponents> {
    let mut new_pools: Vec<TransactionProtocolComponents> = vec![];
    let params = Params::parse_from_query(&params)?;
    get_new_pools(params, &block, &mut new_pools);
    Ok(BlockTransactionProtocolComponents { tx_components: new_pools })
}

fn handle_new_dpp(event: NewDpp, tx: &TransactionTrace) -> TransactionProtocolComponents {
    let tycho_tx: Transaction = tx.into();
    let tokens = [event.base_token.as_slice(), event.quote_token.as_slice()];
    let contracts = [event.dpp.as_slice()];
    let component = ProtocolComponent::new(&event.dpp.to_hex())
        .with_tokens(&tokens)
        .with_contracts(&contracts)
        .with_attributes(&[("pool_type", "dpp")])
        .as_swap_type("dodo_v2_pool", ImplementationType::Custom);

    TransactionProtocolComponents { tx: Some(tycho_tx), components: vec![component] }
}

fn handle_remove_dpp(event: RemoveDpp, tx: &TransactionTrace) -> TransactionProtocolComponents {
    let tycho_tx: Transaction = tx.into();
    let contracts = [event.dpp.as_slice()];
    let mut component = ProtocolComponent::new(&event.dpp.to_hex())
        .with_contracts(&contracts)
        .with_attributes(&[("pool_type", "dpp")])
        .as_swap_type("dodo_v2_pool", ImplementationType::Custom);

    component.set_change(ChangeType::Deletion);

    TransactionProtocolComponents { tx: Some(tycho_tx), components: vec![component] }
}

fn handle_new_dsp(event: NewDsp, tx: &TransactionTrace) -> TransactionProtocolComponents {
    let tycho_tx: Transaction = tx.into();
    let tokens = [event.base_token.as_slice(), event.quote_token.as_slice()];
    let contracts = [event.dsp.as_slice()];
    let component = ProtocolComponent::new(&event.dsp.to_hex())
        .with_tokens(&tokens)
        .with_contracts(&contracts)
        .with_attributes(&[("pool_type", "dsp")])
        .as_swap_type("dodo_v2_pool", ImplementationType::Custom);

    TransactionProtocolComponents { tx: Some(tycho_tx), components: vec![component] }
}

fn handle_remove_dsp(event: RemoveDsp, tx: &TransactionTrace) -> TransactionProtocolComponents {
    let tycho_tx: Transaction = tx.into();
    let contracts = [event.dsp.as_slice()];
    let mut component = ProtocolComponent::new(&event.dsp.to_hex())
        .with_contracts(&contracts)
        .with_attributes(&[("pool_type", "dsp")])
        .as_swap_type("dodo_v2_pool", ImplementationType::Custom);
    component.set_change(ChangeType::Deletion);

    TransactionProtocolComponents { tx: Some(tycho_tx), components: vec![component] }
}

fn handle_new_dvm(event: NewDvm, tx: &TransactionTrace) -> TransactionProtocolComponents {
    let tycho_tx: Transaction = tx.into();
    let tokens = [event.base_token.as_slice(), event.quote_token.as_slice()];
    let contracts = [event.dvm.as_slice()];
    let component = ProtocolComponent::new(&event.dvm.to_hex())
        .with_tokens(&tokens)
        .with_contracts(&contracts)
        .with_attributes(&[("pool_type", "dvm")])
        .as_swap_type("dodo_v2_pool", ImplementationType::Custom);

    TransactionProtocolComponents { tx: Some(tycho_tx), components: vec![component] }
}

fn handle_remove_dvm(event: RemoveDvm, tx: &TransactionTrace) -> TransactionProtocolComponents {
    let tycho_tx: Transaction = tx.into();
    let contracts = [event.dvm.as_slice()];
    let mut component = ProtocolComponent::new(&event.dvm.to_hex())
        .with_contracts(&contracts)
        .with_attributes(&[("pool_type", "dvm")])
        .as_swap_type("dodo_v2_pool", ImplementationType::Custom);
    component.set_change(ChangeType::Deletion);

    TransactionProtocolComponents { tx: Some(tycho_tx), components: vec![component] }
}

fn handle_new_gsp(event: NewGsp, tx: &TransactionTrace) -> TransactionProtocolComponents {
    let tycho_tx: Transaction = tx.into();
    let tokens = [event.base_token.as_slice(), event.quote_token.as_slice()];
    let contracts = [event.gsp.as_slice()];
    let component = ProtocolComponent::new(&event.gsp.to_hex())
        .with_tokens(&tokens)
        .with_contracts(&contracts)
        .with_attributes(&[("pool_type", "gsp")])
        .as_swap_type("dodo_v2_pool", ImplementationType::Custom);

    TransactionProtocolComponents { tx: Some(tycho_tx), components: vec![component] }
}

fn handle_remove_gsp(event: RemoveGsp, tx: &TransactionTrace) -> TransactionProtocolComponents {
    let tycho_tx: Transaction = tx.into();
    let contracts = [event.gsp.as_slice()];
    let mut component = ProtocolComponent::new(&event.gsp.to_hex())
        .with_contracts(&contracts)
        .with_attributes(&[("pool_type", "gsp")])
        .as_swap_type("dodo_v2_pool", ImplementationType::Custom);
    component.set_change(ChangeType::Deletion);

    TransactionProtocolComponents { tx: Some(tycho_tx), components: vec![component] }
}

fn get_new_pools(
    params: Params,
    block: &Block,
    new_pools: &mut Vec<TransactionProtocolComponents>,
) {
    let dodo_factories = params.decode_addresses().unwrap();
    let (dpp_factory, dsp_factory, dvm_factory, gsp_factory) = (
        dodo_factories.dpp_factory,
        dodo_factories.dsp_factory,
        dodo_factories.dvm_factory,
        dodo_factories.gsp_factory,
    );
    let mut eh = EventHandler::new(block);
    eh.filter_by_address(vec![
        Address::from_slice(&dpp_factory),
        Address::from_slice(&dsp_factory),
        Address::from_slice(&dvm_factory),
        Address::from_slice(&gsp_factory),
    ]);

    let shared_pools = Rc::new(RefCell::new(Vec::new()));

    {
        let pools_clone = Rc::clone(&shared_pools);
        eh.on::<NewDpp, _>(move |event, tx, log| {
            if log.address == dpp_factory {
                pools_clone
                    .borrow_mut()
                    .push(handle_new_dpp(event, tx));
            }
        });
    }

    {
        let pools_clone = Rc::clone(&shared_pools);
        eh.on::<RemoveDpp, _>(move |event, tx, log| {
            if log.address == dpp_factory {
                pools_clone
                    .borrow_mut()
                    .push(handle_remove_dpp(event, tx));
            }
        });
    }

    {
        let pools_clone = Rc::clone(&shared_pools);
        eh.on::<NewDsp, _>(move |event, tx, log| {
            if log.address == dsp_factory {
                pools_clone
                    .borrow_mut()
                    .push(handle_new_dsp(event, tx));
            }
        });
    }

    {
        let pools_clone = Rc::clone(&shared_pools);
        eh.on::<RemoveDsp, _>(move |event, tx, log| {
            if log.address == dsp_factory {
                pools_clone
                    .borrow_mut()
                    .push(handle_remove_dsp(event, tx));
            }
        });
    }

    {
        let pools_clone = Rc::clone(&shared_pools);
        eh.on::<NewDvm, _>(move |event, tx, log| {
            if log.address == dvm_factory {
                pools_clone
                    .borrow_mut()
                    .push(handle_new_dvm(event, tx));
            }
        });
    }

    {
        let pools_clone = Rc::clone(&shared_pools);
        eh.on::<RemoveDvm, _>(move |event, tx, log| {
            if log.address == dvm_factory {
                pools_clone
                    .borrow_mut()
                    .push(handle_remove_dvm(event, tx));
            }
        });
    }

    {
        let pools_clone = Rc::clone(&shared_pools);
        eh.on::<NewGsp, _>(move |event, tx, log| {
            if log.address == gsp_factory {
                pools_clone
                    .borrow_mut()
                    .push(handle_new_gsp(event, tx));
            }
        });
    }

    {
        let pools_clone = Rc::clone(&shared_pools);
        eh.on::<RemoveGsp, _>(move |event, tx, log| {
            if log.address == gsp_factory {
                pools_clone
                    .borrow_mut()
                    .push(handle_remove_gsp(event, tx));
            }
        });
    }

    eh.handle_events();

    new_pools.append(&mut shared_pools.borrow_mut());
}
