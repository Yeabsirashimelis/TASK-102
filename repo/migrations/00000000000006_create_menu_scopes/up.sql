CREATE TABLE menu_scopes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    permission_point_id UUID NOT NULL REFERENCES permission_points(id) ON DELETE CASCADE,
    menu_key VARCHAR(256) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (permission_point_id, menu_key)
);
