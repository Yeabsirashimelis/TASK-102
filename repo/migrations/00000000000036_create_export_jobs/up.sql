CREATE TYPE export_status AS ENUM ('queued', 'running', 'completed', 'failed', 'cancelled');

CREATE TABLE export_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_definition_id UUID NOT NULL REFERENCES report_definitions(id),
    export_format VARCHAR(16) NOT NULL DEFAULT 'xlsx',
    status export_status NOT NULL DEFAULT 'queued',
    total_rows BIGINT,
    processed_rows BIGINT NOT NULL DEFAULT 0,
    progress_pct SMALLINT NOT NULL DEFAULT 0,
    file_path VARCHAR(1024),
    file_size_bytes BIGINT,
    error_message TEXT,
    approval_request_id UUID REFERENCES approval_requests(id),
    requested_by UUID NOT NULL REFERENCES users(id),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_ej_status ON export_jobs(status);
CREATE INDEX idx_ej_requested ON export_jobs(requested_by);
CREATE INDEX idx_ej_report ON export_jobs(report_definition_id);
