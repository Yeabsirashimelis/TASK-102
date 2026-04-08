CREATE TYPE approval_status AS ENUM ('pending', 'approved', 'rejected', 'expired');

CREATE TABLE approval_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    permission_point_id UUID NOT NULL REFERENCES permission_points(id),
    requester_user_id UUID NOT NULL REFERENCES users(id),
    payload JSONB NOT NULL,
    status approval_status NOT NULL DEFAULT 'pending',
    approved_by UUID[] NOT NULL DEFAULT '{}',
    rejected_by UUID,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_appreq_status ON approval_requests(status);
