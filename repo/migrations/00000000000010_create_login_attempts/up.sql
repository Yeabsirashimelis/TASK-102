CREATE TABLE login_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(128) NOT NULL,
    success BOOLEAN NOT NULL,
    ip_address VARCHAR(45),
    attempted_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_login_username ON login_attempts(username, attempted_at);
