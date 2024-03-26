CREATE TABLE IF NOT EXISTS factory_pair_created (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" DECIMAL,
    "pair" VARCHAR(40),
    "param3" DECIMAL,
    "token0" VARCHAR(40),
    "token1" VARCHAR(40),
    PRIMARY KEY(evt_tx_hash,evt_index)
);
CREATE TABLE IF NOT EXISTS factory_call_create_pair1 (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    "fee" DECIMAL,
    "output_pair" VARCHAR(40),
    "token_a" VARCHAR(40),
    "token_b" VARCHAR(40),
    PRIMARY KEY(call_tx_hash,call_ordinal)
);
CREATE TABLE IF NOT EXISTS factory_call_create_pair2 (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    "output_pair" VARCHAR(40),
    "token_a" VARCHAR(40),
    "token_b" VARCHAR(40),
    PRIMARY KEY(call_tx_hash,call_ordinal)
);
CREATE TABLE IF NOT EXISTS factory_call_set_fee_to (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    "u_fee_to" VARCHAR(40),
    PRIMARY KEY(call_tx_hash,call_ordinal)
);
CREATE TABLE IF NOT EXISTS factory_call_set_fee_to_setter (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    "u_fee_to_setter" VARCHAR(40),
    PRIMARY KEY(call_tx_hash,call_ordinal)
);
CREATE TABLE IF NOT EXISTS factory_call_toggle_global_pause (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    PRIMARY KEY(call_tx_hash,call_ordinal)
);


