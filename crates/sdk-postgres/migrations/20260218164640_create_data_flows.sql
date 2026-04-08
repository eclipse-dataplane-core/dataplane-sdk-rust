CREATE TYPE data_flow_state AS ENUM ('started','suspended','terminated','completed', 'initiating', 'initiated', 'preparing');
CREATE TYPE data_flow_type AS ENUM ('consumer','provider');


CREATE TABLE IF NOT EXISTS data_flows (
    id TEXT PRIMARY KEY,
    participant_context_id TEXT NOT NULL,
    transfer_type TEXT NOT NULL,
    agreement_id TEXT NOT NULL,
    dataset_id TEXT NOT NULL,
    participant_id TEXT NOT NULL,
    dataspace_context TEXT NOT NULL,
    counter_party_id TEXT NOT NULL,
    callback_address TEXT NOT NULL,
    state data_flow_state NOT NULL,
    type data_flow_type NOT NULL,
    data_address JSONB ,
    suspension_reason TEXT,
    termination_reason TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    labels JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
)
