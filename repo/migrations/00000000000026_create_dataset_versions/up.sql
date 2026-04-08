CREATE TABLE dataset_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id UUID NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    storage_path VARCHAR(1024) NOT NULL,
    file_size_bytes BIGINT,
    sha256_hash VARCHAR(64),
    row_count BIGINT,
    transformation_note TEXT,
    is_current BOOLEAN NOT NULL DEFAULT TRUE,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (dataset_id, version_number)
);

CREATE INDEX idx_dsv_dataset ON dataset_versions(dataset_id);
CREATE INDEX idx_dsv_current ON dataset_versions(dataset_id, is_current) WHERE is_current = TRUE;
CREATE INDEX idx_dsv_created ON dataset_versions(created_at);
