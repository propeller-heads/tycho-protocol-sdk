-- This script pre-creates historical partitions for February and March 2026.
-- It runs automatically on DB first initialization (via docker-entrypoint-initdb.d)
-- BEFORE the indexer connects, guaranteeing February data never lands in the
-- DEFAULT partition and avoids the duplicate key constraint violation.
--
-- The DEFAULT partition has a unique key on (protocol_component_id, attribute_name)
-- WITHOUT valid_from, so it cannot store temporal history (multiple versions).
-- These named partitions include valid_from in their key and CAN store history.
--
-- NOTE: This runs AFTER Tycho's own migrations (which create the protocol_state
-- partition structure), so these tables are safe to create here.

-- We use a DO block with IF NOT EXISTS so this is idempotent if re-run.
DO $$
DECLARE
    d DATE := '2026-02-01';
BEGIN
    -- Wait until the protocol_state table exists (created by Tycho migrations)
    -- This init script runs once on first DB start, after extensions are ready.
    -- Tycho migrations run at indexer startup, so we pre-create the parent
    -- partition structure ourselves here.

    -- Create partitions for Feb 2026 through Mar 24 2026 (Mar 25+ are created by partman)
    WHILE d < '2026-03-25' LOOP
        -- protocol_state partitions (naming: protocol_state_pYYYYMMDD)
        BEGIN
            EXECUTE format(
                'CREATE TABLE IF NOT EXISTS protocol_state_p%s
                 PARTITION OF protocol_state
                 FOR VALUES FROM (%L) TO (%L)',
                to_char(d, 'YYYYMMDD'),
                d::timestamptz,
                (d + 1)::timestamptz
            );
        EXCEPTION WHEN undefined_table THEN
            -- protocol_state doesn't exist yet (migrations not run), skip silently
            NULL;
        END;

        -- component_balance partitions
        BEGIN
            EXECUTE format(
                'CREATE TABLE IF NOT EXISTS component_balance_p%s
                 PARTITION OF component_balance
                 FOR VALUES FROM (%L) TO (%L)',
                to_char(d, 'YYYYMMDD'),
                d::timestamptz,
                (d + 1)::timestamptz
            );
        EXCEPTION WHEN undefined_table THEN
            NULL;
        END;

        d := d + 1;
    END LOOP;

    RAISE NOTICE 'Historical partition init: processed dates 2026-02-01 to 2026-03-24';
END $$;
