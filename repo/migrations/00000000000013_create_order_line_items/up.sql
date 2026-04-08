CREATE TABLE order_line_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    sku VARCHAR(128) NOT NULL,
    description VARCHAR(512) NOT NULL,
    quantity INTEGER NOT NULL,
    unit_price_cents BIGINT NOT NULL,
    tax_cents BIGINT NOT NULL DEFAULT 0,
    line_total_cents BIGINT NOT NULL,
    original_line_item_id UUID REFERENCES order_line_items(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_line_items_order ON order_line_items(order_id);
