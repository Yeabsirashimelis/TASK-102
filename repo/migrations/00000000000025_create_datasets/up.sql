CREATE TYPE dataset_type AS ENUM ('raw', 'cleaned', 'feature', 'result');

CREATE TABLE datasets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(256) NOT NULL UNIQUE,
    description TEXT,
    dataset_type dataset_type NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_datasets_type ON datasets(dataset_type);
CREATE INDEX idx_datasets_name ON datasets(name);
