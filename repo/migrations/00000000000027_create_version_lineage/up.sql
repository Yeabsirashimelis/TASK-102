CREATE TABLE version_lineage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    child_version_id UUID NOT NULL REFERENCES dataset_versions(id) ON DELETE CASCADE,
    parent_version_id UUID NOT NULL REFERENCES dataset_versions(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (child_version_id, parent_version_id)
);

CREATE INDEX idx_lineage_child ON version_lineage(child_version_id);
CREATE INDEX idx_lineage_parent ON version_lineage(parent_version_id);
