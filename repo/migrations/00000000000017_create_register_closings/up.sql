CREATE TYPE closing_status AS ENUM ('pending', 'confirmed', 'variance_flagged', 'manager_confirmed');

CREATE TABLE register_closings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    location VARCHAR(128) NOT NULL,
    cashier_user_id UUID NOT NULL REFERENCES users(id),
    closing_date DATE NOT NULL,
    expected_cash_cents BIGINT NOT NULL,
    actual_cash_cents BIGINT NOT NULL,
    expected_card_cents BIGINT NOT NULL,
    actual_card_cents BIGINT NOT NULL,
    expected_gift_card_cents BIGINT NOT NULL,
    actual_gift_card_cents BIGINT NOT NULL,
    variance_cents BIGINT NOT NULL,
    status closing_status NOT NULL DEFAULT 'pending',
    approval_request_id UUID REFERENCES approval_requests(id),
    notes TEXT,
    closed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    confirmed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX idx_closing_unique ON register_closings(location, cashier_user_id, closing_date);
CREATE INDEX idx_closing_status ON register_closings(status);
