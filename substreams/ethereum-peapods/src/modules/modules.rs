use crate::abi;
use anyhow::Result;
use substreams::store::{StoreGet, StoreGetBigInt, StoreNew, StoreSet, StoreSetProto};
use substreams_ethereum::{
    pb::eth::{self},
    Function,
};
use tycho_substreams::prelude::*;

// #[substreams::handlers::map]
// pub fn map_adapter_static(
//     params: String, // adapter address (hex)
//     _block: eth::v2::Block,
// ) -> Result<BlockTransactionProtocolComponents, anyhow::Error> {
//     // Decode the adapter address from the input parameter.
//     let adapter_address = hex::decode(params).unwrap();

//     // static ProtocolComponent for the adapter.
//     let adapter_component = ProtocolComponent::at_contract(&adapter_address)
//         .with_attributes(&[("role", "adapter"), ("function", "ERC-7815 Adapter")]);

//     // Since the adapter is static, if there will be multiple deployments then tx can be included
//     let tx_component =
//         TransactionProtocolComponents { tx: None, components: vec![adapter_component] };

//     Ok(BlockTransactionProtocolComponents { tx_components: vec![tx_component] })
// }

// This mapping module processes each block, filters for function calls
// to the adapter, and emits ProtocolComponents accordingly.
#[substreams::handlers::map]
pub fn map_protocol_components(
    params: String, // adapter address in hex form
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents, anyhow::Error> {
    // Decode adapter address from parameters.
    let adapter_address = hex::decode(params).unwrap();

    let mut tx_components = vec![];

    for tx in block.transactions() {
        // Iterate over the internal calls in the transaction.
        let components: Vec<ProtocolComponent> = tx
            .calls()
            // Filter calls directed to our adapter contract.
            .filter(|call| call.transaction.to == adapter_address)
            .filter(|call| !call.call.state_reverted)
            .filter_map(|call| {
                // Depending on the function selector, identify the call type.
                if is_swap_call(&call.call) {
                    let swap_call = abi::swap_adapter::functions::Swap2::match_and_decode(call)?;

                    let (addr1, addr2) =
                        canonicalize_addresses(swap_call.sell_token, swap_call.buy_token);

                    Some(ProtocolComponent {
                        id: key_from_tokens(addr1.clone(), addr2.clone()),
                        tokens: vec![addr1, addr2],
                        contracts: vec![],
                        static_att: vec![],
                        protocol_type: Some(ProtocolType {
                            name: "adapter_pool".to_string(),
                            financial_type: FinancialType::Swap.into(),
                            attribute_schema: vec![],
                            implementation_type: ImplementationType::Vm.into(),
                        }),
                        change: ChangeType::Creation.into(),
                    })
                } else if is_swap_to_price_call(&call.call) {
                    let swap_to_price_call =
                        abi::swap_adapter::functions::SwapToPrice::match_and_decode(call)?;

                    let (addr1, addr2) = canonicalize_addresses(
                        swap_to_price_call.sell_token,
                        swap_to_price_call.buy_token,
                    );

                    Some(ProtocolComponent {
                        id: key_from_tokens(addr1.clone(), addr2.clone()),
                        tokens: vec![addr1, addr2],
                        contracts: vec![],
                        static_att: vec![],
                        protocol_type: Some(ProtocolType {
                            name: "adapter_pool".to_string(),
                            financial_type: FinancialType::Swap.into(),
                            attribute_schema: vec![],
                            implementation_type: ImplementationType::Vm.into(),
                        }),
                        change: ChangeType::Creation.into(),
                    })
                } else if is_price_call(&call.call) {
                    let price_call = abi::swap_adapter::functions::Price::match_and_decode(call)?;

                    let (addr1, addr2) =
                        canonicalize_addresses(price_call.sell_token, price_call.buy_token);

                    Some(ProtocolComponent {
                        id: key_from_tokens(addr1.clone(), addr2.clone()),
                        tokens: vec![addr1, addr2],
                        contracts: vec![],
                        static_att: vec![],
                        protocol_type: Some(ProtocolType {
                            name: "adapter_pool".to_string(),
                            financial_type: FinancialType::Swap.into(),
                            attribute_schema: vec![],
                            implementation_type: ImplementationType::Vm.into(),
                        }),
                        change: ChangeType::Creation.into(),
                    })
                } else {
                    None
                }
            })
            .collect();

        if !components.is_empty() {
            tx_components.push(TransactionProtocolComponents { tx: Some(tx.into()), components });
        }
    }

    Ok(BlockTransactionProtocolComponents { tx_components })
}

#[substreams::handlers::store]
pub fn store_adapter_token_pairs(
    components: BlockTransactionProtocolComponents,
    component_store: StoreSetProto<ProtocolComponent>,
) {
    for tx_component in components.tx_components {
        for component in tx_component.components {
            component_store.set(0, &component.id, &component);
        }
    }
}

#[substreams::handlers::map]
pub fn map_adapter_token_delta(
    params: String,
    block: eth::v2::Block,
    _store: StoreGetBigInt,
) -> Result<BlockBalanceDeltas> {
    Ok(BlockBalanceDeltas { balance_deltas: vec![] })
}
