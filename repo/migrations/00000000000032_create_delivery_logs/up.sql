CREATE TYPE delivery_result AS ENUM ('success', 'failure');

CREATE TABLE delivery_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    notification_id UUID NOT NULL REFERENCES notifications(id) ON DELETE CASCADE,
    attempt_number INTEGER NOT NULL DEFAULT 1,
    result delivery_result NOT NULL,
    error_message TEXT,
    attempted_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_dl_notification ON delivery_logs(notification_id);
CREATE INDEX idx_dl_attempted ON delivery_logs(attempted_at);
