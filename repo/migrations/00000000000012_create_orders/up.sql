CREATE TYPE order_status AS ENUM (
    'draft', 'open', 'tendering', 'paid', 'closed',
    'return_initiated', 'returned',
    'reversal_pending', 'reversed'
);

CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_number VARCHAR(64) NOT NULL UNIQUE,
    status order_status NOT NULL DEFAULT 'draft',
    cashier_user_id UUID NOT NULL REFERENCES users(id),
    location VARCHAR(128) NOT NULL,
    department VARCHAR(128),
    customer_reference VARCHAR(256),
    original_order_id UUID REFERENCES orders(id),
    subtotal_cents BIGINT NOT NULL DEFAULT 0,
    tax_cents BIGINT NOT NULL DEFAULT 0,
    total_cents BIGINT NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_cashier ON orders(cashier_user_id);
CREATE INDEX idx_orders_location ON orders(location);
CREATE INDEX idx_orders_original ON orders(original_order_id);
