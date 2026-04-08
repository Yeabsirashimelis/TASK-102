CREATE TYPE tender_type AS ENUM ('cash', 'card', 'gift_card');
CREATE TYPE ledger_entry_kind AS ENUM ('payment', 'refund', 'reversal');

CREATE TABLE ledger_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES orders(id),
    tender_type tender_type NOT NULL,
    entry_kind ledger_entry_kind NOT NULL DEFAULT 'payment',
    amount_cents BIGINT NOT NULL,
    reference_code VARCHAR(256),
    idempotency_key UUID NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_ledger_order ON ledger_entries(order_id);
CREATE UNIQUE INDEX idx_ledger_idempotency ON ledger_entries(idempotency_key);
