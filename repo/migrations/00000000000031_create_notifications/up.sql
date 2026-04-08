CREATE TYPE notification_status AS ENUM ('pending', 'delivered', 'read', 'failed');
CREATE TYPE notification_category AS ENUM ('moderation_outcome', 'comment_reply', 'system_announcement', 'general');

CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipient_user_id UUID NOT NULL REFERENCES users(id),
    template_id UUID REFERENCES notification_templates(id),
    category notification_category NOT NULL DEFAULT 'general',
    subject VARCHAR(512) NOT NULL,
    body TEXT NOT NULL,
    status notification_status NOT NULL DEFAULT 'pending',
    reference_type VARCHAR(64),
    reference_id UUID,
    read_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_notif_recipient ON notifications(recipient_user_id, status);
CREATE INDEX idx_notif_status ON notifications(status);
CREATE INDEX idx_notif_created ON notifications(created_at);
CREATE INDEX idx_notif_category ON notifications(category);
CREATE INDEX idx_notif_ref ON notifications(reference_type, reference_id);
