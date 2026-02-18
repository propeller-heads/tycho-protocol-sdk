use std::collections::HashMap;
use substreams_ethereum::pb::eth;
use tycho_substreams::models::{
    BlockTransactionProtocolComponents, ChangeType, FinancialType, ImplementationType,
    ProtocolComponent, ProtocolType, Transaction, TransactionProtocolComponents,
};

use crate::abi::swap_adapter::functions::{Price, Swap2, SwapToPrice};

use super::{
    config::DeploymentConfig,
    helper_funcs::{canonicalize_addresses, key_from_tokens},
};

struct ProtocolComponentWithTransaction {
    proto_comp: ProtocolComponent,
    tx: Transaction,
}

#[substreams::handlers::map]
/// Find and create all relevant protocol components
///
/// This method maps over blocks and instantiates ProtocolComponents with a unique ids
/// as well as all necessary metadata for routing and encoding.
#[substreams::handlers::map]
fn map_token_pairs(
    params: String, // adapter address in hex form
    block: eth::v2::Block,
) -> Result<BlockTransactionProtocolComponents, anyhow::Error> {
    // Decode adapter address from parameters.
    let config: DeploymentConfig = serde_qs::from_str(params.as_str())?;
    let adapter_address = config.adapter_address;

    let mut proto_comp_with_txn: HashMap<String, ProtocolComponentWithTransaction> = HashMap::new();

    for tx in block.transactions() {
        // Iterate over the internal calls in the transaction.
        tx.calls()
            // Filter calls directed to our adapter contract.
            .filter(|call| call.transaction.to == adapter_address)
            .filter(|call| !call.call.state_reverted)
            .for_each(|call| {
                if Swap2::match_call(call.call) {
                    let swap_call = Swap2::decode(call.call).unwrap();
                    let (addr0, addr1) = canonicalize_addresses(
                        swap_call.sell_token.clone(),
                        swap_call.buy_token.clone(),
                    );
                    let component_id = key_from_tokens(&addr0, &addr1);

                    if proto_comp_with_txn.contains_key(&component_id) {
                        return;
                    }

                    proto_comp_with_txn.insert(
                        component_id.clone(),
                        ProtocolComponentWithTransaction {
                            proto_comp: ProtocolComponent {
                                id: component_id,
                                tokens: vec![addr0, addr1],
                                contracts: vec![adapter_address.clone()],
                                static_att: vec![],
                                change: ChangeType::Creation.into(),
                                protocol_type: Some(ProtocolType {
                                    name: "Adapter Token Pair".to_string(),
                                    financial_type: FinancialType::Swap.into(),
                                    attribute_schema: vec![],
                                    implementation_type: ImplementationType::Vm.into(),
                                }),
                            },
                            tx: Transaction {
                                hash: call.transaction.hash.clone(),
                                from: call.transaction.from.clone(),
                                to: call.transaction.to.clone(),
                                index: call.transaction.index as u64,
                            },
                        },
                    );
                } else if SwapToPrice::match_call(&call.call) {
                    let swap_to_price_call = SwapToPrice::decode(call.call).unwrap();
                    let (addr0, addr1) = canonicalize_addresses(
                        swap_to_price_call.sell_token.clone(),
                        swap_to_price_call.buy_token.clone(),
                    );
                    let component_id = key_from_tokens(&addr0, &addr1);

                    if proto_comp_with_txn.contains_key(&component_id) {
                        return;
                    }

                    proto_comp_with_txn.insert(
                        component_id.clone(),
                        ProtocolComponentWithTransaction {
                            proto_comp: ProtocolComponent {
                                id: component_id,
                                tokens: vec![addr0, addr1],
                                contracts: vec![adapter_address.clone()],
                                static_att: vec![],
                                change: ChangeType::Creation.into(),
                                protocol_type: Some(ProtocolType {
                                    name: "Adapter Token Pair".to_string(),
                                    financial_type: FinancialType::Swap.into(),
                                    attribute_schema: vec![],
                                    implementation_type: ImplementationType::Vm.into(),
                                }),
                            },
                            tx: Transaction {
                                hash: call.transaction.hash.clone(),
                                from: call.transaction.from.clone(),
                                to: call.transaction.to.clone(),
                                index: call.transaction.index as u64,
                            },
                        },
                    );
                } else if Price::match_call(&call.call) {
                    let price_call = Price::decode(call.call).unwrap();
                    let (addr0, addr1) = canonicalize_addresses(
                        price_call.sell_token.clone(),
                        price_call.buy_token.clone(),
                    );
                    let component_id = key_from_tokens(&addr0, &addr1);

                    if proto_comp_with_txn.contains_key(&component_id) {
                        return;
                    }

                    proto_comp_with_txn.insert(
                        component_id.clone(),
                        ProtocolComponentWithTransaction {
                            proto_comp: ProtocolComponent {
                                id: component_id,
                                tokens: vec![addr0, addr1],
                                contracts: vec![adapter_address.clone()],
                                static_att: vec![],
                                change: ChangeType::Creation.into(),
                                protocol_type: Some(ProtocolType {
                                    name: "Adapter Token Pair".to_string(),
                                    financial_type: FinancialType::Swap.into(),
                                    attribute_schema: vec![],
                                    implementation_type: ImplementationType::Vm.into(),
                                }),
                            },
                            tx: Transaction {
                                hash: call.transaction.hash.clone(),
                                from: call.transaction.from.clone(),
                                to: call.transaction.to.clone(),
                                index: call.transaction.index as u64,
                            },
                        },
                    );
                } else {
                    return;
                }
            });
    }

    let mut protocol_components: Vec<TransactionProtocolComponents> = vec![];
    for (_, proto_comp_with_txn) in proto_comp_with_txn {
        protocol_components.push(TransactionProtocolComponents {
            components: vec![proto_comp_with_txn.proto_comp],
            tx: Some(proto_comp_with_txn.tx),
        });
    }

    Ok(BlockTransactionProtocolComponents { tx_components: protocol_components })
}
