CREATE TABLE IF NOT EXISTS factory_pair_created (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" UInt64,
    "pair" VARCHAR(40),
    "param3" UInt256,
    "token0" VARCHAR(40),
    "token1" VARCHAR(40)
) ENGINE = MergeTree PRIMARY KEY ("evt_tx_hash","evt_index");

CREATE TABLE IF NOT EXISTS factory_call_create_pair1 (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL,
    "fee" UInt256,
    "output_pair" VARCHAR(40),
    "token_a" VARCHAR(40),
    "token_b" VARCHAR(40)
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
CREATE TABLE IF NOT EXISTS factory_call_create_pair2 (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL,
    "output_pair" VARCHAR(40),
    "token_a" VARCHAR(40),
    "token_b" VARCHAR(40)
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
CREATE TABLE IF NOT EXISTS factory_call_set_fee_to (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL,
    "u_fee_to" VARCHAR(40)
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
CREATE TABLE IF NOT EXISTS factory_call_set_fee_to_setter (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL,
    "u_fee_to_setter" VARCHAR(40)
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
CREATE TABLE IF NOT EXISTS factory_call_toggle_global_pause (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
CREATE TABLE IF NOT EXISTS pair_approval (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" UInt64,
    "evt_address" VARCHAR(40),
    "owner" VARCHAR(40),
    "spender" VARCHAR(40),
    "value" UInt256
) ENGINE = MergeTree PRIMARY KEY ("evt_tx_hash","evt_index");
CREATE TABLE IF NOT EXISTS pair_burn (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" UInt64,
    "evt_address" VARCHAR(40),
    "amount0" UInt256,
    "amount1" UInt256,
    "sender" VARCHAR(40),
    "to" VARCHAR(40)
) ENGINE = MergeTree PRIMARY KEY ("evt_tx_hash","evt_index");
CREATE TABLE IF NOT EXISTS pair_cancel_long_term_order (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" UInt64,
    "evt_address" VARCHAR(40),
    "addr" VARCHAR(40),
    "buy_token" VARCHAR(40),
    "order_id" UInt256,
    "purchased_amount" UInt256,
    "sell_token" VARCHAR(40),
    "unsold_amount" UInt256
) ENGINE = MergeTree PRIMARY KEY ("evt_tx_hash","evt_index");
CREATE TABLE IF NOT EXISTS pair_long_term_swap0_to1 (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" UInt64,
    "evt_address" VARCHAR(40),
    "addr" VARCHAR(40),
    "amount0_in" UInt256,
    "number_of_time_intervals" UInt256,
    "order_id" UInt256
) ENGINE = MergeTree PRIMARY KEY ("evt_tx_hash","evt_index");
CREATE TABLE IF NOT EXISTS pair_long_term_swap1_to0 (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" UInt64,
    "evt_address" VARCHAR(40),
    "addr" VARCHAR(40),
    "amount1_in" UInt256,
    "number_of_time_intervals" UInt256,
    "order_id" UInt256
) ENGINE = MergeTree PRIMARY KEY ("evt_tx_hash","evt_index");
CREATE TABLE IF NOT EXISTS pair_lp_fee_updated (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" UInt64,
    "evt_address" VARCHAR(40),
    "fee" UInt256
) ENGINE = MergeTree PRIMARY KEY ("evt_tx_hash","evt_index");
CREATE TABLE IF NOT EXISTS pair_mint (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" UInt64,
    "evt_address" VARCHAR(40),
    "amount0" UInt256,
    "amount1" UInt256,
    "sender" VARCHAR(40)
) ENGINE = MergeTree PRIMARY KEY ("evt_tx_hash","evt_index");
CREATE TABLE IF NOT EXISTS pair_swap (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" UInt64,
    "evt_address" VARCHAR(40),
    "amount0_in" UInt256,
    "amount0_out" UInt256,
    "amount1_in" UInt256,
    "amount1_out" UInt256,
    "sender" VARCHAR(40),
    "to" VARCHAR(40)
) ENGINE = MergeTree PRIMARY KEY ("evt_tx_hash","evt_index");
CREATE TABLE IF NOT EXISTS pair_sync (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" UInt64,
    "evt_address" VARCHAR(40),
    "reserve0" UInt128,
    "reserve1" UInt128
) ENGINE = MergeTree PRIMARY KEY ("evt_tx_hash","evt_index");
CREATE TABLE IF NOT EXISTS pair_transfer (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" UInt64,
    "evt_address" VARCHAR(40),
    "from" VARCHAR(40),
    "to" VARCHAR(40),
    "value" UInt256
) ENGINE = MergeTree PRIMARY KEY ("evt_tx_hash","evt_index");
CREATE TABLE IF NOT EXISTS pair_withdraw_proceeds_from_long_term_order (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" UInt64,
    "evt_address" VARCHAR(40),
    "addr" VARCHAR(40),
    "order_expired" BOOL,
    "order_id" UInt256,
    "proceed_token" VARCHAR(40),
    "proceeds" UInt256
) ENGINE = MergeTree PRIMARY KEY ("evt_tx_hash","evt_index");
CREATE TABLE IF NOT EXISTS pair_call_approve (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "output_param0" BOOL,
    "spender" VARCHAR(40),
    "value" UInt256
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
CREATE TABLE IF NOT EXISTS pair_call_burn (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "output_amount0" UInt256,
    "output_amount1" UInt256,
    "to" VARCHAR(40)
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
CREATE TABLE IF NOT EXISTS pair_call_cancel_long_term_swap (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "order_id" UInt256
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
CREATE TABLE IF NOT EXISTS pair_call_execute_virtual_orders (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "block_timestamp" UInt256
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
CREATE TABLE IF NOT EXISTS pair_call_get_twamm_order_proceeds (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "order_id" UInt256,
    "output_order_expired" BOOL,
    "output_total_reward" UInt256
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
CREATE TABLE IF NOT EXISTS pair_call_initialize (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "u_fee" UInt256,
    "u_token0" VARCHAR(40),
    "u_token1" VARCHAR(40)
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
CREATE TABLE IF NOT EXISTS pair_call_long_term_swap_from0_to1 (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "amount0_in" UInt256,
    "number_of_time_intervals" UInt256,
    "output_order_id" UInt256
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
CREATE TABLE IF NOT EXISTS pair_call_long_term_swap_from1_to0 (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "amount1_in" UInt256,
    "number_of_time_intervals" UInt256,
    "output_order_id" UInt256
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
CREATE TABLE IF NOT EXISTS pair_call_mint (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "output_liquidity" UInt256,
    "to" VARCHAR(40)
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
CREATE TABLE IF NOT EXISTS pair_call_permit (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "deadline" UInt256,
    "owner" VARCHAR(40),
    "r" TEXT,
    "s" TEXT,
    "spender" VARCHAR(40),
    "v" UInt8,
    "value" UInt256
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
CREATE TABLE IF NOT EXISTS pair_call_set_fee (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "new_fee" UInt256
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
CREATE TABLE IF NOT EXISTS pair_call_skim (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "to" VARCHAR(40)
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
CREATE TABLE IF NOT EXISTS pair_call_swap (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "amount0_out" UInt256,
    "amount1_out" UInt256,
    "data" TEXT,
    "to" VARCHAR(40)
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
CREATE TABLE IF NOT EXISTS pair_call_sync (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40)
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
CREATE TABLE IF NOT EXISTS pair_call_toggle_pause_new_swaps (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40)
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
CREATE TABLE IF NOT EXISTS pair_call_transfer (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "output_param0" BOOL,
    "to" VARCHAR(40),
    "value" UInt256
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
CREATE TABLE IF NOT EXISTS pair_call_transfer_from (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "from" VARCHAR(40),
    "output_param0" BOOL,
    "to" VARCHAR(40),
    "value" UInt256
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
CREATE TABLE IF NOT EXISTS pair_call_withdraw_proceeds_from_long_term_swap (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" UInt64,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "order_id" UInt256,
    "output_is_expired" BOOL,
    "output_reward_tkn" VARCHAR(40),
    "output_total_reward" UInt256
) ENGINE = MergeTree PRIMARY KEY ("call_tx_hash","call_ordinal");
