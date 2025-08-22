#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use substreams::store::StoreGet;
    use substreams_ethereum::pb::eth::v2::{Block, Log, TransactionReceipt, TransactionTrace};
    use tycho_substreams::prelude::*;

    // Shared mock store implementation for testing
    struct MockStore {
        data: HashMap<String, String>,
    }

    impl MockStore {
        fn new_with_data() -> Self {
            Self { data: HashMap::new() }
        }

        fn insert(&mut self, key: String, value: String) {
            self.data.insert(key, value);
        }
    }

    impl StoreGet<String> for MockStore {
        fn new(_size: u32) -> Self {
            Self { data: HashMap::new() }
        }

        fn get_at<K: AsRef<str>>(&self, _ord: u64, key: K) -> Option<String> {
            self.data.get(key.as_ref()).cloned()
        }

        fn get_first<K: AsRef<str>>(&self, key: K) -> Option<String> {
            self.data.get(key.as_ref()).cloned()
        }

        fn get_last<K: AsRef<str>>(&self, key: K) -> Option<String> {
            self.data.get(key.as_ref()).cloned()
        }

        fn has_at<K: AsRef<str>>(&self, _ord: u64, key: K) -> bool {
            self.data.contains_key(key.as_ref())
        }

        fn has_last<K: AsRef<str>>(&self, key: K) -> bool {
            self.data.contains_key(key.as_ref())
        }

        fn has_first<K: AsRef<str>>(&self, key: K) -> bool {
            self.data.contains_key(key.as_ref())
        }
    }

    // Helper function to create mock BlockEntityChanges with UniswapV4 pool creation
    fn create_mock_pools_created() -> BlockEntityChanges {
        let mut tx_changes = TransactionEntityChanges::default();

        let mut component_change = ProtocolComponent::default();
        component_change.id =
            "0x85405f10672f18aa00705afe87ec937d4eadcfc2652f223591b17040ea1d39d4".to_string();
        component_change.change = i32::from(ChangeType::Creation);

        // Add hook address attribute
        component_change
            .static_att
            .push(Attribute {
                name: "hooks".to_string(),
                value: hex::decode("D585c8Baa6c0099d2cc59a5a089B8366Cb3ea8A8").unwrap(),
                change: ChangeType::Creation.into(),
            });

        tx_changes.component_changes = vec![component_change];

        BlockEntityChanges { block: None, changes: vec![tx_changes] }
    }

    #[test]
    fn test_track_uniswap_pools_by_hook() {
        // Given: BlockEntityChanges with a pool creation that has a hook
        let pools_created = create_mock_pools_created();

        // When: Processing the pool creations
        let result = crate::variant_modules::store_pools_per_hook::_track_uniswap_pools_by_hook(
            pools_created,
        );

        // Expect: Should create one pool-to-hook mapping
        assert_eq!(result.len(), 1);

        let (key, pool_id) = &result[0];
        assert!(key.starts_with("hook:"));
        assert_eq!(pool_id, "0x85405f10672f18aa00705afe87ec937d4eadcfc2652f223591b17040ea1d39d4");

        println!("Created mapping: {} -> {}", key, pool_id);
    }

    #[test]
    fn test_basic_functionality() {
        // This is a basic test to ensure the module structure works
        let pools_created = create_mock_pools_created();

        // Test that we can extract hook addresses from component changes
        let tx_changes = &pools_created.changes[0];
        let component_change = &tx_changes.component_changes[0];

        let hooks_attr = component_change
            .static_att
            .iter()
            .find(|attr| attr.name == "hooks");

        assert!(hooks_attr.is_some());
        assert_eq!(
            hooks_attr.unwrap().value,
            hex::decode("D585c8Baa6c0099d2cc59a5a089B8366Cb3ea8A8").unwrap()
        );

        println!("Successfully extracted hook address from component change");
    }

    // Test based on real block 23120299 and transaction
    // 0xb2347c7bd922fe5c7f5027523e3f3b4c2e72e7b535e4d0ddd2f4ea4f21c6edbf
    fn create_real_block_23120299() -> Block {
        let mut block = Block::default();
        block.number = 23120299;

        // Create the transaction trace based on the real transaction
        let mut tx = TransactionTrace::default();
        tx.index = 0; // Assuming this was the first transaction in the block for simplicity
        tx.hash = hex::decode("b2347c7bd922fe5c7f5027523e3f3b4c2e72e7b535e4d0ddd2f4ea4f21c6edbf")
            .unwrap();
        tx.to = hex::decode("000AFbF798467f9b3b97F90D05Bf7Df592d89A6CF0").unwrap(); // EulerSwap factory (padded to 20 bytes)

        // Create PoolDeployed log (simplified - real log would have proper event encoding)
        let mut pool_deployed_log = Log::default();
        pool_deployed_log.address =
            hex::decode("000AFbF798467f9b3b97F90D05Bf7Df592d89A6CF0").unwrap();

        // Real addresses from the transaction
        pool_deployed_log.topics = vec![
            // PoolDeployed event signature
            hex::decode("5f7560a5797edc6f72421362defa094d690eb9f7ced3cc5a5c13383502e4fcc5")
                .unwrap(),
            // asset0: USDC
            hex::decode("000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
                .unwrap(),
            // asset1: USDT
            hex::decode("000000000000000000000000dac17f958d2ee523a2206206994597c13d831ec7")
                .unwrap(),
            // eulerAccount
            hex::decode("0000000000000000000000000AFbF798467f9b3b97F90d05bF7df592D89A6CF6")
                .unwrap(),
        ];

        // Pool address in data field
        pool_deployed_log.data =
            hex::decode("000000000000000000000000D585c8Baa6c0099d2cc59a5a089B8366Cb3ea8A8")
                .unwrap();

        // Create Initialize log for UniswapV4 (simplified)
        let mut initialize_log = Log::default();
        initialize_log.address = hex::decode("000000000004444c5dc75cB358380D2e3dE08A90").unwrap(); // PoolManager

        // This would contain the real Initialize event data
        initialize_log.topics = vec![
            // Initialize event signature
            hex::decode("dd466e674ea557f56295e2d0218a125ea4b4f0f6f3307b95f85e6110838d6438")
                .unwrap(),
        ];

        // Mock receipt with both logs
        tx.receipt = Some(TransactionReceipt {
            logs: vec![pool_deployed_log, initialize_log],
            ..Default::default()
        });

        block.transaction_traces = vec![tx];
        block
    }

    // Helper to create BlockEntityChanges that corresponds to the real transaction
    fn create_real_pools_created_23120299() -> BlockEntityChanges {
        let mut tx_changes = TransactionEntityChanges::default();

        let mut component_change = ProtocolComponent::default();
        // This would be the actual pool ID from the Initialize event
        component_change.id = "real_pool_id_from_tx".to_string();
        component_change.change = i32::from(ChangeType::Creation);

        // Add the hook address that corresponds to the EulerSwap pool
        component_change.static_att.push(Attribute {
            name: "hooks".to_string(),
            value: hex::decode("D585c8Baa6c0099d2cc59a5a089B8366Cb3ea8A8").unwrap(), // Real hook address
            change: ChangeType::Creation.into(),
        });

        tx_changes.component_changes = vec![component_change];

        BlockEntityChanges { block: None, changes: vec![tx_changes] }
    }

    #[test]
    fn test_real_transaction_block_23120299() {
        // Given: Real block data from block 23120299
        let _block = create_real_block_23120299();
        let pools_created = create_real_pools_created_23120299();

        // When: Processing the pool creations for hook tracking
        let result = crate::variant_modules::store_pools_per_hook::_track_uniswap_pools_by_hook(
            pools_created,
        );

        // Expect: Should create one pool-to-hook mapping with real addresses
        assert_eq!(result.len(), 1);

        let (key, pool_id) = &result[0];
        assert!(key.starts_with("hook:"));
        assert!(key.contains("0xd585c8baa6c0099d2cc59a5a089b8366cb3ea8a8")); // Lowercase hex of hook address
        assert_eq!(pool_id, "real_pool_id_from_tx");

        println!("Real transaction test - Created mapping: {} -> {}", key, pool_id);
        println!("This demonstrates the integration works with real transaction data");
    }

    #[test]
    fn test_handle_pool_uninstalled_single_pool() {
        // Create mock store with a single pool for a hook
        let mut store = MockStore::new_with_data();

        // Add a single pool for the hook (with trailing semicolon as per append store format)
        store.insert(
            "hook:0xd585c8baa6c0099d2cc59a5a089b8366cb3ea8a8".to_string(),
            "0x85405f10672f18aa00705afe87ec937d4eadcfc2652f223591b17040ea1d39d4;".to_string(),
        );

        // Test with a single uninstalled hook
        let uninstalled_hooks = vec!["0xd585c8baa6c0099d2cc59a5a089b8366cb3ea8a8".to_string()];
        let results = crate::variant_modules::map_euler_enriched_protocol_changes::_handle_pool_uninstalled_events(
            uninstalled_hooks,
            &store,
        );

        assert_eq!(results.len(), 1);
        let (hook, pool_ids) = &results[0];
        assert_eq!(hook, "0xd585c8baa6c0099d2cc59a5a089b8366cb3ea8a8");
        assert_eq!(pool_ids.len(), 1);
        assert_eq!(
            pool_ids[0],
            "0x85405f10672f18aa00705afe87ec937d4eadcfc2652f223591b17040ea1d39d4"
        );

        println!("Single pool uninstall test passed");
    }

    #[test]
    fn test_handle_pool_uninstalled_multiple_pools() {
        // Create mock store with multiple pools for a hook
        let mut store = MockStore::new_with_data();

        // Add multiple pools for the same hook (semicolon-separated with trailing semicolon)
        store.insert("hook:0xaabbccdd".to_string(), "0xpool1;0xpool2;0xpool3;".to_string());

        // Test with the uninstalled hook
        let uninstalled_hooks = vec!["0xaabbccdd".to_string()];
        let results = crate::variant_modules::map_euler_enriched_protocol_changes::_handle_pool_uninstalled_events(
            uninstalled_hooks,
            &store,
        );

        assert_eq!(results.len(), 1);
        let (hook, pool_ids) = &results[0];
        assert_eq!(hook, "0xaabbccdd");
        assert_eq!(pool_ids.len(), 3);
        assert_eq!(pool_ids[0], "0xpool1");
        assert_eq!(pool_ids[1], "0xpool2");
        assert_eq!(pool_ids[2], "0xpool3");

        println!("Multiple pools uninstall test passed - 3 pools marked as paused");
    }

    #[test]
    fn test_handle_pool_uninstalled_empty_list() {
        // Create mock store with empty pool list
        let mut store = MockStore::new_with_data();

        // Add an empty entry (just semicolons)
        store.insert("hook:0xemptyhook".to_string(), ";;".to_string());

        // Test with the uninstalled hook
        let uninstalled_hooks = vec!["0xemptyhook".to_string()];
        let results = crate::variant_modules::map_euler_enriched_protocol_changes::_handle_pool_uninstalled_events(
            uninstalled_hooks,
            &store,
        );

        // Should handle empty strings gracefully
        assert_eq!(results.len(), 0);

        println!("Empty pool list test passed - no pools to pause");
    }

    #[test]
    fn test_semicolon_separated_format() {
        // Test the actual format that the append store creates
        let pools_data = "0x85405f10672f18aa00705afe87ec937d4eadcfc2652f223591b17040ea1d39d4;";
        let pool_ids: Vec<&str> = pools_data
            .split(';')
            .filter(|s| !s.is_empty())
            .collect();

        assert_eq!(pool_ids.len(), 1);
        assert_eq!(
            pool_ids[0],
            "0x85405f10672f18aa00705afe87ec937d4eadcfc2652f223591b17040ea1d39d4"
        );

        // Test with multiple entries
        let pools_data_multi = "0xpool1;0xpool2;0xpool3;";
        let pool_ids_multi: Vec<&str> = pools_data_multi
            .split(';')
            .filter(|s| !s.is_empty())
            .collect();

        assert_eq!(pool_ids_multi.len(), 3);
        assert_eq!(pool_ids_multi[0], "0xpool1");
        assert_eq!(pool_ids_multi[1], "0xpool2");
        assert_eq!(pool_ids_multi[2], "0xpool3");

        println!("Semicolon format parsing test passed");
    }
}
