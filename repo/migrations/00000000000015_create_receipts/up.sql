CREATE TABLE receipts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES orders(id),
    receipt_number VARCHAR(128) NOT NULL UNIQUE,
    receipt_data JSONB NOT NULL,
    printed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID NOT NULL REFERENCES users(id)
);

CREATE INDEX idx_receipts_order ON receipts(order_id);
