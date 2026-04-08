CREATE TABLE delegations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    delegator_user_id UUID NOT NULL REFERENCES users(id),
    delegate_user_id UUID NOT NULL REFERENCES users(id),
    permission_point_id UUID NOT NULL REFERENCES permission_points(id),
    source_department VARCHAR(128),
    target_department VARCHAR(128),
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (ends_at > starts_at)
);

CREATE INDEX idx_deleg_delegate ON delegations(delegate_user_id);
CREATE INDEX idx_deleg_active ON delegations(is_active, starts_at, ends_at);
