CREATE TABLE api_capabilities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    permission_point_id UUID NOT NULL REFERENCES permission_points(id) ON DELETE CASCADE,
    http_method VARCHAR(10) NOT NULL,
    path_pattern VARCHAR(512) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (http_method, path_pattern)
);

CREATE INDEX idx_apicap_perm ON api_capabilities(permission_point_id);
