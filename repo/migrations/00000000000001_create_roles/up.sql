CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TYPE data_scope_enum AS ENUM ('department', 'location', 'individual');

CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(128) NOT NULL UNIQUE,
    description TEXT,
    data_scope data_scope_enum NOT NULL DEFAULT 'individual',
    scope_value VARCHAR(256),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
