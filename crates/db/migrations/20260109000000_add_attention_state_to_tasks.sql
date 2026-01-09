-- Add attention_state field to tasks table for KAP integration
-- This field is derived from .kanban_agent/state.json and cached in DB for query performance

ALTER TABLE tasks ADD COLUMN attention_state TEXT NOT NULL DEFAULT 'normal'
    CHECK (attention_state IN ('normal', 'needs_input', 'risk', 'waiting_external'));

-- Create index for filtering by attention_state (e.g., "only show red cards")
CREATE INDEX idx_tasks_attention_state ON tasks(attention_state);

-- Create index for combined project + attention_state queries
CREATE INDEX idx_tasks_project_attention ON tasks(project_id, attention_state);
