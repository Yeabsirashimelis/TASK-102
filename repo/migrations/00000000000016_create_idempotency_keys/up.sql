CREATE TABLE idempotency_keys (
    key UUID PRIMARY KEY,
    resource_type VARCHAR(64) NOT NULL,
    resource_id UUID NOT NULL,
    response_status SMALLINT NOT NULL,
    response_body JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT now() + INTERVAL '24 hours'
);

CREATE INDEX idx_idemp_expires ON idempotency_keys(expires_at);
