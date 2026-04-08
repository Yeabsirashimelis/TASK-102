CREATE TYPE schedule_frequency AS ENUM ('daily', 'weekly', 'monthly', 'quarterly');

CREATE TABLE scheduled_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_definition_id UUID NOT NULL REFERENCES report_definitions(id) ON DELETE CASCADE,
    frequency schedule_frequency NOT NULL,
    export_format VARCHAR(16) NOT NULL DEFAULT 'xlsx',
    next_run_at TIMESTAMPTZ NOT NULL,
    last_run_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_sr_next_run ON scheduled_reports(next_run_at) WHERE is_active = TRUE;
CREATE INDEX idx_sr_report ON scheduled_reports(report_definition_id);
