CREATE TABLE approval_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    permission_point_id UUID NOT NULL REFERENCES permission_points(id) ON DELETE CASCADE,
    min_approvers INTEGER NOT NULL DEFAULT 1,
    approver_role_id UUID NOT NULL REFERENCES roles(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (permission_point_id, approver_role_id)
);