CREATE TABLE IF NOT EXISTS pair_approval (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" DECIMAL,
    "evt_address" VARCHAR(40),
    "owner" VARCHAR(40),
    "spender" VARCHAR(40),
    "value" DECIMAL,
    PRIMARY KEY(evt_tx_hash,evt_index)
);
CREATE TABLE IF NOT EXISTS pair_burn (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" DECIMAL,
    "evt_address" VARCHAR(40),
    "amount0" DECIMAL,
    "amount1" DECIMAL,
    "sender" VARCHAR(40),
    "to" VARCHAR(40),
    PRIMARY KEY(evt_tx_hash,evt_index)
);
CREATE TABLE IF NOT EXISTS pair_cancel_long_term_order (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" DECIMAL,
    "evt_address" VARCHAR(40),
    "addr" VARCHAR(40),
    "buy_token" VARCHAR(40),
    "order_id" DECIMAL,
    "purchased_amount" DECIMAL,
    "sell_token" VARCHAR(40),
    "unsold_amount" DECIMAL,
    PRIMARY KEY(evt_tx_hash,evt_index)
);
CREATE TABLE IF NOT EXISTS pair_long_term_swap0_to1 (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" DECIMAL,
    "evt_address" VARCHAR(40),
    "addr" VARCHAR(40),
    "amount0_in" DECIMAL,
    "number_of_time_intervals" DECIMAL,
    "order_id" DECIMAL,
    PRIMARY KEY(evt_tx_hash,evt_index)
);
CREATE TABLE IF NOT EXISTS pair_long_term_swap1_to0 (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" DECIMAL,
    "evt_address" VARCHAR(40),
    "addr" VARCHAR(40),
    "amount1_in" DECIMAL,
    "number_of_time_intervals" DECIMAL,
    "order_id" DECIMAL,
    PRIMARY KEY(evt_tx_hash,evt_index)
);
CREATE TABLE IF NOT EXISTS pair_lp_fee_updated (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" DECIMAL,
    "evt_address" VARCHAR(40),
    "fee" DECIMAL,
    PRIMARY KEY(evt_tx_hash,evt_index)
);
CREATE TABLE IF NOT EXISTS pair_mint (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" DECIMAL,
    "evt_address" VARCHAR(40),
    "amount0" DECIMAL,
    "amount1" DECIMAL,
    "sender" VARCHAR(40),
    PRIMARY KEY(evt_tx_hash,evt_index)
);
CREATE TABLE IF NOT EXISTS pair_swap (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" DECIMAL,
    "evt_address" VARCHAR(40),
    "amount0_in" DECIMAL,
    "amount0_out" DECIMAL,
    "amount1_in" DECIMAL,
    "amount1_out" DECIMAL,
    "sender" VARCHAR(40),
    "to" VARCHAR(40),
    PRIMARY KEY(evt_tx_hash,evt_index)
);
CREATE TABLE IF NOT EXISTS pair_sync (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" DECIMAL,
    "evt_address" VARCHAR(40),
    "reserve0" DECIMAL,
    "reserve1" DECIMAL,
    PRIMARY KEY(evt_tx_hash,evt_index)
);
CREATE TABLE IF NOT EXISTS pair_transfer (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" DECIMAL,
    "evt_address" VARCHAR(40),
    "from" VARCHAR(40),
    "to" VARCHAR(40),
    "value" DECIMAL,
    PRIMARY KEY(evt_tx_hash,evt_index)
);
CREATE TABLE IF NOT EXISTS pair_withdraw_proceeds_from_long_term_order (
    "evt_tx_hash" VARCHAR(64),
    "evt_index" INT,
    "evt_block_time" TIMESTAMP,
    "evt_block_number" DECIMAL,
    "evt_address" VARCHAR(40),
    "addr" VARCHAR(40),
    "order_expired" BOOL,
    "order_id" DECIMAL,
    "proceed_token" VARCHAR(40),
    "proceeds" DECIMAL,
    PRIMARY KEY(evt_tx_hash,evt_index)
);CREATE TABLE IF NOT EXISTS pair_call_approve (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "output_param0" BOOL,
    "spender" VARCHAR(40),
    "value" DECIMAL,
    PRIMARY KEY(call_tx_hash,call_ordinal)
);
CREATE TABLE IF NOT EXISTS pair_call_burn (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "output_amount0" DECIMAL,
    "output_amount1" DECIMAL,
    "to" VARCHAR(40),
    PRIMARY KEY(call_tx_hash,call_ordinal)
);
CREATE TABLE IF NOT EXISTS pair_call_cancel_long_term_swap (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "order_id" DECIMAL,
    PRIMARY KEY(call_tx_hash,call_ordinal)
);
CREATE TABLE IF NOT EXISTS pair_call_execute_virtual_orders (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "block_timestamp" DECIMAL,
    PRIMARY KEY(call_tx_hash,call_ordinal)
);
CREATE TABLE IF NOT EXISTS pair_call_get_twamm_order_proceeds (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "order_id" DECIMAL,
    "output_order_expired" BOOL,
    "output_total_reward" DECIMAL,
    PRIMARY KEY(call_tx_hash,call_ordinal)
);
CREATE TABLE IF NOT EXISTS pair_call_initialize (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "u_fee" DECIMAL,
    "u_token0" VARCHAR(40),
    "u_token1" VARCHAR(40),
    PRIMARY KEY(call_tx_hash,call_ordinal)
);
CREATE TABLE IF NOT EXISTS pair_call_long_term_swap_from0_to1 (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "amount0_in" DECIMAL,
    "number_of_time_intervals" DECIMAL,
    "output_order_id" DECIMAL,
    PRIMARY KEY(call_tx_hash,call_ordinal)
);
CREATE TABLE IF NOT EXISTS pair_call_long_term_swap_from1_to0 (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "amount1_in" DECIMAL,
    "number_of_time_intervals" DECIMAL,
    "output_order_id" DECIMAL,
    PRIMARY KEY(call_tx_hash,call_ordinal)
);
CREATE TABLE IF NOT EXISTS pair_call_mint (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "output_liquidity" DECIMAL,
    "to" VARCHAR(40),
    PRIMARY KEY(call_tx_hash,call_ordinal)
);
CREATE TABLE IF NOT EXISTS pair_call_permit (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "deadline" DECIMAL,
    "owner" VARCHAR(40),
    "r" TEXT,
    "s" TEXT,
    "spender" VARCHAR(40),
    "v" INT,
    "value" DECIMAL,
    PRIMARY KEY(call_tx_hash,call_ordinal)
);
CREATE TABLE IF NOT EXISTS pair_call_set_fee (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "new_fee" DECIMAL,
    PRIMARY KEY(call_tx_hash,call_ordinal)
);
CREATE TABLE IF NOT EXISTS pair_call_skim (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "to" VARCHAR(40),
    PRIMARY KEY(call_tx_hash,call_ordinal)
);
CREATE TABLE IF NOT EXISTS pair_call_swap (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "amount0_out" DECIMAL,
    "amount1_out" DECIMAL,
    "data" TEXT,
    "to" VARCHAR(40),
    PRIMARY KEY(call_tx_hash,call_ordinal)
);
CREATE TABLE IF NOT EXISTS pair_call_sync (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    PRIMARY KEY(call_tx_hash,call_ordinal)
);
CREATE TABLE IF NOT EXISTS pair_call_toggle_pause_new_swaps (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    PRIMARY KEY(call_tx_hash,call_ordinal)
);
CREATE TABLE IF NOT EXISTS pair_call_transfer (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "output_param0" BOOL,
    "to" VARCHAR(40),
    "value" DECIMAL,
    PRIMARY KEY(call_tx_hash,call_ordinal)
);
CREATE TABLE IF NOT EXISTS pair_call_transfer_from (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "from" VARCHAR(40),
    "output_param0" BOOL,
    "to" VARCHAR(40),
    "value" DECIMAL,
    PRIMARY KEY(call_tx_hash,call_ordinal)
);
CREATE TABLE IF NOT EXISTS pair_call_withdraw_proceeds_from_long_term_swap (
    "call_tx_hash" VARCHAR(64),
    "call_block_time" TIMESTAMP,
    "call_block_number" DECIMAL,
    "call_ordinal" INT,
    "call_success" BOOL,
    "call_address" VARCHAR(40),
    "order_id" DECIMAL,
    "output_is_expired" BOOL,
    "output_reward_tkn" VARCHAR(40),
    "output_total_reward" DECIMAL,
    PRIMARY KEY(call_tx_hash,call_ordinal)
);


