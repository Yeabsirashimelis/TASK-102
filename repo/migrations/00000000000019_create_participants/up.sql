CREATE TABLE participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    first_name VARCHAR(128) NOT NULL,
    last_name VARCHAR(128) NOT NULL,
    email VARCHAR(256),
    phone VARCHAR(64),
    department VARCHAR(128),
    location VARCHAR(128),
    employee_id VARCHAR(64),
    notes TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_participants_name ON participants(last_name, first_name);
CREATE INDEX idx_participants_dept ON participants(department);
CREATE INDEX idx_participants_location ON participants(location);
CREATE INDEX idx_participants_employee ON participants(employee_id);
CREATE INDEX idx_participants_active ON participants(is_active);
