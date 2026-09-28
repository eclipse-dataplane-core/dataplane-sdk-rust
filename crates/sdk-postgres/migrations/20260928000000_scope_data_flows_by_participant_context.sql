-- Flow ids are unique per participant context, not globally. Keying the table on the pair keeps
-- one participant context from colliding with, or probing for, another context's flow ids.
ALTER TABLE data_flows DROP CONSTRAINT IF EXISTS data_flows_pkey;
ALTER TABLE data_flows ADD PRIMARY KEY (participant_context_id, id);
