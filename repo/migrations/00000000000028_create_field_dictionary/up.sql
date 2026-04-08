CREATE TABLE field_dictionaries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    version_id UUID NOT NULL REFERENCES dataset_versions(id) ON DELETE CASCADE,
    field_name VARCHAR(256) NOT NULL,
    field_type VARCHAR(128) NOT NULL,
    meaning TEXT,
    source_system VARCHAR(256),
    last_updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (version_id, field_name)
);

CREATE INDEX idx_fd_version ON field_dictionaries(version_id);
